use crate::Kind;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{spanned::Spanned, Error};

#[derive(PartialEq, Eq)]
enum DeriveVariant {
    Type(syn::Type),
    StructLike(syn::FieldsNamed),
}

pub struct MultiDerive {
    ident: syn::Ident,
    data: syn::DataEnum,
    support: HashMap<Kind, DeriveVariant>,
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
                        "FromDeriveInput enum variants must have names `{}`,`{}` or `{}`",
                        Kind::Struct,
                        Kind::Enum,
                        Kind::Union,
                    ),
                ));
            }

            match var.fields {
                syn::Fields::Unnamed(ref fields) => {
                    if fields.unnamed.len() != 1 {
                        return Err(Error::new(
                            var.ident.span(),
                            "FromDeriveInput variants only support 1 unnamed field",
                        ));
                    }
                    let field = fields.unnamed[0];

                    if var.ident == Kind::Struct {
                        self.support
                            .insert(Kind::Struct, DeriveVariant::Type(field.ty));
                    } else if var.ident == Kind::Enum {
                        self.support
                            .insert(Kind::Enum, DeriveVariant::Type(field.ty));
                    } else if var.ident == Kind::Union {
                        self.support
                            .insert(Kind::Union, DeriveVariant::Type(field.ty));
                    }
                }
                syn::Fields::Named(fields) => {
                    if var.ident == Kind::Struct {
                        self.support
                            .insert(Kind::Struct, DeriveVariant::StructLike(fields));
                    } else if var.ident == Kind::Enum {
                        self.support
                            .insert(Kind::Enum, DeriveVariant::StructLike(fields));
                    } else if var.ident == Kind::Union {
                        self.support
                            .insert(Kind::Union, DeriveVariant::StructLike(fields));
                    }
                }
                syn::Fields::Unit => {
                    return Err(Error::new(
                        var.ident.span(),
                        "FromDeriveInput cannot be derived for unit variants",
                    ));
                }
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
                Some(DeriveVariant::Type(ref ty)) => {
                    quote! {
                        ::harled::syn::Data::#variant(_) => Ok(Self::#variant(<#ty as ::harled::FromDeriveInput>::parse(ast)?)),
                    }
                }
                Some(DeriveVariant::StructLike(_)) => todo!(),
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
