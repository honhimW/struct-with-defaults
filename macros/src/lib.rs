use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, Token, Type, Expr, Visibility};
use syn::parse::{Parse, ParseStream, Parser, Result};

#[proc_macro]
pub fn default_field_values(input: TokenStream) -> TokenStream {
    let StructDef { attrs,visibility,   name, generics, fields } = parse_macro_input!(input as StructDef);
    let field_defs = fields.iter().map(|f| {
        let visibility = &f.visibility;
        let ident = &f.ident;
        let ty = &f.ty;
        quote! { #visibility #ident: #ty }
    });

    let constructor_args = fields.iter().filter_map(|f| {
        if f.default.is_none() {
            let ident = &f.ident;
            let ty = &f.ty;
            Some(quote! { #ident: #ty })
        } else {
            None
        }
    });

    let constructor_inits = fields.iter().map(|f| {
        let ident = &f.ident;
        if let Some(default) = &f.default {
            quote! { #ident: #default }
        } else {
            quote! { #ident }
        }
    });

    let (impl_gen, ty_gen, where_gen) = generics.split_for_impl();
    let expanded = if has_derive_default(&attrs) {
        let has_default_inits = fields.iter()
            .filter_map(|f| {
                let ident = &f.ident;
                if let Some(default) = &f.default {
                    Some(quote! { #ident: #default })
                } else {
                    None
                }
            });

        let no_default_inits = fields.iter().filter_map(|f| {
            if f.default.is_none() {
                let ident = &f.ident;
                Some(quote! { #ident: Default::default() })
            } else {
                None
            }
        });
        let strip_default_attrs = strip_default_from_derive(attrs);
        quote! {
            #(#strip_default_attrs)*
            #visibility struct #name #generics #where_gen {
                #(#field_defs,)*
            }

            impl #impl_gen #name #generics {
                pub fn new(#(#constructor_args),*) -> Self {
                    Self {
                        #(#constructor_inits,)*
                    }
                }
            }

            impl #impl_gen Default for #name #generics {
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
            #visibility struct #name #generics #where_gen {
                #(#field_defs,)*
            }

            impl #impl_gen #name #generics {
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

fn has_derive_default(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("derive") {
            return false;
        }
        attr.parse_args_with(|parser: syn::parse::ParseStream| {
            let idents = syn::punctuated::Punctuated::<syn::Ident, syn::Token![,]>::parse_terminated(parser)?;
            Ok(idents.iter().any(|ident| ident == "Default"))
        }).unwrap_or(false)
    })
}

fn strip_default_from_derive(attrs: Vec<syn::Attribute>) -> Vec<syn::Attribute> {
    let mut output: Vec<syn::Attribute> = Vec::new();

    // for attr in attrs {
    //     if attr.path().is_ident("derive") {
    //         let idents = attr.parse_args_with(
    //             |parser: syn::parse::ParseStream| {
    //                 syn::punctuated::Punctuated::<Ident, Token![,]>::parse_terminated(parser)
    //             }
    //         ).unwrap_or_default();
    //
    //         let retained: Vec<_> = idents.into_iter().filter(|i| i != "Default").collect();
    //
    //         if !retained.is_empty() {
    //             output.push(quote! {
    //                 #[derive(#(#retained),*)]
    //             }.into());
    //         }
    //     } else {
    //         output.push(quote! { #attr }.into());
    //     }
    // }

    for attr in attrs {
        if attr.path().is_ident("derive") {
            if let Ok(punct) = attr.parse_args_with(|parser: ParseStream| {
                syn::punctuated::Punctuated::<Ident, Token![,]>::parse_terminated(parser)
            }) {
                let mut iter = punct.into_iter().peekable();
                let mut kept = Vec::new();
                while let Some(ident) = iter.next() {
                    if ident == "Default" {
                        // if let Some(comma) = iter.peek() {
                        //     if comma == "," {
                        //         iter.next();
                        //     }
                        // }
                        continue;
                    }
                    kept.push(ident);
                }

                if kept.is_empty() {
                    continue;
                }

                let new_tokens = quote! { #[derive(#(#kept),*)] };
                eprintln!("{:?}", new_tokens);
                match syn::Attribute::parse_outer.parse2(new_tokens) {
                    Ok(new_attrs) => output.extend(new_attrs),
                    Err(e) => {
                        eprintln!("{:?}", e);
                        output.push(attr)
                    },
                }
            } else {
                output.push(attr); // 无法解析就保留原样
            }
        } else {
            output.push(attr); // 非 derive 直接保留
        }
    }
    output
}

struct StructDef {
    attrs: Vec<syn::Attribute>,
    visibility: Visibility,
    name: Ident,
    generics: syn::Generics,
    fields: Vec<Field>,
}

impl Parse for StructDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let visibility = input.parse()?;
        let _ = input.parse::<Token![struct]>()?;
        let name: Ident = input.parse()?;
        let mut generics: syn::Generics = input.parse()?;
        generics.where_clause = input.parse()?;
        if let Some(ref w) = generics.where_clause {
            eprintln!("{:?}", w.where_token.span);
        } else {
            eprintln!("no where clause");
        }
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
    visibility: Visibility,
    ident: Ident,
    colon_token: Token![:],
    ty: Type,
    eq_token: Option<Token![=]>,
    default: Option<Expr>,
    comma_token: Option<Token![,]>,
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        let visibility = input.parse()?;
        let ident: Ident = input.parse()?;
        let colon_token: Token![:] = input.parse()?;
        let ty: Type = input.parse()?;

        let (eq_token, default) = if input.peek(Token![=]) {
            let eq_token: Token![=] = input.parse()?;
            let default: Expr = input.parse()?;
            (Some(eq_token), Some(default))
        } else {
            (None, None)
        };

        let comma_token = input.parse().ok();

        Ok(Field {
            visibility,
            ident,
            colon_token,
            ty,
            eq_token,
            default,
            comma_token,
        })
    }
}