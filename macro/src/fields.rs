use crate::Kind;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::{spanned::Spanned, Error};

fn legal_fields(kind: Kind) -> HashSet<String> {
    match kind {
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

pub fn validate_fields(kind: Kind, fields: &syn::FieldsNamed) -> syn::Result<Vec<String>> {
    let legal_fields = legal_fields(kind);

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

    Ok(fields.intersection(&legal_fields).cloned().collect())
}

pub fn construct_fields(used_fields: &Vec<String>) -> TokenStream {
    used_fields
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
        })
}
