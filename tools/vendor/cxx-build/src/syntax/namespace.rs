use crate::syntax::qualified::QualifiedName;
use quote::IdentFragment;
use std::fmt::{self, Display};
use std::iter::FromIterator;
use std::slice::Iter;
use syn::parse::{Parse, ParseStream, Result};
use syn::{Ident, Token};

mod kw {
    syn::custom_keyword!(namespace);
}

#[derive(Clone, Default)]
pub struct Namespace {
    segments: Vec<Ident>,
}

impl Namespace {
    pub const ROOT: Self = Namespace {
        segments: Vec::new(),
    };

    pub fn iter(&self) -> Iter<Ident> {
        self.segments.iter()
    }

    pub fn parse_bridge_attr_namespace(input: ParseStream) -> Result<Namespace> {
        if input.is_empty() {
            return Ok(Namespace::ROOT);
        }

        input.parse::<kw::namespace>()?;
        input.parse::<Token![=]>()?;
        let namespace = input.parse::<Namespace>()?;
        input.parse::<Option<Token![,]>>()?;
        Ok(namespace)
    }
}

impl Default for &Namespace {
    fn default() -> Self {
        const ROOT: &Namespace = &Namespace::ROOT;
        ROOT
    }
}

impl Parse for Namespace {
    fn parse(input: ParseStream) -> Result<Self> {
        let segments = QualifiedName::parse_quoted_or_unquoted(input)?.segments;
        Ok(Namespace { segments })
    }
}

impl Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for segment in self {
            write!(f, "{}$", segment)?;
        }
        Ok(())
    }
}

impl IdentFragment for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl<'a> IntoIterator for &'a Namespace {
    type Item = &'a Ident;
    type IntoIter = Iter<'a, Ident>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> FromIterator<&'a Ident> for Namespace {
    fn from_iter<I>(idents: I) -> Self
    where
        I: IntoIterator<Item = &'a Ident>,
    {
        let segments = idents.into_iter().cloned().collect();
        Namespace { segments }
    }
}
