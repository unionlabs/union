use crate::prelude::*;
#[cfg(feature = "indexmap_1")]
use indexmap_1::{IndexMap, IndexSet};

pub trait PreventDuplicateInsertsSet<T> {
    fn new(size_hint: Option<usize>) -> Self;

    /// Return true if the insert was successful and the value did not exist in the set
    fn insert(&mut self, value: T) -> bool;
}

pub trait PreventDuplicateInsertsMap<K, V> {
    fn new(size_hint: Option<usize>) -> Self;

    /// Return true if the insert was successful and the key did not exist in the map
    fn insert(&mut self, key: K, value: V) -> bool;
}

#[cfg(feature = "std")]
impl<T, S> PreventDuplicateInsertsSet<T> for HashSet<T, S>
where
    T: Eq + Hash,
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
    fn insert(&mut self, value: T) -> bool {
        self.insert(value)
    }
}

#[cfg(feature = "indexmap_1")]
impl<T, S> PreventDuplicateInsertsSet<T> for IndexSet<T, S>
where
    T: Eq + Hash,
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
    fn insert(&mut self, value: T) -> bool {
        self.insert(value)
    }
}

impl<T> PreventDuplicateInsertsSet<T> for BTreeSet<T>
where
    T: Ord,
{
    #[inline]
    fn new(_size_hint: Option<usize>) -> Self {
        Self::new()
    }

    #[inline]
    fn insert(&mut self, value: T) -> bool {
        self.insert(value)
    }
}

#[cfg(feature = "std")]
impl<K, V, S> PreventDuplicateInsertsMap<K, V> for HashMap<K, V, S>
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
    fn insert(&mut self, key: K, value: V) -> bool {
        self.insert(key, value).is_none()
    }
}

#[cfg(feature = "indexmap_1")]
impl<K, V, S> PreventDuplicateInsertsMap<K, V> for IndexMap<K, V, S>
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
    fn insert(&mut self, key: K, value: V) -> bool {
        self.insert(key, value).is_none()
    }
}

impl<K, V> PreventDuplicateInsertsMap<K, V> for BTreeMap<K, V>
where
    K: Ord,
{
    #[inline]
    fn new(_size_hint: Option<usize>) -> Self {
        Self::new()
    }

    #[inline]
    fn insert(&mut self, key: K, value: V) -> bool {
        self.insert(key, value).is_none()
    }
}
