// Copyright 2012-2018 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![deny(missing_docs)]

//! A simple map based on a vector for small integer keys. Space requirements
//! are O(highest integer key).

// optional serde support
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

use self::Entry::*;

use std::cmp::{Ordering, max};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::iter::{Enumerate, FilterMap, FromIterator};
use std::mem::{replace, swap};
use std::ops::{Index, IndexMut};
use std::slice;
use std::vec;

/// A map optimized for small integer keys.
///
/// # Examples
///
/// ```
/// use vec_map::VecMap;
///
/// let mut months = VecMap::new();
/// months.insert(1, "Jan");
/// months.insert(2, "Feb");
/// months.insert(3, "Mar");
///
/// if !months.contains_key(12) {
///     println!("The end is near!");
/// }
///
/// assert_eq!(months.get(1), Some(&"Jan"));
///
/// if let Some(value) = months.get_mut(3) {
///     *value = "Venus";
/// }
///
/// assert_eq!(months.get(3), Some(&"Venus"));
///
/// // Print out all months
/// for (key, value) in &months {
///     println!("month {} is {}", key, value);
/// }
///
/// months.clear();
/// assert!(months.is_empty());
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VecMap<V> {
    n: usize,
    v: Vec<Option<V>>,
}

/// A view into a single entry in a map, which may either be vacant or occupied.
pub enum Entry<'a, V: 'a> {
    /// A vacant Entry
    Vacant(VacantEntry<'a, V>),

    /// An occupied Entry
    Occupied(OccupiedEntry<'a, V>),
}

/// A vacant Entry.
pub struct VacantEntry<'a, V: 'a> {
    map: &'a mut VecMap<V>,
    index: usize,
}

/// An occupied Entry.
pub struct OccupiedEntry<'a, V: 'a> {
    map: &'a mut VecMap<V>,
    index: usize,
}

impl<V> Default for VecMap<V> {
    #[inline]
    fn default() -> Self { Self::new() }
}

impl<V: Hash> Hash for VecMap<V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // In order to not traverse the `VecMap` twice, count the elements
        // during iteration.
        let mut count: usize = 0;
        for elt in self {
            elt.hash(state);
            count += 1;
        }
        count.hash(state);
    }
}

impl<V> VecMap<V> {
    /// Creates an empty `VecMap`.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_map::VecMap;
    /// let mut map: VecMap<&str> = VecMap::new();
    /// ```
    pub fn new() -> Self { VecMap { n: 0, v: vec![] } }

    /// Creates an empty `VecMap` with space for at least `capacity`
    /// elements before resizing.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_map::VecMap;
    /// let mut map: VecMap<&str> = VecMap::with_capacity(10);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        VecMap { n: 0, v: Vec::with_capacity(capacity) }
    }

    /// Returns the number of elements the `VecMap` can hold without
    /// reallocating.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_map::VecMap;
    /// let map: VecMap<String> = VecMap::with_capacity(10);
    /// assert!(map.capacity() >= 10);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.v.capacity()
    }

    /// Reserves capacity for the given `VecMap` to contain `len` distinct keys.
    /// In the case of `VecMap` this means reallocations will not occur as long
    /// as all inserted keys are less than `len`.
    ///
    /// The collection may reserve more space to avoid frequent reallocations.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_map::VecMap;
    /// let mut map: VecMap<&str> = VecMap::new();
    /// map.reserve_len(10);
    /// assert!(map.capacity() >= 10);
    /// ```
    pub fn reserve_len(&mut self, len: usize) {
        let cur_len = self.v.len();
        if len >= cur_len {
            self.v.reserve(len - cur_len);
        }
    }

    /// Reserves the minimum capacity for the given `VecMap` to contain `len` distinct keys.
    /// In the case of `VecMap` this means reallocations will not occur as long as all inserted
    /// keys are less than `len`.
    ///
    /// Note that the allocator may give the collection more space than it requests.
    /// Therefore capacity cannot be relied upon to be precisely minimal.  Prefer
    /// `reserve_len` if future insertions are expected.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_map::VecMap;
    /// let mut map: VecMap<&str> = VecMap::new();
    /// map.reserve_len_exact(10);
    /// assert!(map.capacity() >= 10);
    /// ```
    pub fn reserve_len_exact(&mut self, len: usize) {
        let cur_len = self.v.len();
        if len >= cur_len {
            self.v.reserve_exact(len - cur_len);
        }
    }

    /// Trims the `VecMap` of any excess capacity.
    ///
    /// The collection may reserve more space to avoid frequent reallocations.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_map::VecMap;
    /// let mut map: VecMap<&str> = VecMap::with_capacity(10);
    /// map.shrink_to_fit();
    /// assert_eq!(map.capacity(), 0);
    /// ```
    pub fn shrink_to_fit(&mut self) {
        // strip off trailing `None`s
        if let Some(idx) = self.v.iter().rposition(Option::is_some) {
            self.v.truncate(idx + 1);
        } else {
            self.v.clear();
        }

        self.v.shrink_to_fit()
    }

    /// Returns an iterator visiting all keys in ascending order of the keys.
    /// The iterator's element type is `usize`.
    pub fn keys(&self) -> Keys<V> {
        Keys { iter: self.iter() }
    }

    /// Returns an iterator visiting all values in ascending order of the keys.
    /// The iterator's element type is `&'r V`.
    pub fn values(&self) -> Values<V> {
        Values { iter: self.iter() }
    }

    /// Returns an iterator visiting all values in ascending order of the keys.
    /// The iterator's element type is `&'r mut V`.
    pub fn values_mut(&mut self) -> ValuesMut<V> {
        ValuesMut { iter_mut: self.iter_mut() }
    }

    /// Returns an iterator visiting all key-value pairs in ascending order of the keys.
    /// The iterator's element type is `(usize, &'r V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_map::VecMap;
    ///
    /// let mut map = VecMap::new();
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    /// map.insert(2, "b");
    ///
    /// // Print `1: a` then `2: b` then `3: c`
    /// for (key, value) in map.iter() {
    ///     println!("{}: {}", key, value);
    /// }
    /// ```
    pub fn iter(&self) -> Iter<V> {
        Iter {
            front: 0,
            back: self.v.len(),
            n: self.n,
            yielded: 0,
            iter: self.v.iter()
        }
    }

    /// Returns an iterator visiting all key-value pairs in ascending order of the keys,
    /// with mutable references to the values.
    /// The iterator's element type is `(usize, &'r mut V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_map::VecMap;
    ///
    /// let mut map = VecMap::new();
    /// map.insert(1, "a");
    /// map.insert(2, "b");
    /// map.insert(3, "c");
    ///
    /// for (key, value) in map.iter_mut() {
    ///     *value = "x";
    /// }
    ///
    /// for (key, value) in &map {
    ///     assert_eq!(value, &"x");
    /// }
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<V> {
        IterMut {
            front: 0,
            back: self.v.len(),
            n: self.n,
            yielded: 0,
            iter: self.v.iter_mut()
        }
    }

    /// Moves all elements from `other` into the map while overwriting existing keys.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_map::VecMap;
    ///
    /// let mut a = VecMap::new();
    /// a.insert(1, "a");
    /// a.insert(2, "b");
    ///
    /// let mut b = VecMap::new();
    /// b.insert(3, "c");
    /// b.insert(4, "d");
    ///
    /// a.append(&mut b);
    ///
    /// assert_eq!(a.len(), 4);
    /// assert_eq!(b.len(), 0);
    /// assert_eq!(a[1], "a");
    /// assert_eq!(a[2], "b");
    /// assert_eq!(a[3], "c");
    /// assert_eq!(a[4], "d");
    /// ```
    pub fn append(&mut self, other: &mut Self) {
        self.extend(other.drain());
    }

    /// Splits the collection into two at the given key.
    ///
    /// Returns a newly allocated `Self`. `self` contains elements `[0, at)`,
    /// and the returned `Self` contains elements `[at, max_key)`.
    ///
    /// Note that the capacity of `self` does not change.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_map::VecMap;
    ///
    /// let mut a = VecMap::new();
    /// a.insert(1, "a");
    /// a.insert(2, "b");
    /// a.insert(3, "c");
    /// a.insert(4, "d");
    ///
    /// let b = a.split_off(3);
    ///
    /// assert_eq!(a[1], "a");
    /// assert_eq!(a[2], "b");
    ///
    /// assert_eq!(b[3], "c");
    /// assert_eq!(b[4], "d");
    /// ```
    pub fn split_off(&mut self, at: usize) -> Self {
        let mut other = VecMap::new();

        if at == 0 {
            // Move all elements to other
            // The swap will also fix .n
            swap(self, &mut other);
            return other
        } else if at >= self.v.len() {
            // No elements to copy
            return other;
        }

        // Look up the index of the first non-None item
        let first_index = self.v.iter().position(|el| el.is_some());
        let start_index = match first_index {
            Some(index) => max(at, index),
            None => {
                // self has no elements
                return other;
            }
        };

        // Fill the new VecMap with `None`s until `start_index`
        other.v.extend((0..start_index).map(|_| None));

        // Move elements beginning with `start_index` from `self` into `other`
        let mut taken = 0;
        other.v.extend(self.v[start_index..].iter_mut().map(|el| {
            let el = el.take();
            if el.is_some() {
                taken += 1;
            }
            el
        }));
        other.n = taken;
        self.n -= taken;

        other
    }

    /// Returns an iterator visiting all key-value pairs in ascending order of
    /// the keys, emptying (but not consuming) the original `VecMap`.
    /// The iterator's element type is `(usize, &'r V)`. Keeps the allocated memory for reuse.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_map::VecMap;
    ///
    /// let mut map = VecMap::new();
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    /// map.insert(2, "b");
    ///
    /// let vec: Vec<(usize, &str)> = map.drain().collect();
    ///
    /// assert_eq!(vec, [(1, "a"), (2, "b"), (3, "c")]);
    /// ```
    pub fn drain(&mut self) -> Drain<V> {
        fn filter<A>((i, v): (usize, Option<A>)) -> Option<(usize, A)> {
            v.map(|v| (i, v))
        }
        let filter: fn((usize, Option<V>)) -> Option<(usize, V)> = filter; // coerce to fn ptr

        self.n = 0;
        Drain { iter: self.v.drain(..).enumerate().filter_map(filter) }
    }

    /// Returns the number of elements in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_map::VecMap;
    ///
    /// let mut a = VecMap::new();
    /// assert_eq!(a.len(), 0);
    /// a.insert(1, "a");
    /// assert_eq!(a.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.n
    }

    /// Returns true if the map contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_map::VecMap;
    ///
    /// let mut a = VecMap::new();
    /// assert!(a.is_empty());
    /// a.insert(1, "a");
    /// assert!(!a.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    /// Clears the map, removing all key-value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_map::VecMap;
    ///
    /// let mut a = VecMap::new();
    /// a.insert(1, "a");
    /// a.clear();
    /// assert!(a.is_empty());
    /// ```
    pub fn clear(&mut self) { self.n = 0; self.v.clear() }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_map::VecMap;
    ///
    /// let mut map = VecMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.get(1), Some(&"a"));
    /// assert_eq!(map.get(2), None);
    /// ```
    pub fn get(&self, key: usize) -> Option<&V> {
        if key < self.v.len() {
            self.v[key].as_ref()
        } else {
            None
        }
    }

    /// Returns true if the map contains a value for the specified key.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_map::VecMap;
    ///
    /// let mut map = VecMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.contains_key(1), true);
    /// assert_eq!(map.contains_key(2), false);
    /// ```
    #[inline]
    pub fn contains_key(&self, key: usize) -> bool {
        self.get(key).is_some()
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_map::VecMap;
    ///
    /// let mut map = VecMap::new();
    /// map.insert(1, "a");
    /// if let Some(x) = map.get_mut(1) {
    ///     *x = "b";
    /// }
    /// assert_eq!(map[1], "b");
    /// ```
    pub fn get_mut(&mut self, key: usize) -> Option<&mut V> {
        if key < self.v.len() {
            self.v[key].as_mut()
        } else {
            None
        }
    }

    /// Inserts a key-value pair into the map. If the key already had a value
    /// present in the map, that value is returned. Otherwise, `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_map::VecMap;
    ///
    /// let mut map = VecMap::new();
    /// assert_eq!(map.insert(37, "a"), None);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// map.insert(37, "b");
    /// assert_eq!(map.insert(37, "c"), Some("b"));
    /// assert_eq!(map[37], "c");
    /// ```
    pub fn insert(&mut self, key: usize, value: V) -> Option<V> {
        let len = self.v.len();
        if len <= key {
            self.v.extend((0..key - len + 1).map(|_| None));
        }
        let was = replace(&mut self.v[key], Some(value));
        if was.is_none() {
            self.n += 1;
        }
        was
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_map::VecMap;
    ///
    /// let mut map = VecMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.remove(1), Some("a"));
    /// assert_eq!(map.remove(1), None);
    /// ```
    pub fn remove(&mut self, key: usize) -> Option<V> {
        if key >= self.v.len() {
            return None;
        }
        let result = &mut self.v[key];
        let was = result.take();
        if was.is_some() {
            self.n -= 1;
        }
        was
    }

    /// Gets the given key's corresponding entry in the map for in-place manipulation.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_map::VecMap;
    ///
    /// let mut count: VecMap<u32> = VecMap::new();
    ///
    /// // count the number of occurrences of numbers in the vec
    /// for x in vec![1, 2, 1, 2, 3, 4, 1, 2, 4] {
    ///     *count.entry(x).or_insert(0) += 1;
    /// }
    ///
    /// assert_eq!(count[1], 3);
    /// ```
    pub fn entry(&mut self, key: usize) -> Entry<V> {
        // FIXME(Gankro): this is basically the dumbest implementation of
        // entry possible, because weird non-lexical borrows issues make it
        // completely insane to do any other way. That said, Entry is a border-line
        // useless construct on VecMap, so it's hardly a big loss.
        if self.contains_key(key) {
            Occupied(OccupiedEntry {
                map: self,
                index: key,
            })
        } else {
            Vacant(VacantEntry {
                map: self,
                index: key,
            })
        }
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all pairs `(k, v)` such that `f(&k, &mut v)` returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_map::VecMap;
    ///
    /// let mut map: VecMap<usize> = (0..8).map(|x|(x, x*10)).collect();
    /// map.retain(|k, _| k % 2 == 0);
    /// assert_eq!(map.len(), 4);
    /// ```
    pub fn retain<F>(&mut self, mut f: F)
        where F: FnMut(usize, &mut V) -> bool
    {
        for (i, e) in self.v.iter_mut().enumerate() {
            let remove = match *e {
                Some(ref mut value) => !f(i, value),
                None => false,
            };
            if remove {
                *e = None;
                self.n -= 1;
            }
        }
    }
}

impl<'a, V> Entry<'a, V> {
    /// Ensures a value is in the entry by inserting the default if empty, and
    /// returns a mutable reference to the value in the entry.
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.insert(default),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default
    /// function if empty, and returns a mutable reference to the value in the
    /// entry.
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.insert(default()),
        }
    }
}

impl<'a, V> VacantEntry<'a, V> {
    /// Sets the value of the entry with the VacantEntry's key,
    /// and returns a mutable reference to it.
    pub fn insert(self, value: V) -> &'a mut V {
        let index = self.index;
        self.map.insert(index, value);
        &mut self.map[index]
    }
}

impl<'a, V> OccupiedEntry<'a, V> {
    /// Gets a reference to the value in the entry.
    pub fn get(&self) -> &V {
        let index = self.index;
        &self.map[index]
    }

    /// Gets a mutable reference to the value in the entry.
    pub fn get_mut(&mut self) -> &mut V {
        let index = self.index;
        &mut self.map[index]
    }

    /// Converts the entry into a mutable reference to its value.
    pub fn into_mut(self) -> &'a mut V {
        let index = self.index;
        &mut self.map[index]
    }

    /// Sets the value of the entry with the OccupiedEntry's key,
    /// and returns the entry's old value.
    pub fn insert(&mut self, value: V) -> V {
        let index = self.index;
        self.map.insert(index, value).unwrap()
    }

    /// Takes the value of the entry out of the map, and returns it.
    pub fn remove(self) -> V {
        let index = self.index;
        self.map.remove(index).unwrap()
    }
}

impl<V: Clone> Clone for VecMap<V> {
    #[inline]
    fn clone(&self) -> Self {
        VecMap { n: self.n, v: self.v.clone() }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.v.clone_from(&source.v);
        self.n = source.n;
    }
}

impl<V: PartialEq> PartialEq for VecMap<V> {
    fn eq(&self, other: &Self) -> bool {
        self.n == other.n && self.iter().eq(other.iter())
    }
}

impl<V: Eq> Eq for VecMap<V> {}

impl<V: PartialOrd> PartialOrd for VecMap<V> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<V: Ord> Ord for VecMap<V> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<V: fmt::Debug> fmt::Debug for VecMap<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map().entries(self).finish()
    }
}

impl<V> FromIterator<(usize, V)> for VecMap<V> {
    fn from_iter<I: IntoIterator<Item = (usize, V)>>(iter: I) -> Self {
        let mut map = Self::new();
        map.extend(iter);
        map
    }
}

impl<T> IntoIterator for VecMap<T> {
    type Item = (usize, T);
    type IntoIter = IntoIter<T>;

    /// Returns an iterator visiting all key-value pairs in ascending order of
    /// the keys, consuming the original `VecMap`.
    /// The iterator's element type is `(usize, &'r V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use vec_map::VecMap;
    ///
    /// let mut map = VecMap::new();
    /// map.insert(1, "a");
    /// map.insert(3, "c");
    /// map.insert(2, "b");
    ///
    /// let vec: Vec<(usize, &str)> = map.into_iter().collect();
    ///
    /// assert_eq!(vec, [(1, "a"), (2, "b"), (3, "c")]);
    /// ```
    fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            n: self.n,
            yielded: 0,
            iter: self.v.into_iter().enumerate()
        }
    }
}

impl<'a, T> IntoIterator for &'a VecMap<T> {
    type Item = (usize, &'a T);
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut VecMap<T> {
    type Item = (usize, &'a mut T);
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> IterMut<'a, T> {
        self.iter_mut()
    }
}

impl<V> Extend<(usize, V)> for VecMap<V> {
    fn extend<I: IntoIterator<Item = (usize, V)>>(&mut self, iter: I) {
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

impl<'a, V: Copy> Extend<(usize, &'a V)> for VecMap<V> {
    fn extend<I: IntoIterator<Item = (usize, &'a V)>>(&mut self, iter: I) {
        self.extend(iter.into_iter().map(|(key, &value)| (key, value)));
    }
}

impl<V> Index<usize> for VecMap<V> {
    type Output = V;

    #[inline]
    fn index(&self, i: usize) -> &V {
        self.get(i).expect("key not present")
    }
}

impl<'a, V> Index<&'a usize> for VecMap<V> {
    type Output = V;

    #[inline]
    fn index(&self, i: &usize) -> &V {
        self.get(*i).expect("key not present")
    }
}

impl<V> IndexMut<usize> for VecMap<V> {
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut V {
        self.get_mut(i).expect("key not present")
    }
}

impl<'a, V> IndexMut<&'a usize> for VecMap<V> {
    #[inline]
    fn index_mut(&mut self, i: &usize) -> &mut V {
        self.get_mut(*i).expect("key not present")
    }
}

macro_rules! iterator {
    (impl $name:ident -> $elem:ty, $($getter:ident),+) => {
        impl<'a, V> Iterator for $name<'a, V> {
            type Item = $elem;

            #[inline]
            fn next(&mut self) -> Option<$elem> {
                while self.front < self.back {
                    if let Some(elem) = self.iter.next() {
                        if let Some(x) = elem$(. $getter ())+ {
                            let index = self.front;
                            self.front += 1;
                            self.yielded += 1;
                            return Some((index, x));
                        }
                    }
                    self.front += 1;
                }
                None
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                (self.n - self.yielded, Some(self.n - self.yielded))
            }
        }
    }
}

macro_rules! double_ended_iterator {
    (impl $name:ident -> $elem:ty, $($getter:ident),+) => {
        impl<'a, V> DoubleEndedIterator for $name<'a, V> {
            #[inline]
            fn next_back(&mut self) -> Option<$elem> {
                while self.front < self.back {
                    if let Some(elem) = self.iter.next_back() {
                        if let Some(x) = elem$(. $getter ())+ {
                            self.back -= 1;
                            return Some((self.back, x));
                        }
                    }
                    self.back -= 1;
                }
                None
            }
        }
    }
}

/// An iterator over the key-value pairs of a map.
pub struct Iter<'a, V: 'a> {
    front: usize,
    back: usize,
    n: usize,
    yielded: usize,
    iter: slice::Iter<'a, Option<V>>
}

// FIXME(#19839) Remove in favor of `#[derive(Clone)]`
impl<'a, V> Clone for Iter<'a, V> {
    fn clone(&self) -> Iter<'a, V> {
        Iter {
            front: self.front,
            back: self.back,
            n: self.n,
            yielded: self.yielded,
            iter: self.iter.clone()
        }
    }
}

iterator! { impl Iter -> (usize, &'a V), as_ref }
impl<'a, V> ExactSizeIterator for Iter<'a, V> {}
double_ended_iterator! { impl Iter -> (usize, &'a V), as_ref }

/// An iterator over the key-value pairs of a map, with the
/// values being mutable.
pub struct IterMut<'a, V: 'a> {
    front: usize,
    back: usize,
    n: usize,
    yielded: usize,
    iter: slice::IterMut<'a, Option<V>>
}

iterator! { impl IterMut -> (usize, &'a mut V), as_mut }
impl<'a, V> ExactSizeIterator for IterMut<'a, V> {}
double_ended_iterator! { impl IterMut -> (usize, &'a mut V), as_mut }

/// An iterator over the keys of a map.
pub struct Keys<'a, V: 'a> {
    iter: Iter<'a, V>,
}

// FIXME(#19839) Remove in favor of `#[derive(Clone)]`
impl<'a, V> Clone for Keys<'a, V> {
    fn clone(&self) -> Keys<'a, V> {
        Keys {
            iter: self.iter.clone()
        }
    }
}

/// An iterator over the values of a map.
pub struct Values<'a, V: 'a> {
    iter: Iter<'a, V>,
}

// FIXME(#19839) Remove in favor of `#[derive(Clone)]`
impl<'a, V> Clone for Values<'a, V> {
    fn clone(&self) -> Values<'a, V> {
        Values {
            iter: self.iter.clone()
        }
    }
}

/// An iterator over the values of a map.
pub struct ValuesMut<'a, V: 'a> {
    iter_mut: IterMut<'a, V>,
}

/// A consuming iterator over the key-value pairs of a map.
pub struct IntoIter<V> {
    n: usize,
    yielded: usize,
    iter: Enumerate<vec::IntoIter<Option<V>>>,
}

/// A draining iterator over the key-value pairs of a map.
pub struct Drain<'a, V: 'a> {
    iter: FilterMap<
    Enumerate<vec::Drain<'a, Option<V>>>,
    fn((usize, Option<V>)) -> Option<(usize, V)>>
}

impl<'a, V> Iterator for Drain<'a, V> {
    type Item = (usize, V);

    fn next(&mut self) -> Option<(usize, V)> { self.iter.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

impl<'a, V> ExactSizeIterator for Drain<'a, V> {}

impl<'a, V> DoubleEndedIterator for Drain<'a, V> {
    fn next_back(&mut self) -> Option<(usize, V)> { self.iter.next_back() }
}

impl<'a, V> Iterator for Keys<'a, V> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> { self.iter.next().map(|e| e.0) }
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

impl<'a, V> ExactSizeIterator for Keys<'a, V> {}

impl<'a, V> DoubleEndedIterator for Keys<'a, V> {
    fn next_back(&mut self) -> Option<usize> { self.iter.next_back().map(|e| e.0) }
}

impl<'a, V> Iterator for Values<'a, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<(&'a V)> { self.iter.next().map(|e| e.1) }
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

impl<'a, V> ExactSizeIterator for Values<'a, V> {}

impl<'a, V> DoubleEndedIterator for Values<'a, V> {
    fn next_back(&mut self) -> Option<(&'a V)> { self.iter.next_back().map(|e| e.1) }
}

impl<'a, V> Iterator for ValuesMut<'a, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<(&'a mut V)> { self.iter_mut.next().map(|e| e.1) }
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter_mut.size_hint() }
}

impl<'a, V> ExactSizeIterator for ValuesMut<'a, V> {}

impl<'a, V> DoubleEndedIterator for ValuesMut<'a, V> {
    fn next_back(&mut self) -> Option<&'a mut V> { self.iter_mut.next_back().map(|e| e.1) }
}

impl<V> Iterator for IntoIter<V> {
    type Item = (usize, V);

    fn next(&mut self) -> Option<(usize, V)> {
        loop {
            match self.iter.next() {
                None => return None,
                Some((i, Some(value))) => {
                    self.yielded += 1;
                    return Some((i, value))
                },
                _ => {}
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.n - self.yielded, Some(self.n - self.yielded))
    }
}

impl<V> ExactSizeIterator for IntoIter<V> {}

impl<V> DoubleEndedIterator for IntoIter<V> {
    fn next_back(&mut self) -> Option<(usize, V)> {
        loop {
            match self.iter.next_back() {
                None => return None,
                Some((i, Some(value))) => return Some((i, value)),
                _ => {}
            }
        }
    }
}

#[allow(dead_code)]
fn assert_properties() {
    fn vec_map_covariant<'a, T>(map: VecMap<&'static T>) -> VecMap<&'a T> { map }

    fn into_iter_covariant<'a, T>(iter: IntoIter<&'static T>) -> IntoIter<&'a T> { iter }

    fn iter_covariant<'i, 'a, T>(iter: Iter<'i, &'static T>) -> Iter<'i, &'a T> { iter }

    fn keys_covariant<'i, 'a, T>(iter: Keys<'i, &'static T>) -> Keys<'i, &'a T> { iter }

    fn values_covariant<'i, 'a, T>(iter: Values<'i, &'static T>) -> Values<'i, &'a T> { iter }
}

#[cfg(test)]
mod test {
    use super::VecMap;
    use super::Entry::{Occupied, Vacant};
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn test_get_mut() {
        let mut m = VecMap::new();
        assert!(m.insert(1, 12).is_none());
        assert!(m.insert(2, 8).is_none());
        assert!(m.insert(5, 14).is_none());
        let new = 100;
        match m.get_mut(5) {
            None => panic!(), Some(x) => *x = new
        }
        assert_eq!(m.get(5), Some(&new));
    }

    #[test]
    fn test_len() {
        let mut map = VecMap::new();
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
        assert!(map.insert(5, 20).is_none());
        assert_eq!(map.len(), 1);
        assert!(!map.is_empty());
        assert!(map.insert(11, 12).is_none());
        assert_eq!(map.len(), 2);
        assert!(!map.is_empty());
        assert!(map.insert(14, 22).is_none());
        assert_eq!(map.len(), 3);
        assert!(!map.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut map = VecMap::new();
        assert!(map.insert(5, 20).is_none());
        assert!(map.insert(11, 12).is_none());
        assert!(map.insert(14, 22).is_none());
        map.clear();
        assert!(map.is_empty());
        assert!(map.get(5).is_none());
        assert!(map.get(11).is_none());
        assert!(map.get(14).is_none());
    }

    #[test]
    fn test_insert() {
        let mut m = VecMap::new();
        assert_eq!(m.insert(1, 2), None);
        assert_eq!(m.insert(1, 3), Some(2));
        assert_eq!(m.insert(1, 4), Some(3));
    }

    #[test]
    fn test_remove() {
        let mut m = VecMap::new();
        m.insert(1, 2);
        assert_eq!(m.remove(1), Some(2));
        assert_eq!(m.remove(1), None);
    }

    #[test]
    fn test_keys() {
        let mut map = VecMap::new();
        map.insert(1, 'a');
        map.insert(2, 'b');
        map.insert(3, 'c');
        let keys: Vec<_> = map.keys().collect();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&1));
        assert!(keys.contains(&2));
        assert!(keys.contains(&3));
    }

    #[test]
    fn test_values() {
        let mut map = VecMap::new();
        map.insert(1, 'a');
        map.insert(2, 'b');
        map.insert(3, 'c');
        let values: Vec<_> = map.values().cloned().collect();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&'a'));
        assert!(values.contains(&'b'));
        assert!(values.contains(&'c'));
    }

    #[test]
    fn test_iterator() {
        let mut m = VecMap::new();

        assert!(m.insert(0, 1).is_none());
        assert!(m.insert(1, 2).is_none());
        assert!(m.insert(3, 5).is_none());
        assert!(m.insert(6, 10).is_none());
        assert!(m.insert(10, 11).is_none());

        let mut it = m.iter();
        assert_eq!(it.size_hint(), (5, Some(5)));
        assert_eq!(it.next().unwrap(), (0, &1));
        assert_eq!(it.size_hint(), (4, Some(4)));
        assert_eq!(it.next().unwrap(), (1, &2));
        assert_eq!(it.size_hint(), (3, Some(3)));
        assert_eq!(it.next().unwrap(), (3, &5));
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert_eq!(it.next().unwrap(), (6, &10));
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(it.next().unwrap(), (10, &11));
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert!(it.next().is_none());
    }

    #[test]
    fn test_iterator_size_hints() {
        let mut m = VecMap::new();

        assert!(m.insert(0, 1).is_none());
        assert!(m.insert(1, 2).is_none());
        assert!(m.insert(3, 5).is_none());
        assert!(m.insert(6, 10).is_none());
        assert!(m.insert(10, 11).is_none());

        assert_eq!(m.iter().size_hint(), (5, Some(5)));
        assert_eq!(m.iter().rev().size_hint(), (5, Some(5)));
        assert_eq!(m.iter_mut().size_hint(), (5, Some(5)));
        assert_eq!(m.iter_mut().rev().size_hint(), (5, Some(5)));
    }

    #[test]
    fn test_mut_iterator() {
        let mut m = VecMap::new();

        assert!(m.insert(0, 1).is_none());
        assert!(m.insert(1, 2).is_none());
        assert!(m.insert(3, 5).is_none());
        assert!(m.insert(6, 10).is_none());
        assert!(m.insert(10, 11).is_none());

        for (k, v) in &mut m {
            *v += k as isize;
        }

        let mut it = m.iter();
        assert_eq!(it.next().unwrap(), (0, &1));
        assert_eq!(it.next().unwrap(), (1, &3));
        assert_eq!(it.next().unwrap(), (3, &8));
        assert_eq!(it.next().unwrap(), (6, &16));
        assert_eq!(it.next().unwrap(), (10, &21));
        assert!(it.next().is_none());
    }

    #[test]
    fn test_rev_iterator() {
        let mut m = VecMap::new();

        assert!(m.insert(0, 1).is_none());
        assert!(m.insert(1, 2).is_none());
        assert!(m.insert(3, 5).is_none());
        assert!(m.insert(6, 10).is_none());
        assert!(m.insert(10, 11).is_none());

        let mut it = m.iter().rev();
        assert_eq!(it.next().unwrap(), (10, &11));
        assert_eq!(it.next().unwrap(), (6, &10));
        assert_eq!(it.next().unwrap(), (3, &5));
        assert_eq!(it.next().unwrap(), (1, &2));
        assert_eq!(it.next().unwrap(), (0, &1));
        assert!(it.next().is_none());
    }

    #[test]
    fn test_mut_rev_iterator() {
        let mut m = VecMap::new();

        assert!(m.insert(0, 1).is_none());
        assert!(m.insert(1, 2).is_none());
        assert!(m.insert(3, 5).is_none());
        assert!(m.insert(6, 10).is_none());
        assert!(m.insert(10, 11).is_none());

        for (k, v) in m.iter_mut().rev() {
            *v += k as isize;
        }

        let mut it = m.iter();
        assert_eq!(it.next().unwrap(), (0, &1));
        assert_eq!(it.next().unwrap(), (1, &3));
        assert_eq!(it.next().unwrap(), (3, &8));
        assert_eq!(it.next().unwrap(), (6, &16));
        assert_eq!(it.next().unwrap(), (10, &21));
        assert!(it.next().is_none());
    }

    #[test]
    fn test_move_iter() {
        let mut m: VecMap<Box<_>> = VecMap::new();
        m.insert(1, Box::new(2));
        let mut called = false;
        for (k, v) in m {
            assert!(!called);
            called = true;
            assert_eq!(k, 1);
            assert_eq!(v, Box::new(2));
        }
        assert!(called);
    }

    #[test]
    fn test_drain_iterator() {
        let mut map = VecMap::new();
        map.insert(1, "a");
        map.insert(3, "c");
        map.insert(2, "b");

        let vec: Vec<_> = map.drain().collect();

        assert_eq!(vec, [(1, "a"), (2, "b"), (3, "c")]);
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_append() {
        let mut a = VecMap::new();
        a.insert(1, "a");
        a.insert(2, "b");
        a.insert(3, "c");

        let mut b = VecMap::new();
        b.insert(3, "d");  // Overwrite element from a
        b.insert(4, "e");
        b.insert(5, "f");

        a.append(&mut b);

        assert_eq!(a.len(), 5);
        assert_eq!(b.len(), 0);
        // Capacity shouldn't change for possible reuse
        assert!(b.capacity() >= 4);

        assert_eq!(a[1], "a");
        assert_eq!(a[2], "b");
        assert_eq!(a[3], "d");
        assert_eq!(a[4], "e");
        assert_eq!(a[5], "f");
    }

    #[test]
    fn test_split_off() {
        // Split within the key range
        let mut a = VecMap::new();
        a.insert(1, "a");
        a.insert(2, "b");
        a.insert(3, "c");
        a.insert(4, "d");

        let b = a.split_off(3);

        assert_eq!(a.len(), 2);
        assert_eq!(b.len(), 2);

        assert_eq!(a[1], "a");
        assert_eq!(a[2], "b");

        assert_eq!(b[3], "c");
        assert_eq!(b[4], "d");

        // Split at 0
        a.clear();
        a.insert(1, "a");
        a.insert(2, "b");
        a.insert(3, "c");
        a.insert(4, "d");

        let b = a.split_off(0);

        assert_eq!(a.len(), 0);
        assert_eq!(b.len(), 4);
        assert_eq!(b[1], "a");
        assert_eq!(b[2], "b");
        assert_eq!(b[3], "c");
        assert_eq!(b[4], "d");

        // Split behind max_key
        a.clear();
        a.insert(1, "a");
        a.insert(2, "b");
        a.insert(3, "c");
        a.insert(4, "d");

        let b = a.split_off(5);

        assert_eq!(a.len(), 4);
        assert_eq!(b.len(), 0);
        assert_eq!(a[1], "a");
        assert_eq!(a[2], "b");
        assert_eq!(a[3], "c");
        assert_eq!(a[4], "d");
    }

    #[test]
    fn test_show() {
        let mut map = VecMap::new();
        let empty = VecMap::<i32>::new();

        map.insert(1, 2);
        map.insert(3, 4);

        let map_str = format!("{:?}", map);
        assert!(map_str == "{1: 2, 3: 4}" || map_str == "{3: 4, 1: 2}");
        assert_eq!(format!("{:?}", empty), "{}");
    }

    #[test]
    fn test_clone() {
        let mut a = VecMap::new();

        a.insert(1, 'x');
        a.insert(4, 'y');
        a.insert(6, 'z');

        assert_eq!(a.clone().iter().collect::<Vec<_>>(), [(1, &'x'), (4, &'y'), (6, &'z')]);
    }

    #[test]
    fn test_eq() {
        let mut a = VecMap::new();
        let mut b = VecMap::new();

        assert!(a == b);
        assert!(a.insert(0, 5).is_none());
        assert!(a != b);
        assert!(b.insert(0, 4).is_none());
        assert!(a != b);
        assert!(a.insert(5, 19).is_none());
        assert!(a != b);
        assert!(!b.insert(0, 5).is_none());
        assert!(a != b);
        assert!(b.insert(5, 19).is_none());
        assert!(a == b);

        a = VecMap::new();
        b = VecMap::with_capacity(1);
        assert!(a == b);
    }

    #[test]
    fn test_lt() {
        let mut a = VecMap::new();
        let mut b = VecMap::new();

        assert!(!(a < b) && !(b < a));
        assert!(b.insert(2, 5).is_none());
        assert!(a < b);
        assert!(a.insert(2, 7).is_none());
        assert!(!(a < b) && b < a);
        assert!(b.insert(1, 0).is_none());
        assert!(b < a);
        assert!(a.insert(0, 6).is_none());
        assert!(a < b);
        assert!(a.insert(6, 2).is_none());
        assert!(a < b && !(b < a));
    }

    #[test]
    fn test_ord() {
        let mut a = VecMap::new();
        let mut b = VecMap::new();

        assert!(a <= b && a >= b);
        assert!(a.insert(1, 1).is_none());
        assert!(a > b && a >= b);
        assert!(b < a && b <= a);
        assert!(b.insert(2, 2).is_none());
        assert!(b > a && b >= a);
        assert!(a < b && a <= b);
    }

    #[test]
    fn test_hash() {
        fn hash<T: Hash>(t: &T) -> u64 {
            let mut s = DefaultHasher::new();
            t.hash(&mut s);
            s.finish()
        }

        let mut x = VecMap::new();
        let mut y = VecMap::new();

        assert!(hash(&x) == hash(&y));
        x.insert(1, 'a');
        x.insert(2, 'b');
        x.insert(3, 'c');

        y.insert(3, 'c');
        y.insert(2, 'b');
        y.insert(1, 'a');

        assert!(hash(&x) == hash(&y));

        x.insert(1000, 'd');
        x.remove(1000);

        assert!(hash(&x) == hash(&y));
    }

    #[test]
    fn test_from_iter() {
        let xs = [(1, 'a'), (2, 'b'), (3, 'c'), (4, 'd'), (5, 'e')];

        let map: VecMap<_> = xs.iter().cloned().collect();

        for &(k, v) in &xs {
            assert_eq!(map.get(k), Some(&v));
        }
    }

    #[test]
    fn test_index() {
        let mut map = VecMap::new();

        map.insert(1, 2);
        map.insert(2, 1);
        map.insert(3, 4);

        assert_eq!(map[3], 4);
    }

    #[test]
    #[should_panic]
    fn test_index_nonexistent() {
        let mut map = VecMap::new();

        map.insert(1, 2);
        map.insert(2, 1);
        map.insert(3, 4);

        map[4];
    }

    #[test]
    fn test_entry() {
        let xs = [(1, 10), (2, 20), (3, 30), (4, 40), (5, 50), (6, 60)];

        let mut map: VecMap<_> = xs.iter().cloned().collect();

        // Existing key (insert)
        match map.entry(1) {
            Vacant(_) => unreachable!(),
            Occupied(mut view) => {
                assert_eq!(view.get(), &10);
                assert_eq!(view.insert(100), 10);
            }
        }

        assert_eq!(map.get(1).unwrap(), &100);
        assert_eq!(map.len(), 6);

        // Existing key (update)
        match map.entry(2) {
            Vacant(_) => unreachable!(),
            Occupied(mut view) => {
                let v = view.get_mut();
                *v *= 10;
            }
        }

        assert_eq!(map.get(2).unwrap(), &200);
        assert_eq!(map.len(), 6);

        // Existing key (take)
        match map.entry(3) {
            Vacant(_) => unreachable!(),
            Occupied(view) => {
                assert_eq!(view.remove(), 30);
            }
        }

        assert_eq!(map.get(3), None);
        assert_eq!(map.len(), 5);

        // Inexistent key (insert)
        match map.entry(10) {
            Occupied(_) => unreachable!(),
            Vacant(view) => {
                assert_eq!(*view.insert(1000), 1000);
            }
        }

        assert_eq!(map.get(10).unwrap(), &1000);
        assert_eq!(map.len(), 6);
    }

    #[test]
    fn test_extend_ref() {
        let mut a = VecMap::new();
        a.insert(1, "one");
        let mut b = VecMap::new();
        b.insert(2, "two");
        b.insert(3, "three");

        a.extend(&b);

        assert_eq!(a.len(), 3);
        assert_eq!(a[&1], "one");
        assert_eq!(a[&2], "two");
        assert_eq!(a[&3], "three");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde() {
        use serde::{Serialize, Deserialize};
        fn impls_serde_traits<'de, S: Serialize + Deserialize<'de>>() {}

        impls_serde_traits::<VecMap<u32>>();
    }

    #[test]
    fn test_retain() {
        let mut map = VecMap::new();
        map.insert(1, "one");
        map.insert(2, "two");
        map.insert(3, "three");
        map.retain(|k, v| match k {
            1 => false,
            2 => {
                *v = "two changed";
                true
            },
            3 => false,
            _ => panic!(),
        });

        assert_eq!(map.len(), 1);
        assert_eq!(map.get(1), None);
        assert_eq!(map[2], "two changed");
        assert_eq!(map.get(3), None);
    }
}
