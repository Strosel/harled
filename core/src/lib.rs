extern crate proc_macro;

pub(crate) mod kind;
#[doc(inline)]
pub use kind::Kind;

#[derive(Debug, Clone)]
pub enum Error {
    Unsupported(kind::Kind, proc_macro2::Span),
    Syn(syn::Error),
}

impl From<syn::Error> for Error {
    fn from(error: syn::Error) -> Self {
        Self::Syn(error)
    }
}

pub trait DeriveInput {
    fn input(self) -> syn::DeriveInput;
}

impl DeriveInput for syn::DeriveInput {
    fn input(self) -> syn::DeriveInput {
        self
    }
}

impl DeriveInput for proc_macro::TokenStream {
    fn input(self) -> syn::DeriveInput {
        syn::parse(self).unwrap()
    }
}

impl DeriveInput for proc_macro2::TokenStream {
    fn input(self) -> syn::DeriveInput {
        syn::parse(self.into()).unwrap()
    }
}

pub trait FromDeriveInput: Sized {
    type Error;
    fn parse<T: DeriveInput>(input: T) -> Result<Self, Self::Error>;
}

pub fn parse<D: DeriveInput, T: FromDeriveInput>(input: D) -> Result<T, T::Error> {
    T::parse(input)
}
