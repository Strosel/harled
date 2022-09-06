#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Struct,
    Enum,
    Union,
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

//impl's PartialEq<Ident> via blanket implementation
impl AsRef<str> for Kind {
    fn as_ref(&self) -> &str {
        match self {
            Kind::Struct => "Struct",
            Kind::Enum => "Enum",
            Kind::Union => "Union",
        }
    }
}

impl std::str::FromStr for Kind {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Struct" => Ok(Kind::Struct),
            "Enum" => Ok(Kind::Enum),
            "Union" => Ok(Kind::Union),
            _ => Err("Unknown Kind"),
        }
    }
}

impl quote::ToTokens for Kind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use proc_macro2::{Ident, Span};
        use quote::TokenStreamExt;

        tokens.append(Ident::new(self.as_ref(), Span::call_site()))
    }
}
