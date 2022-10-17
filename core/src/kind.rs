use std::{convert::TryFrom, str::FromStr};
use strum::{AsRefStr, Display, EnumString};

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, AsRefStr, EnumString)]
pub enum Kind {
    Struct,
    Enum,
    Union,
}

impl quote::ToTokens for Kind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use proc_macro2::{Ident, Span};
        use quote::TokenStreamExt;

        tokens.append(Ident::new(self.as_ref(), Span::call_site()))
    }
}

impl TryFrom<syn::Ident> for Kind {
    type Error = <Kind as FromStr>::Err;

    fn try_from(value: syn::Ident) -> Result<Self, Self::Error> {
        value.to_string().parse()
    }
}
use syn::parse::{Parse, ParseStream};
impl Parse for Kind {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        syn::Ident::parse(input)?
            .try_into()
            .map_err(|e| syn::Error::new(input.span(), format!("{:?}", e)))
    }
}
