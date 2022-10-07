extern crate proc_macro;
use proc_macro::TokenStream;
use syn::spanned::Spanned;

mod fields;
mod multi_derive;
mod single_derive;

pub(crate) use harled_core::Kind;

#[proc_macro_derive(FromDeriveInput, attributes(Struct, Enum, Union))]
#[allow(clippy::needless_return)] //used with rustfmt::skip to trick rustfmt
pub fn derive_from_derive_input(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    match &ast.data {
        syn::Data::Enum(_) => multi_derive::MultiDerive::new(ast).derive().into(),
        syn::Data::Struct(_) => {
            enum Count {
                None,
                One(Kind),
                Many,
            }

            let kind = ast
                .attrs
                .iter()
                .map(|attr| attr.path.get_ident().unwrap().to_string().parse::<Kind>())
                .fold(Count::None, |acc, res| match (&acc, res) {
                    (Count::None, Ok(k)) => Count::One(k),
                    (Count::One(_), Ok(_)) => Count::Many,
                    _ => acc,
                });

            match kind {
                Count::One(k) => single_derive::SingleDerive::new(k, ast).derive().into(),
                _ => {
                    #[rustfmt::skip]
                    return syn::Error::new(
                        ast.span(),
                        format!(
                            "FromDeriveInput on a struct requires exactly one of the following attributes `{}`, `{}`, `{}`",
                             Kind::Struct, Kind::Enum, Kind::Union,
                        )
                    )
                    .to_compile_error()
                    .into();
                }
            }
        }
        syn::Data::Union(_) => {
            return syn::Error::new(
                ast.span(),
                "FromDeriveInput can only be derived for structs with attributes and enums",
            )
            .to_compile_error()
            .into();
        }
    }
}
