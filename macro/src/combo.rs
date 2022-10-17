use crate::Kind;
use proc_macro2::{TokenStream, TokenTree};
use std::ops::Deref;
use syn::{
    parse::{Parse, ParseStream, Peek, Result},
    punctuated::Punctuated,
    Token,
};

#[derive(Default)]
pub(crate) struct ComboKind(Punctuated<Kind, syn::Token![|]>);

fn parse_until<E: Peek>(input: ParseStream, end: E) -> Result<TokenStream> {
    let mut tokens = TokenStream::new();
    while !input.is_empty() && !input.peek(end) {
        let next: TokenTree = input.parse()?;
        tokens.extend(Some(next));
    }
    Ok(tokens)
}

impl Parse for ComboKind {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut out = Self::default();

        let first = parse_until(input, Token![|])?;
        out.0.push_value(syn::parse2(first)?);

        while input.peek(Token![|]) {
            out.0.push_punct(input.parse()?);

            let next = parse_until(input, Token![|])?;
            out.0.push_value(syn::parse2(next)?);
        }

        Ok(out)
    }
}

impl Deref for ComboKind {
    type Target = Punctuated<Kind, Token![|]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
