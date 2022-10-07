extern crate proc_macro;
use proc_macro::TokenStream;
use syn::spanned::Spanned;

mod fields;
mod multi_derive;
mod single_derive;

pub(crate) use harled_core::Kind;

#[proc_macro_derive(FromDeriveInput, attributes(harled))]
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
                .filter_map(|attr| {
                    (attr.path.get_ident().unwrap() == "harled").then(|| {
                        //syn parse to proc_macro2::Group ?
                        let group: proc_macro2::Group = syn::parse(attr.tokens.clone().into())
                            .map_err(|_| "Attr is not a group")?;
                        if group.delimiter() == proc_macro2::Delimiter::Parenthesis {
                            Ok(group
                                .stream()
                                .to_string()
                                .parse::<Kind>()
                                .map_err(|_| "Attr is not a kind")?)
                        } else {
                            Err("Attr is not paren delimitered")
                        }
                    })
                })
                .fold(Count::None, |acc, res| match (&acc, res) {
                    (Count::None, Ok(k)) => Count::One(k),
                    (Count::One(_), Ok(_)) => Count::Many,
                    _ => acc,
                });

            match kind {
                Count::One(k) => single_derive::SingleDerive::new(k, ast).derive().into(),
                Count::None => syn::Error::new(
                    ast.span(),
                    concat!(
                        "FromDeriveInput requires the `#[harled(<Kind>)]` attribute,\n",
                        "where `<Kind>` is a variant of harled_core::Kind"
                    ),
                )
                .to_compile_error()
                .into(),
                Count::Many => syn::Error::new(
                    ast.span(),
                    concat!(
                        "FromDeriveInput currently only supports one kind at a time for structs",
                        "Try using an enum instead"
                    ),
                )
                .to_compile_error()
                .into(),
            }
        }
        syn::Data::Union(_) => {
            return syn::Error::new(
                ast.span(),
                "FromDeriveInput can only be derived for structs and enums",
            )
            .to_compile_error()
            .into();
        }
    }
}
