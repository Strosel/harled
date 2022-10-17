use crate::{fields, ComboKind, Kind};
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::Error;

pub(crate) struct SingleDerive {
    kind: ComboKind,
    ident: syn::Ident,
    data: syn::DataStruct,
    used_fields: HashSet<String>,
}

impl SingleDerive {
    pub(crate) fn new(kind: ComboKind, input: syn::DeriveInput) -> Self {
        Self {
            kind,
            ident: input.ident,
            data: if let syn::Data::Struct(s) = input.data {
                s
            } else {
                panic!("SingleDerive was passed non-struct")
            },
            used_fields: HashSet::new(),
        }
    }

    fn validate(&mut self) -> syn::Result<()> {
        if let syn::Fields::Named(ref fields) = self.data.fields {
            self.used_fields = self
                .kind
                .iter()
                .map(|kind| fields::validate_fields(*kind, fields))
                .try_fold(HashSet::<String>::new(), |hs, fields| {
                    if hs.is_empty() {
                        fields
                    } else {
                        Ok(&hs & &fields?)
                    }
                })?;

            if self.kind.len() > 1 {
                //NOTE currently usupported due to Structs and Unions having different field types
                self.used_fields.remove("fields");
            }
        } else {
            return Err(Error::new(
                self.ident.span(),
                "FromDeriveInput only support named struct fields",
            ));
        }

        Ok(())
    }

    pub(crate) fn derive(mut self) -> TokenStream {
        if let Err(e) = self.validate() {
            return e.into_compile_error();
        }

        let Self {
            ident, used_fields, ..
        } = self;

        let construct = fields::construct_fields(&used_fields);

        let branches = self
            .kind
            .iter()
            .map(|kind| match kind {
                Kind::Struct => quote! {
                    ::harled::syn::Data::Struct(s)=> Ok(Self{
                        #construct
                    }),
                },
                Kind::Enum => quote! {
                    ::harled::syn::Data::Enum(s)=> Ok(Self{
                        #construct
                    }),
                },
                Kind::Union => quote! {
                    ::harled::syn::Data::Union(s)=> Ok(Self{
                        #construct
                    }),
                },
            })
            .reduce(|l, r| quote!(#l #r))
            .unwrap();

        quote! {
            impl ::harled::FromDeriveInput for #ident {
                type Error = ::harled::syn::Error;
                fn parse<T: ::harled::DeriveInput>(input: T) -> Result<Self, Self::Error> {
                    use ::harled::syn::spanned::Spanned;
                    let ast = input.input();
                    match ast.data {
                        #branches
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
