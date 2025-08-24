use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream, Parser, Result};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, token, Attribute, Expr, Generics, Ident, Token, Type, Visibility};

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
    let struct_kind = parse_macro_input!(input as StructKind);
    match struct_kind {
        StructKind::Braced(b) => gen_braced(b),
        StructKind::Parenthesized(p) => gen_parenthesized(p),
    }
}

fn gen_braced(def: BracedDef) -> TokenStream {
    let BracedDef {
        attrs,
        visibility,
        name,
        generics,
        fields,
    } = def;
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

    let constructor_args = fields.iter().filter_map(|f| match &f.default {
        DefaultValue::None => {
            let ident = &f.ident;
            let ty = &f.ty;
            Some(quote! { #ident: #ty })
        }
        _ => None,
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
        let has_default_inits = fields.iter().filter_map(|f| {
            let ident = &f.ident;
            match &f.default {
                DefaultValue::None => None,
                DefaultValue::Underscore => Some(quote! { #ident: Default::default() }),
                DefaultValue::Expr(default_expr) => Some(quote! { #ident: #default_expr }),
            }
        });

        let no_default_inits = fields.iter().filter_map(|f| match &f.default {
            DefaultValue::None => {
                let ident = &f.ident;
                Some(quote! { #ident: Default::default() })
            }
            DefaultValue::Underscore => None,
            DefaultValue::Expr(_) => None,
        });
        let strip_default_attrs = strip_default_from_derive(attrs);
        quote! {
            #(#strip_default_attrs)*
            #visibility struct #name #type_gen #where_gen {
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
            #visibility struct #name #type_gen #where_gen {
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

fn gen_parenthesized(def: ParenthesizedDef) -> TokenStream {
    let ParenthesizedDef {
        attrs,
        visibility,
        name,
        generics,
        elements,
    } = def;
    let element_defs = elements.iter().map(|f| {
        let attrs = &f.attrs;
        let visibility = &f.visibility;
        let ty = &f.ty;
        quote! {
            #(#attrs)*
            #visibility #ty
        }
    });

    let constructor_args = elements
        .iter()
        .enumerate()
        .filter_map(|(i, e)| match &e.default {
            DefaultValue::None => {
                let ident = format_ident!("_{}", i);
                let ty = &e.ty;
                Some(quote! { #ident: #ty })
            }
            _ => None,
        });

    let constructor_inits = elements.iter().enumerate().map(|(i, e)| {
        let ident = format_ident!("_{}", i);
        match &e.default {
            DefaultValue::None => quote! { #ident },
            DefaultValue::Underscore => quote! { Default::default() },
            DefaultValue::Expr(default_expr) => quote! { #default_expr },
        }
    });

    let (impl_gen, type_gen, _) = generics.split_for_impl();
    let expanded = if has_derive_default(&attrs) {
        let inits = elements.iter().filter_map(|e| match &e.default {
            DefaultValue::None | DefaultValue::Underscore => Some(quote! { Default::default() }),
            DefaultValue::Expr(default_expr) => Some(quote! { #default_expr }),
        });
        let strip_default_attrs = strip_default_from_derive(attrs);
        quote! {
            #(#strip_default_attrs)*
            #visibility struct #name #type_gen(#(#element_defs,)*);

            impl #impl_gen #name #type_gen {
                pub fn new(#(#constructor_args),*) -> Self {
                    Self(#(#constructor_inits,)*)
                }
            }

            impl #impl_gen Default for #name #type_gen {
                fn default() -> Self {
                    Self(#(#inits,)*)
                }
            }
        }
    } else {
        quote! {
            #(#attrs)*
            #visibility struct #name #type_gen(#(#element_defs,)*);

            impl #impl_gen #name #type_gen {
                pub fn new(#(#constructor_args),*) -> Self {
                    Self (#(#constructor_inits,)*)
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
        })
        .unwrap_or(false)
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

enum DefaultValue {
    None,
    Underscore,
    Expr(Expr),
}

struct BracedDef {
    attrs: Vec<Attribute>,
    visibility: Visibility,
    name: Ident,
    generics: Generics,
    fields: Vec<Field>,
}

struct ParenthesizedDef {
    attrs: Vec<Attribute>,
    visibility: Visibility,
    name: Ident,
    generics: Generics,
    elements: Vec<Element>,
}

#[allow(unused)]
struct Field {
    attrs: Vec<Attribute>,
    visibility: Visibility,
    ident: Ident,
    ty: Type,
    default: DefaultValue,
}

#[allow(unused)]
struct Element {
    attrs: Vec<Attribute>,
    visibility: Visibility,
    ty: Type,
    default: DefaultValue,
}

enum StructKind {
    Braced(BracedDef),
    Parenthesized(ParenthesizedDef),
}

impl Parse for StructKind {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let visibility = input.parse()?;
        let _ = input.parse::<Token![struct]>()?;
        let name: Ident = input.parse()?;
        let mut generics: Generics = input.parse()?;
        generics.where_clause = input.parse()?;
        let content;
        if input.peek(token::Brace) {
            syn::braced!(content in input);
            let mut fields = Vec::new();
            while !content.is_empty() {
                fields.push(content.parse()?);
            }
            let def = BracedDef {
                attrs,
                visibility,
                name,
                generics,
                fields,
            };
            Ok(StructKind::Braced(def))
        } else {
            syn::parenthesized!(content in input);
            let mut fields = Vec::new();
            while !content.is_empty() {
                fields.push(content.parse()?);
            }
            let _ = input.parse::<Token![;]>()?;
            let def = ParenthesizedDef {
                attrs,
                visibility,
                name,
                generics,
                elements: fields,
            };
            Ok(StructKind::Parenthesized(def))
        }
    }
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

impl Parse for Element {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let visibility = input.parse()?;
        let ty: Type = input.parse()?;

        let default = if input.peek(Token![=]) {
            let _eq_token: Token![=] = input.parse()?;
            let default: DefaultValue = input.parse()?;
            default
        } else {
            DefaultValue::None
        };

        let _comma_token: Result<Token![,]> = input.parse();

        Ok(Element {
            attrs,
            visibility,
            ty,
            default,
        })
    }
}
