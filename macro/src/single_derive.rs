use crate::{fields, Kind};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Error;

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

    fn validate(&mut self) -> syn::Result<()> {
        if let syn::Fields::Named(ref fields) = self.data.fields {
            self.used_fields = fields::validate_fields(self.kind, fields)?;
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

        let construct = fields::construct_fields(&used_fields);

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
