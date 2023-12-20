use crate::mapref;
use core::hash::{BuildHasher, Hash};
use core::ops::Deref;
use std::collections::hash_map::RandomState;
pub struct RefMulti<'a, K, S = RandomState> {
    inner: mapref::multiple::RefMulti<'a, K, (), S>,
}

impl<'a, K: Eq + Hash, S: BuildHasher> RefMulti<'a, K, S> {
    pub(crate) fn new(inner: mapref::multiple::RefMulti<'a, K, (), S>) -> Self {
        Self { inner }
    }

    pub fn key(&self) -> &K {
        self.inner.key()
    }
}

impl<'a, K: Eq + Hash, S: BuildHasher> Deref for RefMulti<'a, K, S> {
    type Target = K;

    fn deref(&self) -> &K {
        self.key()
    }
}
