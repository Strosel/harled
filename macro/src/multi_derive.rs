use crate::Kind;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{spanned::Spanned, Error};

pub struct MultiDerive {
    ident: syn::Ident,
    data: syn::DataEnum,
    support: HashMap<Kind, syn::Type>,
}

impl MultiDerive {
    pub fn new(input: syn::DeriveInput) -> Self {
        Self {
            ident: input.ident,
            data: if let syn::Data::Enum(e) = input.data {
                e
            } else {
                panic!("MultiDerive was passed non-enum")
            },
            support: HashMap::new(),
        }
    }

    pub fn validate(&mut self) -> syn::Result<()> {
        if self.data.variants.is_empty() {
            return Err(Error::new(
                self.ident.span(),
                "FromDeriveInput must have at least one variant",
            ));
        }

        for var in self.data.variants.iter() {
            if ![Kind::Struct, Kind::Enum, Kind::Union]
                .map(|k| k.to_string())
                .contains(&var.ident.to_string())
            {
                return Err(Error::new(
                    var.span(),
                    format!(
                        "FromDeriveInput variants must have names `{}`,`{}` or `{}`",
                        Kind::Struct,
                        Kind::Enum,
                        Kind::Union,
                    ),
                ));
            }

            let field = if let syn::Fields::Unnamed(ref fields) = var.fields {
                if fields.unnamed.len() == 1 {
                    fields.unnamed[0].clone()
                } else {
                    return Err(Error::new(
                        var.ident.span(),
                        "FromDeriveInput variants only support 1 field",
                    ));
                }
            } else {
                return Err(Error::new(
                    var.ident.span(),
                    "FromDeriveInput variants only support unnamed fields",
                ));
            };

            if var.ident == Kind::Struct {
                self.support.insert(Kind::Struct, field.ty);
            } else if var.ident == Kind::Enum {
                self.support.insert(Kind::Enum, field.ty);
            } else if var.ident == Kind::Union {
                self.support.insert(Kind::Union, field.ty);
            }
        }

        Ok(())
    }

    pub fn derive(mut self) -> TokenStream {
        if let Err(e) = self.validate() {
            return e.into_compile_error();
        }

        let Self { ident, support, .. } = self;

        //NOTE should both inner types and struct like enums be allowed or just inner types?
        let branches = [Kind::Struct, Kind::Enum, Kind::Union].into_iter().fold(quote!(), |mut branches, variant| {
            let span = match variant {
                Kind::Struct => quote!(struct_token),
                Kind::Enum => quote!(enum_token),
                Kind::Union => quote!(union_token),
            };

            branches.extend(match support.get(&variant) {
                Some(ref ty) => {
                    quote! {
                        ::harled::syn::Data::#variant(_) => Ok(Self::#variant(<#ty as ::harled::FromDeriveInput>::parse(ast)?)),
                    }
                }
                None => {
                    quote! {
                        ::harled::syn::Data::#variant(s) => {
                            use ::harled::syn::spanned::Spanned;
                            Err(::harled::Error::Unsupported(
                                ::harled::Kind::#variant,
                                s.#span.span,
                            ))
                        }
                    }
                }
            });
            branches
        });

        quote! {
            impl ::harled::FromDeriveInput for #ident {
                type Error = ::harled::Error;
                fn parse<T: ::harled::DeriveInput>(input: T) -> Result<Self, Self::Error> {
                    let ast = input.input();
                    match ast.data {
                        #branches
                    }
                }
            }
        }
    }
}
