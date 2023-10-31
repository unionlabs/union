//! `ScopedHashMap`
//!
//! This module defines a struct `ScopedHashMap<K, V>` which defines a `FxHashMap`-like
//! container that has a concept of scopes that can be entered and exited, such that
//! values inserted while inside a scope aren't visible outside the scope.

use crate::fx::FxHashMap;
use core::hash::Hash;
use core::mem;

#[cfg(not(feature = "std"))]
use crate::fx::FxHasher;
#[cfg(not(feature = "std"))]
type Hasher = core::hash::BuildHasherDefault<FxHasher>;

struct Val<K, V> {
    value: V,
    next_key: Option<K>,
    depth: usize,
}

/// A view into an occupied entry in a `ScopedHashMap`. It is part of the `Entry` enum.
pub struct OccupiedEntry<'a, K: 'a, V: 'a> {
    #[cfg(feature = "std")]
    entry: super::hash_map::OccupiedEntry<'a, K, Val<K, V>>,
    #[cfg(not(feature = "std"))]
    entry: super::hash_map::OccupiedEntry<'a, K, Val<K, V>, Hasher>,
}

impl<'a, K, V> OccupiedEntry<'a, K, V> {
    /// Gets a reference to the value in the entry.
    pub fn get(&self) -> &V {
        &self.entry.get().value
    }
}

/// A view into a vacant entry in a `ScopedHashMap`. It is part of the `Entry` enum.
pub struct VacantEntry<'a, K: 'a, V: 'a> {
    #[cfg(feature = "std")]
    entry: super::hash_map::VacantEntry<'a, K, Val<K, V>>,
    #[cfg(not(feature = "std"))]
    entry: super::hash_map::VacantEntry<'a, K, Val<K, V>, Hasher>,
    next_key: Option<K>,
    depth: usize,
}

impl<'a, K: Hash, V> VacantEntry<'a, K, V> {
    /// Sets the value of the entry with the `VacantEntry`'s key.
    pub fn insert(self, value: V) {
        self.entry.insert(Val {
            value,
            next_key: self.next_key,
            depth: self.depth,
        });
    }
}

/// A view into a single entry in a map, which may either be vacant or occupied.
///
/// This enum is constructed from the `entry` method on `ScopedHashMap`.
pub enum Entry<'a, K: 'a, V: 'a> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>),
}

/// A wrapper around a `FxHashMap` which adds the concept of scopes. Items inserted
/// within a scope are removed when the scope is exited.
///
/// Shadowing, where one scope has entries with the same keys as a containing scope,
/// is not supported in this implementation.
pub struct ScopedHashMap<K, V> {
    map: FxHashMap<K, Val<K, V>>,
    last_insert: Option<K>,
    current_depth: usize,
}

impl<K, V> ScopedHashMap<K, V>
where
    K: PartialEq + Eq + Hash + Clone,
{
    /// Creates an empty `ScopedHashMap`.
    pub fn new() -> Self {
        Self {
            map: FxHashMap(),
            last_insert: None,
            current_depth: 0,
        }
    }

    /// Similar to `FxHashMap::entry`, gets the given key's corresponding entry in the map for
    /// in-place manipulation.
    pub fn entry(&mut self, key: K) -> Entry<K, V> {
        use super::hash_map::Entry::*;
        match self.map.entry(key) {
            Occupied(entry) => Entry::Occupied(OccupiedEntry { entry }),
            Vacant(entry) => {
                let clone_key = entry.key().clone();
                Entry::Vacant(VacantEntry {
                    entry,
                    next_key: mem::replace(&mut self.last_insert, Some(clone_key)),
                    depth: self.current_depth,
                })
            }
        }
    }

    /// Enter a new scope.
    pub fn increment_depth(&mut self) {
        // Increment the depth.
        self.current_depth = self.current_depth.checked_add(1).unwrap();
    }

    /// Exit the current scope.
    pub fn decrement_depth(&mut self) {
        // Remove all elements inserted at the current depth.
        while let Some(key) = self.last_insert.clone() {
            use crate::hash_map::Entry::*;
            match self.map.entry(key) {
                Occupied(entry) => {
                    if entry.get().depth != self.current_depth {
                        break;
                    }
                    self.last_insert = entry.remove_entry().1.next_key;
                }
                Vacant(_) => panic!(),
            }
        }

        // Decrement the depth.
        self.current_depth = self.current_depth.checked_sub(1).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let mut map: ScopedHashMap<i32, i32> = ScopedHashMap::new();

        match map.entry(0) {
            Entry::Occupied(_entry) => panic!(),
            Entry::Vacant(entry) => entry.insert(1),
        }
        match map.entry(2) {
            Entry::Occupied(_entry) => panic!(),
            Entry::Vacant(entry) => entry.insert(8),
        }
        match map.entry(2) {
            Entry::Occupied(entry) => assert!(*entry.get() == 8),
            Entry::Vacant(_entry) => panic!(),
        }
        map.increment_depth();
        match map.entry(2) {
            Entry::Occupied(entry) => assert!(*entry.get() == 8),
            Entry::Vacant(_entry) => panic!(),
        }
        match map.entry(1) {
            Entry::Occupied(_entry) => panic!(),
            Entry::Vacant(entry) => entry.insert(3),
        }
        match map.entry(1) {
            Entry::Occupied(entry) => assert!(*entry.get() == 3),
            Entry::Vacant(_entry) => panic!(),
        }
        match map.entry(0) {
            Entry::Occupied(entry) => assert!(*entry.get() == 1),
            Entry::Vacant(_entry) => panic!(),
        }
        match map.entry(2) {
            Entry::Occupied(entry) => assert!(*entry.get() == 8),
            Entry::Vacant(_entry) => panic!(),
        }
        map.decrement_depth();
        match map.entry(0) {
            Entry::Occupied(entry) => assert!(*entry.get() == 1),
            Entry::Vacant(_entry) => panic!(),
        }
        match map.entry(2) {
            Entry::Occupied(entry) => assert!(*entry.get() == 8),
            Entry::Vacant(_entry) => panic!(),
        }
        map.increment_depth();
        match map.entry(2) {
            Entry::Occupied(entry) => assert!(*entry.get() == 8),
            Entry::Vacant(_entry) => panic!(),
        }
        match map.entry(1) {
            Entry::Occupied(_entry) => panic!(),
            Entry::Vacant(entry) => entry.insert(4),
        }
        match map.entry(1) {
            Entry::Occupied(entry) => assert!(*entry.get() == 4),
            Entry::Vacant(_entry) => panic!(),
        }
        match map.entry(2) {
            Entry::Occupied(entry) => assert!(*entry.get() == 8),
            Entry::Vacant(_entry) => panic!(),
        }
        map.decrement_depth();
        map.increment_depth();
        map.increment_depth();
        map.increment_depth();
        match map.entry(2) {
            Entry::Occupied(entry) => assert!(*entry.get() == 8),
            Entry::Vacant(_entry) => panic!(),
        }
        match map.entry(1) {
            Entry::Occupied(_entry) => panic!(),
            Entry::Vacant(entry) => entry.insert(5),
        }
        match map.entry(1) {
            Entry::Occupied(entry) => assert!(*entry.get() == 5),
            Entry::Vacant(_entry) => panic!(),
        }
        match map.entry(2) {
            Entry::Occupied(entry) => assert!(*entry.get() == 8),
            Entry::Vacant(_entry) => panic!(),
        }
        map.decrement_depth();
        map.decrement_depth();
        map.decrement_depth();
        match map.entry(2) {
            Entry::Occupied(entry) => assert!(*entry.get() == 8),
            Entry::Vacant(_entry) => panic!(),
        }
        match map.entry(1) {
            Entry::Occupied(_entry) => panic!(),
            Entry::Vacant(entry) => entry.insert(3),
        }
    }
}
