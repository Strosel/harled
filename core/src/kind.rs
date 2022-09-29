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
