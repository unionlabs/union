use crate::mapref;
use core::hash::{BuildHasher, Hash};
use core::ops::Deref;
use std::collections::hash_map::RandomState;
pub struct Ref<'a, K, S = RandomState> {
    inner: mapref::one::Ref<'a, K, (), S>,
}

impl<'a, K: Eq + Hash, S: BuildHasher> Ref<'a, K, S> {
    pub(crate) fn new(inner: mapref::one::Ref<'a, K, (), S>) -> Self {
        Self { inner }
    }

    pub fn key(&self) -> &K {
        self.inner.key()
    }
}

impl<'a, K: Eq + Hash, S: BuildHasher> Deref for Ref<'a, K, S> {
    type Target = K;

    fn deref(&self) -> &K {
        self.key()
    }
}
