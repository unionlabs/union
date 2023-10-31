use core::iter::Iterator;
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::{parse_quote, Error, Generics, Path, TypeGenerics};

/// Merge multiple [`syn::Error`] into one.
pub(crate) trait IteratorExt {
    fn collect_error(self) -> Result<(), Error>
    where
        Self: Iterator<Item = Result<(), Error>> + Sized,
    {
        let accu = Ok(());
        self.fold(accu, |accu, error| match (accu, error) {
            (Ok(()), error) => error,
            (accu, Ok(())) => accu,
            (Err(mut err), Err(error)) => {
                err.combine(error);
                Err(err)
            }
        })
    }
}
impl<I> IteratorExt for I where I: Iterator<Item = Result<(), Error>> + Sized {}

/// Attributes usable for derive macros
#[derive(FromDeriveInput)]
#[darling(attributes(serde_with))]
pub(crate) struct DeriveOptions {
    /// Path to the crate
    #[darling(rename = "crate", default)]
    pub(crate) alt_crate_path: Option<Path>,
}

impl DeriveOptions {
    pub(crate) fn from_derive_input(input: &syn::DeriveInput) -> Result<Self, TokenStream> {
        match <Self as FromDeriveInput>::from_derive_input(input) {
            Ok(v) => Ok(v),
            Err(e) => Err(TokenStream::from(e.write_errors())),
        }
    }

    pub(crate) fn get_serde_with_path(&self) -> Path {
        self.alt_crate_path
            .clone()
            .unwrap_or_else(|| syn::parse_str("::serde_with").unwrap())
    }
}

// Inspired by https://github.com/serde-rs/serde/blob/fb2fe409c8f7ad6c95e3096e5e9ede865c8cfb49/serde_derive/src/de.rs#L3120
// Serde is also licensed Apache 2 + MIT
pub(crate) fn split_with_de_lifetime(
    generics: &Generics,
) -> (
    DeImplGenerics<'_>,
    TypeGenerics<'_>,
    Option<&syn::WhereClause>,
) {
    let de_impl_generics = DeImplGenerics(generics);
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    (de_impl_generics, ty_generics, where_clause)
}

pub(crate) struct DeImplGenerics<'a>(&'a Generics);

impl<'a> ToTokens for DeImplGenerics<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let mut generics = self.0.clone();
        generics.params = Some(parse_quote!('de))
            .into_iter()
            .chain(generics.params)
            .collect();
        let (impl_generics, _, _) = generics.split_for_impl();
        impl_generics.to_tokens(tokens);
    }
}
