use crate::Kind;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::{spanned::Spanned, Error};

pub(crate) struct SingleDerive {
    kind: Kind,
    ident: syn::Ident,
    data: syn::DataStruct,
    used_fields: Vec<String>,
}

impl SingleDerive {
    pub(crate) fn new(kind: Kind, input: syn::DeriveInput) -> Self {
        Self {
            kind,
            ident: input.ident,
            data: if let syn::Data::Struct(s) = input.data {
                s
            } else {
                panic!("SingleDerive was passed non-struct")
            },
            used_fields: vec![],
        }
    }

    fn legal_fields(&mut self) -> HashSet<String> {
        match self.kind {
            Kind::Struct => [
                "attrs",
                "vis",
                "struct_token",
                "ident",
                "generics",
                "fields",
            ],
            Kind::Enum => [
                "attrs",
                "vis",
                "enum_token",
                "ident",
                "generics",
                "variants",
            ],
            Kind::Union => ["attrs", "vis", "union_token", "ident", "generics", "fields"],
        }
        .map(String::from)
        .into()
    }

    fn validate(&mut self) -> syn::Result<()> {
        let legal_fields = self.legal_fields();

        if let syn::Fields::Named(ref fields) = self.data.fields {
            let fields: HashSet<_> = fields
                .named
                .iter()
                .filter_map(|f| f.ident.as_ref())
                .map(|f| f.to_string())
                .collect();

            if let Some(ident) = fields.difference(&legal_fields).next() {
                return Err(Error::new(
                    ident.span(),
                    format!("Unsupported field `{}`", ident),
                ));
            }

            self.used_fields = fields.intersection(&legal_fields).cloned().collect();
        } else {
            return Err(Error::new(
                self.ident.span(),
                "FromDeriveInput only support named struct fields",
            ));
        };

        Ok(())
    }

    pub(crate) fn derive(mut self) -> TokenStream {
        if let Err(e) = self.validate() {
            return e.into_compile_error();
        }

        let Self {
            ident, used_fields, ..
        } = self;

        let branch = match self.kind {
            Kind::Struct => quote!(::harled::syn::Data::Struct(s)),
            Kind::Enum => quote!(::harled::syn::Data::Enum(s)),
            Kind::Union => quote!(::harled::syn::Data::Union(s)),
        };

        let construct = used_fields
            .into_iter()
            .fold(quote!(), |mut construct, field| {
                construct.extend(match field.as_str() {
                    "attrs" => quote! {
                        attrs: ast.attrs,
                    },
                    "vis" => quote! {
                        vis: ast.vis,
                    },
                    "struct_token" => quote! {
                        struct_token: s.struct_token,
                    },
                    "enum_token" => quote! {
                        enum_token: s.enum_token,
                    },
                    "union_token" => quote! {
                        union_token: s.union_token,
                    },
                    "ident" => quote! {
                        ident: ast.ident,
                    },
                    "generics" => quote! {
                        generics: ast.generics,
                    },
                    "fields" => quote! {
                        fields: s.fields,
                    },
                    "variants" => quote! {
                        variants: s.variants.into_iter().collect(),
                    },
                    _ => unreachable!(),
                });
                construct
            });

        quote! {
            impl ::harled::FromDeriveInput for #ident {
                type Error = ::harled::syn::Error;
                fn parse<T: ::harled::DeriveInput>(input: T) -> Result<Self, Self::Error> {
                    use ::harled::syn::spanned::Spanned;
                    let ast = input.input();
                    match ast.data {
                        #branch => Ok(Self{
                            #construct
                        }),
                        _ => Err(::harled::syn::Error::new(
                            ast.span(),
                            "#[derive(FromDeriveInput)] with attribute can only be used on structs"
                        )),
                    }
                }
            }
        }
    }
}
