//! `ScopedHashMap`
//!
//! This module defines a struct `ScopedHashMap<K, V>` which defines a `FxHashMap`-like
//! container that has a concept of scopes that can be entered and exited, such that
//! values inserted while inside a scope aren't visible outside the scope.

use crate::fx::FxHashMap;
use core::hash::Hash;
use smallvec::{smallvec, SmallVec};

#[cfg(not(feature = "std"))]
use crate::fx::FxHasher;
#[cfg(not(feature = "std"))]
type Hasher = core::hash::BuildHasherDefault<FxHasher>;

struct Val<V> {
    value: V,
    level: u32,
    generation: u32,
}

/// A view into an occupied entry in a `ScopedHashMap`. It is part of the `Entry` enum.
pub struct OccupiedEntry<'a, K: 'a, V: 'a> {
    entry: super::hash_map::OccupiedEntry<'a, K, Val<V>>,
}

impl<'a, K, V> OccupiedEntry<'a, K, V> {
    /// Gets a reference to the value in the entry.
    pub fn get(&self) -> &V {
        &self.entry.get().value
    }
}

/// A view into a vacant entry in a `ScopedHashMap`. It is part of the `Entry` enum.
pub struct VacantEntry<'a, K: 'a, V: 'a> {
    entry: InsertLoc<'a, K, V>,
    depth: u32,
    generation: u32,
}

/// Where to insert from a `VacantEntry`. May be vacant or occupied in
/// the underlying map because of lazy (generation-based) deletion.
enum InsertLoc<'a, K: 'a, V: 'a> {
    Vacant(super::hash_map::VacantEntry<'a, K, Val<V>>),
    Occupied(super::hash_map::OccupiedEntry<'a, K, Val<V>>),
}

impl<'a, K, V> VacantEntry<'a, K, V> {
    /// Sets the value of the entry with the `VacantEntry`'s key.
    pub fn insert(self, value: V) {
        let val = Val {
            value,
            level: self.depth,
            generation: self.generation,
        };
        match self.entry {
            InsertLoc::Vacant(v) => {
                v.insert(val);
            }
            InsertLoc::Occupied(mut o) => {
                o.insert(val);
            }
        }
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
    map: FxHashMap<K, Val<V>>,
    generation_by_depth: SmallVec<[u32; 8]>,
    generation: u32,
}

impl<K, V> ScopedHashMap<K, V>
where
    K: PartialEq + Eq + Hash + Clone,
{
    /// Creates an empty `ScopedHashMap`.
    pub fn new() -> Self {
        Self {
            map: FxHashMap(),
            generation: 0,
            generation_by_depth: smallvec![0],
        }
    }

    /// Creates an empty `ScopedHashMap` with some pre-allocated capacity.
    pub fn with_capacity(cap: usize) -> Self {
        let mut map = FxHashMap::default();
        map.reserve(cap);
        Self {
            map,
            generation: 0,
            generation_by_depth: smallvec![0],
        }
    }

    /// Similar to `FxHashMap::entry`, gets the given key's corresponding entry in the map for
    /// in-place manipulation.
    pub fn entry<'a>(&'a mut self, key: K) -> Entry<'a, K, V> {
        self.entry_with_depth(key, self.depth())
    }

    /// Get the entry, setting the scope depth at which to insert.
    pub fn entry_with_depth<'a>(&'a mut self, key: K, depth: usize) -> Entry<'a, K, V> {
        debug_assert!(depth <= self.generation_by_depth.len());
        let generation = self.generation_by_depth[depth];
        let depth = depth as u32;
        use super::hash_map::Entry::*;
        match self.map.entry(key) {
            Occupied(entry) => {
                let entry_generation = entry.get().generation;
                let entry_depth = entry.get().level as usize;
                if self.generation_by_depth.get(entry_depth).cloned() == Some(entry_generation) {
                    Entry::Occupied(OccupiedEntry { entry })
                } else {
                    Entry::Vacant(VacantEntry {
                        entry: InsertLoc::Occupied(entry),
                        depth,
                        generation,
                    })
                }
            }
            Vacant(entry) => Entry::Vacant(VacantEntry {
                entry: InsertLoc::Vacant(entry),
                depth,
                generation,
            }),
        }
    }

    /// Get a value from a key, if present.
    pub fn get<'a>(&'a self, key: &K) -> Option<&'a V> {
        self.map
            .get(key)
            .filter(|entry| {
                let level = entry.level as usize;
                self.generation_by_depth.get(level).cloned() == Some(entry.generation)
            })
            .map(|entry| &entry.value)
    }

    /// Insert a key-value pair if absent. No-op if already exists.
    pub fn insert_if_absent(&mut self, key: K, value: V) {
        self.insert_if_absent_with_depth(key, value, self.depth());
    }

    /// Insert a key-value pair if absent, using the given depth for
    /// the insertion. No-op if already exists.
    pub fn insert_if_absent_with_depth(&mut self, key: K, value: V, depth: usize) {
        match self.entry_with_depth(key, depth) {
            Entry::Vacant(v) => {
                v.insert(value);
            }
            Entry::Occupied(_) => {
                // Nothing.
            }
        }
    }

    /// Enter a new scope.
    pub fn increment_depth(&mut self) {
        self.generation_by_depth.push(self.generation);
    }

    /// Exit the current scope.
    pub fn decrement_depth(&mut self) {
        self.generation += 1;
        self.generation_by_depth.pop();
    }

    /// Return the current scope depth.
    pub fn depth(&self) -> usize {
        self.generation_by_depth
            .len()
            .checked_sub(1)
            .expect("generation_by_depth cannot be empty")
    }

    /// Remote an entry.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.map.remove(key).and_then(|val| {
            let entry_generation = val.generation;
            let entry_depth = val.level as usize;
            if self.generation_by_depth.get(entry_depth).cloned() == Some(entry_generation) {
                Some(val.value)
            } else {
                None
            }
        })
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

    #[test]
    fn insert_arbitrary_depth() {
        let mut map: ScopedHashMap<i32, i32> = ScopedHashMap::new();
        map.insert_if_absent(1, 2);
        assert_eq!(map.get(&1), Some(&2));
        map.increment_depth();
        assert_eq!(map.get(&1), Some(&2));
        map.insert_if_absent(3, 4);
        assert_eq!(map.get(&3), Some(&4));
        map.decrement_depth();
        assert_eq!(map.get(&3), None);
        map.increment_depth();
        map.insert_if_absent_with_depth(3, 4, 0);
        assert_eq!(map.get(&3), Some(&4));
        map.decrement_depth();
        assert_eq!(map.get(&3), Some(&4));
    }
}
