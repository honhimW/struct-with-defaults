use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, Token, Type, Expr, Visibility, Attribute, Generics};
use syn::parse::{Parse, ParseStream, Parser, Result};
use syn::punctuated::Punctuated;

/// ```rust
/// use macro3681::default_field_values;
///
/// default_field_values! {
///     pub struct Foo {
///         i: i32,
///         j: u32 = 1,
///         string: String = {
///             let s = format!("{} {}", "foo", "bar");
///             s
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn default_field_values(input: TokenStream) -> TokenStream {
    let StructDef { attrs,visibility,   name, generics, fields } = parse_macro_input!(input as StructDef);
    let field_defs = fields.iter().map(|f| {
        let attrs = &f.attrs;
        let visibility = &f.visibility;
        let ident = &f.ident;
        let ty = &f.ty;
        quote! {
            #(#attrs)*
            #visibility #ident: #ty
        }
    });

    let constructor_args = fields.iter().filter_map(|f| {
        match &f.default {
            DefaultValue::None => {
                let ident = &f.ident;
                let ty = &f.ty;
                Some(quote! { #ident: #ty })
            }
            _ => None,
        }
    });

    let constructor_inits = fields.iter().map(|f| {
        let ident = &f.ident;
        match &f.default {
            DefaultValue::None => quote! { #ident },
            DefaultValue::Underscore => quote! { #ident: Default::default() },
            DefaultValue::Expr(default_expr) => quote! { #ident: #default_expr },
        }
    });

    let (impl_gen, type_gen, where_gen) = generics.split_for_impl();
    let expanded = if has_derive_default(&attrs) {
        let has_default_inits = fields.iter()
            .filter_map(|f| {
                let ident = &f.ident;
                match &f.default {
                    DefaultValue::None => None,
                    DefaultValue::Underscore => Some(quote! { #ident: Default::default() }),
                    DefaultValue::Expr(default_expr) => Some(quote! { #ident: #default_expr }),
                }
            });

        let no_default_inits = fields.iter().filter_map(|f| {
            match &f.default {
                DefaultValue::None => {
                    let ident = &f.ident;
                    Some(quote! { #ident: Default::default() })
                },
                DefaultValue::Underscore => None,
                DefaultValue::Expr(_) => None,
            }
        });
        let strip_default_attrs = strip_default_from_derive(attrs);
        quote! {
            #(#strip_default_attrs)*
            #visibility struct #name #generics {
                #(#field_defs,)*
            }

            impl #impl_gen #name #type_gen #where_gen {
                pub fn new(#(#constructor_args),*) -> Self {
                    Self {
                        #(#constructor_inits,)*
                    }
                }
            }

            impl #impl_gen Default for #name #type_gen #where_gen {
                fn default() -> Self {
                    Self {
                        #(#has_default_inits,)*
                        #(#no_default_inits,)*
                    }
                }
            }

        }
    } else {
        quote! {
            #(#attrs)*
            #visibility struct #name #generics {
                #(#field_defs,)*
            }

            impl #impl_gen #name #type_gen #where_gen {
                pub fn new(#(#constructor_args),*) -> Self {
                    Self {
                        #(#constructor_inits,)*
                    }
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn has_derive_default(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("derive") {
            return false;
        }
        attr.parse_args_with(|parser: ParseStream| {
            let idents = Punctuated::<Ident, Token![,]>::parse_terminated(parser)?;
            Ok(idents.iter().any(|ident| ident == "Default"))
        }).unwrap_or(false)
    })
}

fn strip_default_from_derive(attrs: Vec<Attribute>) -> Vec<Attribute> {
    let mut output: Vec<Attribute> = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("derive") {
            if let Ok(punct) = attr.parse_args_with(|parser: ParseStream| {
                Punctuated::<Ident, Token![,]>::parse_terminated(parser)
            }) {
                let mut iter = punct.into_iter().peekable();
                let mut kept = Vec::new();
                while let Some(ident) = iter.next() {
                    if ident == "Default" {
                        continue;
                    }
                    kept.push(ident);
                }

                if kept.is_empty() {
                    continue;
                }

                let new_tokens = quote! { #[derive(#(#kept),*)] };
                match Attribute::parse_outer.parse2(new_tokens) {
                    Ok(new_attrs) => output.extend(new_attrs),
                    Err(_) => output.push(attr),
                }
            } else {
                output.push(attr);
            }
        } else {
            output.push(attr);
        }
    }
    output
}

struct StructDef {
    attrs: Vec<Attribute>,
    visibility: Visibility,
    name: Ident,
    generics: Generics,
    fields: Vec<Field>,
}

impl Parse for StructDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let visibility = input.parse()?;
        let _ = input.parse::<Token![struct]>()?;
        let name: Ident = input.parse()?;
        let mut generics: Generics = input.parse()?;
        generics.where_clause = input.parse()?;
        let content;
        syn::braced!(content in input);
        let mut fields = Vec::new();
        while !content.is_empty() {
            fields.push(content.parse()?);
        }
        Ok(StructDef { attrs, visibility, name, generics, fields })
    }
}

#[allow(unused)]
struct Field {
    attrs: Vec<Attribute>,
    visibility: Visibility,
    ident: Ident,
    ty: Type,
    default: DefaultValue,
}

enum DefaultValue {
    None,
    Underscore,
    Expr(Expr),
}

impl Parse for DefaultValue {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![_]) {
            input.parse::<Token![_]>()?;
            Ok(DefaultValue::Underscore)
        } else {
            let expr: Expr = input.parse()?;
            Ok(DefaultValue::Expr(expr))
        }
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let visibility = input.parse()?;
        let ident: Ident = input.parse()?;
        let _colon_token: Token![:] = input.parse()?;
        let ty: Type = input.parse()?;

        let default = if input.peek(Token![=]) {
            let _eq_token: Token![=] = input.parse()?;
            let default: DefaultValue = input.parse()?;
            default
        } else {
            DefaultValue::None
        };

        let _comma_token: Result<Token![,]> = input.parse();

        Ok(Field {
            attrs,
            visibility,
            ident,
            ty,
            default,
        })
    }
}