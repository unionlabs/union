use crate::syntax::instantiate::NamedImplKey;
use crate::syntax::{Lifetimes, NamedType, Pair, Types};
use proc_macro2::Ident;

#[derive(Copy, Clone)]
pub struct Resolution<'a> {
    pub name: &'a Pair,
    pub generics: &'a Lifetimes,
}

impl<'a> Types<'a> {
    pub fn resolve(&self, ident: &impl UnresolvedName) -> Resolution<'a> {
        let ident = ident.ident();
        match self.try_resolve(ident) {
            Some(resolution) => resolution,
            None => panic!("Unable to resolve type `{}`", ident),
        }
    }

    pub fn try_resolve(&self, ident: &impl UnresolvedName) -> Option<Resolution<'a>> {
        let ident = ident.ident();
        self.resolutions.get(ident).copied()
    }
}

pub trait UnresolvedName {
    fn ident(&self) -> &Ident;
}

impl UnresolvedName for Ident {
    fn ident(&self) -> &Ident {
        self
    }
}

impl UnresolvedName for NamedType {
    fn ident(&self) -> &Ident {
        &self.rust
    }
}

impl<'a> UnresolvedName for NamedImplKey<'a> {
    fn ident(&self) -> &Ident {
        self.rust
    }
}
