use crate::prelude::*;
#[cfg(feature = "indexmap_1")]
use indexmap_1::IndexMap;

pub trait DuplicateInsertsFirstWinsMap<K, V> {
    fn new(size_hint: Option<usize>) -> Self;

    /// Insert the value into the map, if there is not already an existing value
    fn insert(&mut self, key: K, value: V);
}

#[cfg(feature = "std")]
impl<K, V, S> DuplicateInsertsFirstWinsMap<K, V> for HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher + Default,
{
    #[inline]
    fn new(size_hint: Option<usize>) -> Self {
        match size_hint {
            Some(size) => Self::with_capacity_and_hasher(size, S::default()),
            None => Self::with_hasher(S::default()),
        }
    }

    #[inline]
    fn insert(&mut self, key: K, value: V) {
        use std::collections::hash_map::Entry;

        match self.entry(key) {
            // we want to keep the first value, so do nothing
            Entry::Occupied(_) => {}
            Entry::Vacant(vacant) => {
                vacant.insert(value);
            }
        }
    }
}

#[cfg(feature = "indexmap_1")]
impl<K, V, S> DuplicateInsertsFirstWinsMap<K, V> for IndexMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher + Default,
{
    #[inline]
    fn new(size_hint: Option<usize>) -> Self {
        match size_hint {
            Some(size) => Self::with_capacity_and_hasher(size, S::default()),
            None => Self::with_hasher(S::default()),
        }
    }

    #[inline]
    fn insert(&mut self, key: K, value: V) {
        use indexmap_1::map::Entry;

        match self.entry(key) {
            // we want to keep the first value, so do nothing
            Entry::Occupied(_) => {}
            Entry::Vacant(vacant) => {
                vacant.insert(value);
            }
        }
    }
}

impl<K, V> DuplicateInsertsFirstWinsMap<K, V> for BTreeMap<K, V>
where
    K: Ord,
{
    #[inline]
    fn new(_size_hint: Option<usize>) -> Self {
        Self::new()
    }

    #[inline]
    fn insert(&mut self, key: K, value: V) {
        use alloc::collections::btree_map::Entry;

        match self.entry(key) {
            // we want to keep the first value, so do nothing
            Entry::Occupied(_) => {}
            Entry::Vacant(vacant) => {
                vacant.insert(value);
            }
        }
    }
}
