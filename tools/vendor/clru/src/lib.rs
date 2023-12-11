//! Another LRU cache implementation in rust.
//! It has two main characteristics that differentiates it from other implementation:
//!
//! 1. It is backed by a [HashMap](https://doc.rust-lang.org/std/collections/struct.HashMap.html): it
//!    offers a O(1) time complexity (amortized average) for any operation that requires to lookup an entry from
//!    a key.
//!
//! 2. It is a weighted cache: each key-value pair has a weight and the capacity serves as both as:
//!    * a limit to the number of elements
//!    * and as a limit to the total weight of its elements
//!
//!    using the following formula:
//!
//!    [`CLruCache::len`] + [`CLruCache::weight`] <= [`CLruCache::capacity`]
//!
//! Even though most operations don't depend on the number of elements in the cache,
//! [`CLruCache::put_with_weight`] has a special behavior: because it needs to make room
//! for the new element, it will remove enough least recently used elements. In the worst
//! case, that will require to fully empty the cache. Additionally, if the weight of the
//! new element is too big, the insertion can fail.
//!
//! For the common case of an LRU cache whose elements don't have a weight, a default
//! [`ZeroWeightScale`] is provided and unlocks some useful APIs like:
//!
//! * [`CLruCache::put`]: an infallible insertion that will remove a maximum of 1 element.
//! * [`CLruCache::put_or_modify`]: a conditional insertion or modification flow similar
//!   to the entry API of [`HashMap`].
//! * [`CLruCache::try_put_or_modify`]: fallible version of [`CLruCache::put_or_modify`].
//! * All APIs that allow to retrieve a mutable reference to a value (e.g.: [`CLruCache::get_mut`]).
//!
//! The cache requires the keys to be clonable because it will store 2 instances
//! of each key in different internal data structures. If cloning a key can be
//! expensive, you might want to consider using an [`std::rc::Rc`] or an [`std::sync::Arc`].
//!
//! ## Examples
//!
//! ### Using the default [`ZeroWeightScale`]:
//!
//! ```rust
//!
//! use std::num::NonZeroUsize;
//! use clru::CLruCache;
//!
//! let mut cache = CLruCache::new(NonZeroUsize::new(2).unwrap());
//! cache.put("apple".to_string(), 3);
//! cache.put("banana".to_string(), 2);
//!
//! assert_eq!(cache.get("apple"), Some(&3));
//! assert_eq!(cache.get("banana"), Some(&2));
//! assert!(cache.get("pear").is_none());
//!
//! assert_eq!(cache.put("banana".to_string(), 4), Some(2));
//! assert_eq!(cache.put("pear".to_string(), 5), None);
//!
//! assert_eq!(cache.get("pear"), Some(&5));
//! assert_eq!(cache.get("banana"), Some(&4));
//! assert!(cache.get("apple").is_none());
//!
//! {
//!     let v = cache.get_mut("banana").unwrap();
//!     *v = 6;
//! }
//!
//! assert_eq!(cache.get("banana"), Some(&6));
//! ```
//!
//! ### Using a custom [`WeightScale`] implementation:
//!
//! ```rust
//!
//! use std::num::NonZeroUsize;
//! use clru::{CLruCache, CLruCacheConfig, WeightScale};
//!
//! struct CustomScale;
//!
//! impl WeightScale<String, &str> for CustomScale {
//!     fn weight(&self, _key: &String, value: &&str) -> usize {
//!         value.len()
//!     }
//! }
//!
//! let mut cache = CLruCache::with_config(
//!     CLruCacheConfig::new(NonZeroUsize::new(6).unwrap()).with_scale(CustomScale),
//! );
//!
//! assert_eq!(cache.put_with_weight("apple".to_string(), "red").unwrap(), None);
//! assert_eq!(
//!     cache.put_with_weight("apple".to_string(), "green").unwrap(),
//!     Some("red")
//! );
//!
//! assert_eq!(cache.len(), 1);
//! assert_eq!(cache.get("apple"), Some(&"green"));
//! ```

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(warnings)]

mod config;
mod list;
mod weight;

pub use crate::config::*;
use crate::list::{FixedSizeList, FixedSizeListIter, FixedSizeListIterMut};
pub use crate::weight::*;

use std::borrow::Borrow;
use std::collections::hash_map::Entry;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};
use std::iter::FromIterator;
use std::num::NonZeroUsize;

#[derive(Debug)]
struct CLruNode<K, V> {
    key: K,
    value: V,
}

/// A weighted LRU cache with mostly¹ constant time operations.
///
/// Each key-value pair in the cache can have a weight that is retrieved
/// using the provided [`WeightScale`] implementation. The default scale is
/// [`ZeroWeightScale`] and always return 0. The number of elements that
/// can be stored in the cache is conditioned by the sum of [`CLruCache::len`]
/// and [`CLruCache::weight`]:
///
/// [`CLruCache::len`] + [`CLruCache::weight`] <= [`CLruCache::capacity`]
///
/// Using the default [`ZeroWeightScale`] scale unlocks some useful APIs
/// that can currently only be implemented for this scale. The most interesting
/// ones are probably:
///
/// * [`CLruCache::put`]
/// * [`CLruCache::put_or_modify`]
/// * [`CLruCache::try_put_or_modify`]
///
/// But more generally, using [`ZeroWeightScale`] unlocks all methods that return
/// a mutable reference to the value of an element.
/// This is because modifying the value of an element can lead to a modification
/// of its weight and therefore would put the cache into an incoherent state.
/// For the same reason, it is a logic error for a value to change weight while
/// being stored in the cache.
///
/// The cache requires the keys to be clonable because it will store 2 instances
/// of each key in different internal data structures. If cloning a key can be
/// expensive, you might want to consider using an `Rc` or an `Arc`.
///
/// Note 1: See [`CLruCache::put_with_weight`]
#[derive(Debug)]
pub struct CLruCache<K, V, S = RandomState, W: WeightScale<K, V> = ZeroWeightScale> {
    lookup: HashMap<K, usize, S>,
    storage: FixedSizeList<CLruNode<K, V>>,
    scale: W,
    weight: usize,
}

impl<K: Eq + Hash, V> CLruCache<K, V> {
    /// Creates a new LRU cache that holds at most `capacity` elements.
    pub fn new(capacity: NonZeroUsize) -> Self {
        Self {
            lookup: HashMap::new(),
            storage: FixedSizeList::new(capacity.get()),
            scale: ZeroWeightScale,
            weight: 0,
        }
    }

    /// Creates a new LRU cache that holds at most `capacity` elements
    /// and pre-allocates memory in order to hold at least `reserve` elements
    /// without reallocating.
    pub fn with_memory(capacity: NonZeroUsize, mut reserve: usize) -> Self {
        if reserve > capacity.get() {
            reserve = capacity.get();
        }
        Self {
            lookup: HashMap::with_capacity(reserve),
            storage: FixedSizeList::with_memory(capacity.get(), reserve),
            scale: ZeroWeightScale,
            weight: 0,
        }
    }
}

impl<K: Eq + Hash, V, S: BuildHasher> CLruCache<K, V, S> {
    /// Creates a new LRU cache that holds at most `capacity` elements
    /// and uses the provided hash builder to hash keys.
    pub fn with_hasher(capacity: NonZeroUsize, hash_builder: S) -> CLruCache<K, V, S> {
        Self {
            lookup: HashMap::with_hasher(hash_builder),
            storage: FixedSizeList::new(capacity.get()),
            scale: ZeroWeightScale,
            weight: 0,
        }
    }
}

impl<K: Eq + Hash, V, W: WeightScale<K, V>> CLruCache<K, V, RandomState, W> {
    /// Creates a new LRU cache that holds at most `capacity` elements
    /// and uses the provided scale to retrieve value's weight.
    pub fn with_scale(capacity: NonZeroUsize, scale: W) -> CLruCache<K, V, RandomState, W> {
        Self {
            lookup: HashMap::with_hasher(RandomState::default()),
            storage: FixedSizeList::new(capacity.get()),
            scale,
            weight: 0,
        }
    }
}

impl<K: Clone + Eq + Hash, V, S: BuildHasher, W: WeightScale<K, V>> CLruCache<K, V, S, W> {
    /// Creates a new LRU cache using the provided configuration.
    pub fn with_config(config: CLruCacheConfig<K, V, S, W>) -> Self {
        let CLruCacheConfig {
            capacity,
            hash_builder,
            reserve,
            scale,
            ..
        } = config;
        Self {
            lookup: HashMap::with_hasher(hash_builder),
            storage: if let Some(reserve) = reserve {
                FixedSizeList::with_memory(capacity.get(), reserve)
            } else {
                FixedSizeList::new(capacity.get())
            },
            scale,
            weight: 0,
        }
    }

    /// Returns the number of key-value pairs that are currently in the cache.
    #[inline]
    pub fn len(&self) -> usize {
        debug_assert_eq!(self.lookup.len(), self.storage.len());
        self.storage.len()
    }

    /// Returns the capacity of the cache. It serves as a limit for
    /// * the number of elements that the cache can hold.
    /// * the total weight of the elements in the cache.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.storage.capacity()
    }

    /// Returns the total weight of the elements in the cache.
    #[inline]
    pub fn weight(&self) -> usize {
        self.weight
    }

    /// Returns a bool indicating whether the cache is empty or not.
    #[inline]
    pub fn is_empty(&self) -> bool {
        debug_assert_eq!(self.lookup.is_empty(), self.storage.is_empty());
        self.storage.is_empty()
    }

    /// Returns a bool indicating whether the cache is full or not.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len() + self.weight() == self.capacity()
    }

    /// Returns the value corresponding to the most recently used item or `None` if the cache is empty.
    /// Like `peek`, `front` does not update the LRU list so the item's position will be unchanged.
    pub fn front(&self) -> Option<(&K, &V)> {
        self.storage
            .front()
            .map(|CLruNode { key, value }| (key, value))
    }

    /// Returns the value corresponding to the least recently used item or `None` if the cache is empty.
    /// Like `peek`, `back` does not update the LRU list so the item's position will be unchanged.
    pub fn back(&self) -> Option<(&K, &V)> {
        self.storage
            .back()
            .map(|CLruNode { key, value }| (key, value))
    }

    /// Puts a key-value pair into cache taking it's weight into account.
    /// If the weight of the new element is greater than what the cache can hold,
    /// it returns the provided key-value pair as an error.
    /// Otherwise, it removes enough elements for the new element to be inserted,
    /// thus making it a non constant time operation.
    /// If the key already exists in the cache, then it updates the key's value and returns the old value.
    /// Otherwise, `None` is returned.
    pub fn put_with_weight(&mut self, key: K, value: V) -> Result<Option<V>, (K, V)> {
        let weight = self.scale.weight(&key, &value);
        if weight >= self.capacity() {
            return Err((key, value));
        }
        match self.lookup.entry(key) {
            Entry::Occupied(mut occ) => {
                // TODO: store keys in the cache itself for reuse.
                let mut keys = Vec::new();
                let old = self.storage.remove(*occ.get()).unwrap();
                self.weight -= self.scale.weight(&old.key, &old.value);
                while self.storage.len() + self.weight + weight >= self.storage.capacity() {
                    let node = self.storage.pop_back().unwrap();
                    self.weight -= self.scale.weight(&node.key, &node.value);
                    keys.push(node.key);
                }
                // It's fine to unwrap here because:
                // * the cache capacity is non zero
                // * the cache cannot be full
                let (idx, _) = self
                    .storage
                    .push_front(CLruNode {
                        key: occ.key().clone(),
                        value,
                    })
                    .unwrap();
                occ.insert(idx);
                self.weight += weight;
                for key in keys.drain(..) {
                    self.lookup.remove(&key);
                }
                Ok(Some(old.value))
            }
            Entry::Vacant(vac) => {
                let mut keys = Vec::new();
                while self.storage.len() + self.weight + weight >= self.storage.capacity() {
                    let node = self.storage.pop_back().unwrap();
                    self.weight -= self.scale.weight(&node.key, &node.value);
                    keys.push(node.key);
                }
                // It's fine to unwrap here because:
                // * the cache capacity is non zero
                // * the cache cannot be full
                let (idx, _) = self
                    .storage
                    .push_front(CLruNode {
                        key: vac.key().clone(),
                        value,
                    })
                    .unwrap();
                vac.insert(idx);
                self.weight += weight;
                for key in keys.drain(..) {
                    self.lookup.remove(&key);
                }
                Ok(None)
            }
        }
    }

    /// Returns a reference to the value of the key in the cache or `None` if it is not present in the cache.
    /// Moves the key to the head of the LRU list if it exists.
    pub fn get<Q: ?Sized>(&mut self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let idx = *self.lookup.get(key)?;
        self.storage.move_front(idx).map(|node| &node.value)
    }

    /// Removes and returns the value corresponding to the key from the cache or `None` if it does not exist.
    pub fn pop<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let idx = self.lookup.remove(key)?;
        self.storage.remove(idx).map(|CLruNode { key, value, .. }| {
            self.weight -= self.scale.weight(&key, &value);
            value
        })
    }

    /// Removes and returns the key and value corresponding to the most recently used item or `None` if the cache is empty.
    pub fn pop_front(&mut self) -> Option<(K, V)> {
        if let Some(CLruNode { key, value }) = self.storage.pop_front() {
            self.lookup.remove(&key).unwrap();
            self.weight -= self.scale.weight(&key, &value);
            Some((key, value))
        } else {
            None
        }
    }

    /// Removes and returns the key and value corresponding to the least recently used item or `None` if the cache is empty.
    pub fn pop_back(&mut self) -> Option<(K, V)> {
        if let Some(CLruNode { key, value }) = self.storage.pop_back() {
            self.lookup.remove(&key).unwrap();
            self.weight -= self.scale.weight(&key, &value);
            Some((key, value))
        } else {
            None
        }
    }

    /// Returns a reference to the value corresponding to the key in the cache or `None` if it is not present in the cache.
    /// Unlike `get`, `peek` does not update the LRU list so the key's position will be unchanged.
    pub fn peek<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let idx = *self.lookup.get(key)?;
        self.storage.get(idx).map(|node| &node.value)
    }

    /// Returns a bool indicating whether the given key is in the cache.
    /// Does not update the LRU list.
    pub fn contains<Q: ?Sized>(&mut self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.peek(key).is_some()
    }

    /// Clears the contents of the cache.
    #[inline]
    pub fn clear(&mut self) {
        self.lookup.clear();
        self.storage.clear();
        self.weight = 0;
    }

    /// Resizes the cache.
    /// If the new capacity is smaller than the size of the current cache any entries past the new capacity are discarded.
    pub fn resize(&mut self, capacity: NonZeroUsize) {
        while capacity.get() < self.storage.len() + self.weight() {
            if let Some(CLruNode { key, value }) = self.storage.pop_back() {
                self.lookup.remove(&key).unwrap();
                self.weight -= self.scale.weight(&key, &value);
            }
        }
        self.storage.resize(capacity.get());
        for i in 0..self.len() {
            let data = self.storage.get(i).unwrap();
            *self.lookup.get_mut(&data.key).unwrap() = i;
        }
    }

    /// Retains only the elements specified by the predicate.
    /// In other words, remove all pairs `(k, v)` such that `f(&k, &v)` returns `false`.
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&K, &V) -> bool,
    {
        self.storage.retain(
            #[inline]
            |CLruNode { ref key, ref value }| {
                if f(key, value) {
                    true
                } else {
                    self.lookup.remove(key).unwrap();
                    false
                }
            },
        )
    }
}

impl<K, V, S, W: WeightScale<K, V>> CLruCache<K, V, S, W> {
    /// Returns an iterator visiting all entries in order.
    /// The iterator element type is `(&'a K, &'a V)`.
    pub fn iter(&self) -> CLruCacheIter<'_, K, V> {
        CLruCacheIter {
            iter: self.storage.iter(),
        }
    }
}

impl<K: Clone + Eq + Hash, V, S: BuildHasher> CLruCache<K, V, S> {
    /// Puts a key-value pair into cache.
    /// If the key already exists in the cache, then it updates the key's value and returns the old value.
    /// Otherwise, `None` is returned.
    pub fn put(&mut self, key: K, value: V) -> Option<V> {
        match self.lookup.entry(key) {
            Entry::Occupied(occ) => {
                // It's fine to unwrap here because:
                // * the entry already exists
                let node = self.storage.move_front(*occ.get()).unwrap();
                Some(std::mem::replace(&mut node.value, value))
            }
            Entry::Vacant(vac) => {
                let key = vac.key().clone();
                if self.storage.is_full() {
                    let idx = self.storage.back_idx();
                    // It's fine to unwrap here because:
                    // * the cache capacity is non zero
                    // * the cache is full
                    let node = self.storage.move_front(idx).unwrap();
                    let obsolete_key = std::mem::replace(node, CLruNode { key, value }).key;
                    vac.insert(idx);
                    self.lookup.remove(&obsolete_key);
                } else {
                    // It's fine to unwrap here because:
                    // * the cache capacity is non zero
                    // * the cache is not full
                    let (idx, _) = self.storage.push_front(CLruNode { key, value }).unwrap();
                    vac.insert(idx);
                }
                None
            }
        }
    }

    /// Puts a new key-value pair or modify an already existing value.
    /// If the key already exists in the cache, then `modify_op` supplied function is called.
    /// Otherwise, `put_op` supplied function is called.
    /// Returns a mutable reference to the value.
    ///
    /// The additional `data` argument can be used to pass extra information
    /// to the supplied functions. This can useful when both functions need
    /// to access the same variable.
    pub fn put_or_modify<T, P: FnMut(&K, T) -> V, M: FnMut(&K, &mut V, T)>(
        &mut self,
        key: K,
        mut put_op: P,
        mut modify_op: M,
        data: T,
    ) -> &mut V {
        match self.lookup.entry(key) {
            Entry::Occupied(occ) => {
                // It's fine to unwrap here because:
                // * the entry already exists
                let node = self.storage.move_front(*occ.get()).unwrap();
                modify_op(&node.key, &mut node.value, data);
                &mut node.value
            }
            Entry::Vacant(vac) => {
                let key = vac.key().clone();
                let value = put_op(&key, data);
                if self.storage.is_full() {
                    let index = self.storage.back_idx();
                    // It's fine to unwrap here because:
                    // * the cache capacity is non zero
                    // * the cache is full
                    let node = self.storage.move_front(index).unwrap();
                    let obsolete_key = std::mem::replace(node, CLruNode { key, value }).key;
                    vac.insert(index);
                    self.lookup.remove(&obsolete_key);
                    &mut node.value
                } else {
                    // It's fine to unwrap here because:
                    // * the cache capacity is non zero
                    // * the cache cannot be full
                    let (idx, node) = self.storage.push_front(CLruNode { key, value }).unwrap();
                    vac.insert(idx);
                    &mut node.value
                }
            }
        }
    }

    /// Puts a new key-value pair or modify an already existing value.
    /// If the key already exists in the cache, then `modify_op` supplied function is called.
    /// Otherwise, `put_op` supplied function is called.
    /// Returns a mutable reference to the value or an error.
    ///
    /// The additional `data` argument can be used to pass extra information
    /// to the supplied functions. This can useful when both functions need
    /// to access the same variable.
    ///
    /// This is the fallible version of [`CLruCache::put_or_modify`].
    pub fn try_put_or_modify<
        T,
        E,
        P: FnMut(&K, T) -> Result<V, E>,
        M: FnMut(&K, &mut V, T) -> Result<(), E>,
    >(
        &mut self,
        key: K,
        mut put_op: P,
        mut modify_op: M,
        data: T,
    ) -> Result<&mut V, E> {
        match self.lookup.entry(key) {
            Entry::Occupied(occ) => {
                // It's fine to unwrap here because:
                // * the entry already exists
                let node = self.storage.move_front(*occ.get()).unwrap();
                match modify_op(&node.key, &mut node.value, data) {
                    Ok(()) => Ok(&mut node.value),
                    Err(err) => Err(err),
                }
            }
            Entry::Vacant(vac) => {
                let value = match put_op(vac.key(), data) {
                    Ok(value) => value,
                    Err(err) => return Err(err),
                };
                let key = vac.key().clone();
                if self.storage.is_full() {
                    let idx = self.storage.back_idx();
                    // It's fine to unwrap here because:
                    // * the cache capacity is non zero
                    // * the cache is full
                    let node = self.storage.move_front(idx).unwrap();
                    let obsolete_key = std::mem::replace(node, CLruNode { key, value }).key;
                    vac.insert(idx);
                    self.lookup.remove(&obsolete_key);
                    Ok(&mut node.value)
                } else {
                    // It's fine to unwrap here because:
                    // * the cache capacity is non zero
                    // * the cache cannot be full
                    let (idx, node) = self.storage.push_front(CLruNode { key, value }).unwrap();
                    vac.insert(idx);
                    Ok(&mut node.value)
                }
            }
        }
    }

    /// Returns the value corresponding to the most recently used item or `None` if the cache is empty.
    /// Like `peek`, `font` does not update the LRU list so the item's position will be unchanged.
    pub fn front_mut(&mut self) -> Option<(&K, &mut V)> {
        self.storage
            .front_mut()
            .map(|CLruNode { key, value }| (&*key, value))
    }

    /// Returns the value corresponding to the least recently used item or `None` if the cache is empty.
    /// Like `peek`, `back` does not update the LRU list so the item's position will be unchanged.
    pub fn back_mut(&mut self) -> Option<(&K, &mut V)> {
        self.storage
            .back_mut()
            .map(|CLruNode { key, value }| (&*key, value))
    }

    /// Returns a mutable reference to the value of the key in the cache or `None` if it is not present in the cache.
    /// Moves the key to the head of the LRU list if it exists.
    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let idx = *self.lookup.get(key)?;
        self.storage.move_front(idx).map(|node| &mut node.value)
    }

    /// Returns a mutable reference to the value corresponding to the key in the cache or `None` if it is not present in the cache.
    /// Unlike `get_mut`, `peek_mut` does not update the LRU list so the key's position will be unchanged.
    pub fn peek_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let idx = *self.lookup.get(key)?;
        self.storage.get_mut(idx).map(|node| &mut node.value)
    }

    /// Retains only the elements specified by the predicate.
    /// In other words, remove all pairs `(k, v)` such that `f(&k, &mut v)` returns `false`.
    pub fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        self.storage.retain_mut(
            #[inline]
            |CLruNode {
                 ref key,
                 ref mut value,
             }| {
                if f(key, value) {
                    true
                } else {
                    self.lookup.remove(key).unwrap();
                    false
                }
            },
        )
    }
}

impl<K, V, S> CLruCache<K, V, S> {
    /// Returns an iterator visiting all entries in order, giving a mutable reference on V.
    /// The iterator element type is `(&'a K, &'a mut V)`.
    pub fn iter_mut(&mut self) -> CLruCacheIterMut<'_, K, V> {
        CLruCacheIterMut {
            iter: self.storage.iter_mut(),
        }
    }
}

/// An iterator over the entries of a `CLruCache`.
///
/// This `struct` is created by the [`iter`] method on [`CLruCache`][`CLruCache`].
/// See its documentation for more.
///
/// [`iter`]: struct.CLruCache.html#method.iter
/// [`CLruCache`]: struct.CLruCache.html
#[derive(Clone, Debug)]
pub struct CLruCacheIter<'a, K, V> {
    iter: FixedSizeListIter<'a, CLruNode<K, V>>,
}

impl<'a, K, V> Iterator for CLruCacheIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(_, CLruNode { key, value })| (key.borrow(), value))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator for CLruCacheIter<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back()
            .map(|(_, CLruNode { key, value })| (key.borrow(), value))
    }
}

impl<'a, K, V> ExactSizeIterator for CLruCacheIter<'a, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, K, V, S, W: WeightScale<K, V>> IntoIterator for &'a CLruCache<K, V, S, W> {
    type Item = (&'a K, &'a V);
    type IntoIter = CLruCacheIter<'a, K, V>;

    #[inline]
    fn into_iter(self) -> CLruCacheIter<'a, K, V> {
        self.iter()
    }
}

/// An iterator over mutables entries of a `CLruCache`.
///
/// This `struct` is created by the [`iter_mut`] method on [`CLruCache`][`CLruCache`].
/// See its documentation for more.
///
/// [`iter_mut`]: struct.CLruCache.html#method.iter_mut
/// [`CLruCache`]: struct.CLruCache.html
pub struct CLruCacheIterMut<'a, K, V> {
    iter: FixedSizeListIterMut<'a, CLruNode<K, V>>,
}

impl<'a, K, V> Iterator for CLruCacheIterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(_, CLruNode { key, value })| (&*key, value))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator for CLruCacheIterMut<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back()
            .map(|(_, CLruNode { key, value })| (&*key, value))
    }
}

impl<'a, K, V> ExactSizeIterator for CLruCacheIterMut<'a, K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, K, V, S> IntoIterator for &'a mut CLruCache<K, V, S> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = CLruCacheIterMut<'a, K, V>;

    #[inline]
    fn into_iter(self) -> CLruCacheIterMut<'a, K, V> {
        self.iter_mut()
    }
}

/// An owning iterator over the elements of a `CLruCache`.
///
/// This `struct` is created by the [`into_iter`] method on [`CLruCache`]
/// (provided by the `IntoIterator` trait). See its documentation for more.
///
/// [`into_iter`]: struct.CLruCache.html#method.into_iter
pub struct CLruCacheIntoIter<K, V, S, W: WeightScale<K, V>> {
    cache: CLruCache<K, V, S, W>,
}

impl<K: Clone + Eq + Hash, V, S: BuildHasher, W: WeightScale<K, V>> Iterator
    for CLruCacheIntoIter<K, V, S, W>
{
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<(K, V)> {
        self.cache.pop_front()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.cache.len(), Some(self.cache.len()))
    }
}

impl<K: Clone + Eq + Hash, V, S: BuildHasher, W: WeightScale<K, V>> DoubleEndedIterator
    for CLruCacheIntoIter<K, V, S, W>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.cache.pop_back()
    }
}

impl<K: Clone + Eq + Hash, V, S: BuildHasher, W: WeightScale<K, V>> ExactSizeIterator
    for CLruCacheIntoIter<K, V, S, W>
{
    fn len(&self) -> usize {
        self.size_hint().0
    }
}

impl<K: Clone + Eq + Hash, V, S: BuildHasher, W: WeightScale<K, V>> IntoIterator
    for CLruCache<K, V, S, W>
{
    type Item = (K, V);
    type IntoIter = CLruCacheIntoIter<K, V, S, W>;

    /// Consumes the cache into an iterator yielding elements by value.
    #[inline]
    fn into_iter(self) -> CLruCacheIntoIter<K, V, S, W> {
        CLruCacheIntoIter { cache: self }
    }
}

impl<K: Clone + Eq + Hash, V, S: BuildHasher + Default> FromIterator<(K, V)>
    for CLruCache<K, V, S>
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let cap = NonZeroUsize::new(usize::MAX).unwrap();
        let mut cache = CLruCache::with_hasher(cap, S::default());

        for (k, v) in iter {
            cache.put(k, v);
        }

        cache.resize(
            NonZeroUsize::new(cache.len()).unwrap_or_else(|| NonZeroUsize::new(1).unwrap()),
        );

        cache
    }
}

impl<K: Clone + Eq + Hash, V, S: BuildHasher> Extend<(K, V)> for CLruCache<K, V, S> {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        for (k, v) in iter {
            self.put(k, v);
        }
    }
}

#[cfg(test)]
#[allow(clippy::bool_assert_comparison)]
mod tests {
    use super::*;

    #[allow(unsafe_code)]
    const ONE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1) };
    #[allow(unsafe_code)]
    const TWO: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(2) };
    #[allow(unsafe_code)]
    const THREE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(3) };
    #[allow(unsafe_code)]
    const FOUR: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(4) };
    #[allow(unsafe_code)]
    const FIVE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(5) };
    #[allow(unsafe_code)]
    const SIX: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(6) };
    #[allow(unsafe_code)]
    const HEIGHT: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(8) };
    #[allow(unsafe_code)]
    const MANY: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(200) };

    #[test]
    fn test_insert_and_get() {
        let mut cache = CLruCache::new(TWO);
        assert!(cache.is_empty());

        assert_eq!(cache.put("apple", "red"), None);
        assert_eq!(cache.put("banana", "yellow"), None);

        assert_eq!(cache.capacity(), 2);
        assert_eq!(cache.len(), 2);
        assert!(!cache.is_empty());
        assert!(cache.is_full());
        assert_eq!(cache.get(&"apple"), Some(&"red"));
        assert_eq!(cache.get(&"banana"), Some(&"yellow"));
    }

    #[test]
    fn test_insert_and_get_mut() {
        let mut cache = CLruCache::new(TWO);

        cache.put("apple", "red");
        cache.put("banana", "yellow");

        assert_eq!(cache.capacity(), 2);
        assert_eq!(cache.len(), 2);
        assert!(!cache.is_empty());
        assert!(cache.is_full());
        assert_eq!(cache.get_mut(&"apple"), Some(&mut "red"));
        assert_eq!(cache.get_mut(&"banana"), Some(&mut "yellow"));
    }

    #[test]
    fn test_get_mut_and_update() {
        let mut cache = CLruCache::new(TWO);

        cache.put("apple", 1);
        cache.put("banana", 3);

        {
            let v = cache.get_mut(&"apple").unwrap();
            *v = 4;
        }

        assert_eq!(cache.capacity(), 2);
        assert_eq!(cache.len(), 2);
        assert!(!cache.is_empty());
        assert!(cache.is_full());
        assert_eq!(cache.get_mut(&"apple"), Some(&mut 4));
        assert_eq!(cache.get_mut(&"banana"), Some(&mut 3));
    }

    #[test]
    fn test_insert_update() {
        let mut cache = CLruCache::new(ONE);

        assert_eq!(cache.put("apple", "red"), None);
        assert_eq!(cache.put("apple", "green"), Some("red"));

        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get(&"apple"), Some(&"green"));
    }

    #[test]
    fn test_insert_removes_oldest() {
        let mut cache = CLruCache::new(TWO);

        assert_eq!(cache.put("apple", "red"), None);
        assert_eq!(cache.put("banana", "yellow"), None);
        assert_eq!(cache.put("pear", "green"), None);

        assert!(cache.get(&"apple").is_none());
        assert_eq!(cache.get(&"banana"), Some(&"yellow"));
        assert_eq!(cache.get(&"pear"), Some(&"green"));

        // Even though we inserted "apple" into the cache earlier it has since been removed from
        // the cache so there is no current value for `insert` to return.
        assert_eq!(cache.put("apple", "green"), None);
        assert_eq!(cache.put("tomato", "red"), None);

        assert!(cache.get(&"pear").is_none());
        assert_eq!(cache.get(&"apple"), Some(&"green"));
        assert_eq!(cache.get(&"tomato"), Some(&"red"));
    }

    #[test]
    fn test_peek() {
        let mut cache = CLruCache::new(TWO);

        cache.put("apple", "red");
        cache.put("banana", "yellow");

        assert_eq!(cache.peek(&"banana"), Some(&"yellow"));
        assert_eq!(cache.peek(&"apple"), Some(&"red"));

        cache.put("pear", "green");

        assert!(cache.peek(&"apple").is_none());
        assert_eq!(cache.peek(&"banana"), Some(&"yellow"));
        assert_eq!(cache.peek(&"pear"), Some(&"green"));
    }

    #[test]
    fn test_peek_mut() {
        let mut cache = CLruCache::new(TWO);

        cache.put("apple", "red");
        cache.put("banana", "yellow");

        assert_eq!(cache.peek_mut(&"banana"), Some(&mut "yellow"));
        assert_eq!(cache.peek_mut(&"apple"), Some(&mut "red"));
        assert!(cache.peek_mut(&"pear").is_none());

        cache.put("pear", "green");

        assert!(cache.peek_mut(&"apple").is_none());
        assert_eq!(cache.peek_mut(&"banana"), Some(&mut "yellow"));
        assert_eq!(cache.peek_mut(&"pear"), Some(&mut "green"));

        {
            let v = cache.peek_mut(&"banana").unwrap();
            *v = "green";
        }

        assert_eq!(cache.peek_mut(&"banana"), Some(&mut "green"));
    }

    #[test]
    fn test_contains() {
        let mut cache = CLruCache::new(TWO);

        cache.put("apple", "red");
        cache.put("banana", "yellow");
        cache.put("pear", "green");

        assert!(!cache.contains(&"apple"));
        assert!(cache.contains(&"banana"));
        assert!(cache.contains(&"pear"));
    }

    #[test]
    fn test_pop() {
        let mut cache = CLruCache::new(TWO);

        cache.put("apple", "red");
        cache.put("banana", "yellow");

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(&"apple"), Some(&"red"));
        assert_eq!(cache.get(&"banana"), Some(&"yellow"));

        let popped = cache.pop(&"apple");
        assert!(popped.is_some());
        assert_eq!(popped.unwrap(), "red");
        assert_eq!(cache.len(), 1);
        assert!(cache.get(&"apple").is_none());
        assert_eq!(cache.get(&"banana"), Some(&"yellow"));
    }

    #[test]
    fn test_pop_front() {
        let mut cache = CLruCache::new(MANY);

        for i in 0..75 {
            cache.put(i, "A");
        }
        for i in 0..75 {
            cache.put(i + 100, "B");
        }
        for i in 0..75 {
            cache.put(i + 200, "C");
        }
        assert_eq!(cache.len(), 200);

        for i in 0..75 {
            assert_eq!(cache.get(&(74 - i + 100)), Some(&"B"));
        }
        assert_eq!(cache.get(&25), Some(&"A"));

        assert_eq!(cache.pop_front(), Some((25, "A")));
        for i in 0..75 {
            assert_eq!(cache.pop_front(), Some((i + 100, "B")));
        }
        for i in 0..75 {
            assert_eq!(cache.pop_front(), Some((74 - i + 200, "C")));
        }
        for i in 0..49 {
            assert_eq!(cache.pop_front(), Some((74 - i, "A")));
        }
        for _ in 0..50 {
            assert_eq!(cache.pop_front(), None);
        }
    }

    #[test]
    fn test_pop_back() {
        let mut cache = CLruCache::new(MANY);

        for i in 0..75 {
            cache.put(i, "A");
        }
        for i in 0..75 {
            cache.put(i + 100, "B");
        }
        for i in 0..75 {
            cache.put(i + 200, "C");
        }
        assert_eq!(cache.len(), 200);

        for i in 0..75 {
            assert_eq!(cache.get(&(74 - i + 100)), Some(&"B"));
        }
        assert_eq!(cache.get(&25), Some(&"A"));

        for i in 26..75 {
            assert_eq!(cache.pop_back(), Some((i, "A")));
        }
        for i in 0..75 {
            assert_eq!(cache.pop_back(), Some((i + 200, "C")));
        }
        for i in 0..75 {
            assert_eq!(cache.pop_back(), Some((74 - i + 100, "B")));
        }
        assert_eq!(cache.pop_back(), Some((25, "A")));
        for _ in 0..50 {
            assert_eq!(cache.pop_back(), None);
        }
    }

    #[test]
    fn test_clear() {
        let mut cache = CLruCache::new(TWO);

        cache.put("apple", "red");
        cache.put("banana", "yellow");

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(&"apple"), Some(&"red"));
        assert_eq!(cache.get(&"banana"), Some(&"yellow"));

        cache.clear();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_resize_larger() {
        let mut cache = CLruCache::new(TWO);

        cache.put(1, "a");
        cache.put(2, "b");

        cache.resize(THREE);

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.capacity(), 3);

        cache.resize(FOUR);

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.capacity(), 4);

        cache.put(3, "c");
        cache.put(4, "d");

        assert_eq!(cache.len(), 4);
        assert_eq!(cache.capacity(), 4);
        assert_eq!(cache.get(&1), Some(&"a"));
        assert_eq!(cache.get(&2), Some(&"b"));
        assert_eq!(cache.get(&3), Some(&"c"));
        assert_eq!(cache.get(&4), Some(&"d"));
    }

    #[test]
    fn test_resize_smaller() {
        let mut cache = CLruCache::new(FOUR);

        cache.put(1, "a");
        cache.put(2, "b");
        cache.put(3, "c");
        cache.put(4, "d");

        cache.resize(TWO);

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.capacity(), 2);
        assert!(cache.get(&1).is_none());
        assert!(cache.get(&2).is_none());
        assert_eq!(cache.get(&3), Some(&"c"));
        assert_eq!(cache.get(&4), Some(&"d"));
    }

    #[test]
    fn test_resize_equal() {
        let mut cache = CLruCache::new(FOUR);

        cache.put(1, "a");
        cache.put(2, "b");
        cache.put(3, "c");
        cache.put(4, "d");

        cache.resize(FOUR);

        assert_eq!(cache.len(), 4);
        assert_eq!(cache.capacity(), 4);
        assert_eq!(cache.get(&1), Some(&"a"));
        assert_eq!(cache.get(&2), Some(&"b"));
        assert_eq!(cache.get(&3), Some(&"c"));
        assert_eq!(cache.get(&4), Some(&"d"));
    }

    #[test]
    fn test_iter_forwards() {
        let mut cache = CLruCache::new(THREE);
        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);

        {
            // iter const
            let mut iter = cache.iter();
            assert_eq!(iter.len(), 3);
            assert_eq!(iter.next(), Some((&"c", &3)));

            assert_eq!(iter.len(), 2);
            assert_eq!(iter.next(), Some((&"b", &2)));

            assert_eq!(iter.len(), 1);
            assert_eq!(iter.next(), Some((&"a", &1)));

            assert_eq!(iter.len(), 0);
            assert_eq!(iter.next(), None);
        }
        {
            // iter mut
            let mut iter = cache.iter_mut();
            assert_eq!(iter.len(), 3);
            assert_eq!(iter.next(), Some((&"c", &mut 3)));

            assert_eq!(iter.len(), 2);
            assert_eq!(iter.next(), Some((&"b", &mut 2)));

            assert_eq!(iter.len(), 1);
            assert_eq!(iter.next(), Some((&"a", &mut 1)));

            assert_eq!(iter.len(), 0);
            assert_eq!(iter.next(), None);

            let mut vec: Vec<_> = cache.iter_mut().collect();
            vec.iter_mut().for_each(|(_, v)| {
                **v -= 1;
            });
            assert_eq!(vec, vec![(&"c", &mut 2), (&"b", &mut 1), (&"a", &mut 0)]);
        }
    }

    #[test]
    fn test_iter_backwards() {
        let mut cache = CLruCache::new(THREE);
        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);

        {
            // iter const
            let mut iter = cache.iter();
            assert_eq!(iter.len(), 3);
            assert_eq!(iter.next_back(), Some((&"a", &1)));

            assert_eq!(iter.len(), 2);
            assert_eq!(iter.next_back(), Some((&"b", &2)));

            assert_eq!(iter.len(), 1);
            assert_eq!(iter.next_back(), Some((&"c", &3)));

            assert_eq!(iter.len(), 0);
            assert_eq!(iter.next_back(), None);
        }

        {
            // iter mut
            let mut iter = cache.iter_mut();
            assert_eq!(iter.len(), 3);
            assert_eq!(iter.next_back(), Some((&"a", &mut 1)));

            assert_eq!(iter.len(), 2);
            assert_eq!(iter.next_back(), Some((&"b", &mut 2)));

            assert_eq!(iter.len(), 1);
            assert_eq!(iter.next_back(), Some((&"c", &mut 3)));

            assert_eq!(iter.len(), 0);
            assert_eq!(iter.next_back(), None);

            let mut vec: Vec<_> = cache.iter_mut().rev().collect();
            vec.iter_mut().for_each(|(_, v)| {
                **v -= 1;
            });
            assert_eq!(vec, vec![(&"a", &mut 0), (&"b", &mut 1), (&"c", &mut 2)]);
        }
    }

    #[test]
    fn test_iter_forwards_and_backwards() {
        let mut cache = CLruCache::new(THREE);
        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);

        {
            // iter const
            let mut iter = cache.iter();
            assert_eq!(iter.len(), 3);
            assert_eq!(iter.next(), Some((&"c", &3)));

            assert_eq!(iter.len(), 2);
            assert_eq!(iter.next_back(), Some((&"a", &1)));

            assert_eq!(iter.len(), 1);
            assert_eq!(iter.next(), Some((&"b", &2)));

            assert_eq!(iter.len(), 0);
            assert_eq!(iter.next_back(), None);
        }
        {
            // iter mut
            let mut iter = cache.iter_mut();
            assert_eq!(iter.len(), 3);
            assert_eq!(iter.next(), Some((&"c", &mut 3)));

            assert_eq!(iter.len(), 2);
            assert_eq!(iter.next_back(), Some((&"a", &mut 1)));

            assert_eq!(iter.len(), 1);
            assert_eq!(iter.next(), Some((&"b", &mut 2)));

            assert_eq!(iter.len(), 0);
            assert_eq!(iter.next_back(), None);
        }
    }

    #[test]
    fn test_iter_clone() {
        let mut cache = CLruCache::new(THREE);
        cache.put("a", 1);
        cache.put("b", 2);

        let mut iter = cache.iter();
        let mut iter_clone = iter.clone();

        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next(), Some((&"b", &2)));
        assert_eq!(iter_clone.len(), 2);
        assert_eq!(iter_clone.next(), Some((&"b", &2)));

        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some((&"a", &1)));
        assert_eq!(iter_clone.len(), 1);
        assert_eq!(iter_clone.next(), Some((&"a", &1)));

        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
        assert_eq!(iter_clone.len(), 0);
        assert_eq!(iter_clone.next(), None);
    }

    #[test]
    fn test_that_pop_actually_detaches_node() {
        let mut cache = CLruCache::new(FIVE);

        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);
        cache.put("d", 4);
        cache.put("e", 5);

        assert_eq!(cache.pop(&"c"), Some(3));

        cache.put("f", 6);

        let mut iter = cache.iter();
        assert_eq!(iter.next(), Some((&"f", &6)));
        assert_eq!(iter.next(), Some((&"e", &5)));
        assert_eq!(iter.next(), Some((&"d", &4)));
        assert_eq!(iter.next(), Some((&"b", &2)));
        assert_eq!(iter.next(), Some((&"a", &1)));
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_get_with_borrow() {
        let mut cache = CLruCache::new(TWO);

        let key = String::from("apple");
        cache.put(key, "red");

        assert_eq!(cache.get("apple"), Some(&"red"));
    }

    #[test]
    fn test_get_mut_with_borrow() {
        let mut cache = CLruCache::new(TWO);

        let key = String::from("apple");
        cache.put(key, "red");

        assert_eq!(cache.get_mut("apple"), Some(&mut "red"));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_no_memory_leaks() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        struct DropCounter;

        impl Drop for DropCounter {
            fn drop(&mut self) {
                DROP_COUNT.fetch_add(1, Ordering::SeqCst);
            }
        }

        let n = 100;
        for _ in 0..n {
            let mut cache = CLruCache::new(ONE);
            for i in 0..n {
                cache.put(i, DropCounter {});
            }
        }
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), n * n);
    }

    #[test]
    fn test_retain() {
        let mut cache = CLruCache::new(FIVE);

        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);
        cache.put("d", 4);
        cache.put("e", 5);

        assert_eq!(cache.len(), 5);

        cache.retain_mut(|k, v| match *k {
            "b" | "d" => false,
            _ => {
                *v += 1;
                true
            }
        });

        assert_eq!(cache.len(), 3);

        assert_eq!(cache.get("a"), Some(&2));
        assert_eq!(cache.get("b"), None);
        assert_eq!(cache.get("c"), Some(&4));
        assert_eq!(cache.get("d"), None);
        assert_eq!(cache.get("e"), Some(&6));

        cache.retain(|_, _| true);

        assert_eq!(cache.len(), 3);

        assert_eq!(cache.get("a"), Some(&2));
        assert_eq!(cache.get("b"), None);
        assert_eq!(cache.get("c"), Some(&4));
        assert_eq!(cache.get("d"), None);
        assert_eq!(cache.get("e"), Some(&6));

        cache.retain(|_, _| false);

        assert_eq!(cache.len(), 0);

        assert_eq!(cache.get("a"), None);
        assert_eq!(cache.get("b"), None);
        assert_eq!(cache.get("c"), None);
        assert_eq!(cache.get("d"), None);
        assert_eq!(cache.get("e"), None);
    }

    #[test]
    fn test_into_iter() {
        let mut cache = CLruCache::new(FIVE);

        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);
        cache.put("d", 4);
        cache.put("e", 5);

        let mut vec = Vec::new();
        for (k, v) in &cache {
            vec.push((k, v));
        }
        assert_eq!(
            vec,
            vec![(&"e", &5), (&"d", &4), (&"c", &3), (&"b", &2), (&"a", &1)]
        );

        let mut vec = Vec::new();
        for (k, v) in &mut cache {
            *v -= 1;
            vec.push((k, v));
        }
        assert_eq!(
            vec,
            vec![
                (&"e", &mut 4),
                (&"d", &mut 3),
                (&"c", &mut 2),
                (&"b", &mut 1),
                (&"a", &mut 0)
            ]
        );

        assert_eq!(
            cache.into_iter().collect::<Vec<_>>(),
            vec![("e", 4), ("d", 3), ("c", 2), ("b", 1), ("a", 0)]
        );
    }

    #[test]
    fn test_put_or_modify() {
        let mut cache = CLruCache::new(TWO);

        let put = |key: &&str, base: Option<usize>| base.unwrap_or(0) + key.len();

        let modify = |key: &&str, value: &mut usize, base: Option<usize>| {
            if key.len() == *value {
                *value *= 2;
            } else {
                *value /= 2;
            }
            *value += base.unwrap_or(0);
        };

        assert_eq!(cache.put_or_modify("a", put, modify, None), &1);
        assert_eq!(cache.len(), 1);

        let mut iter = cache.iter();
        assert_eq!(iter.next(), Some((&"a", &1)));
        assert_eq!(iter.next(), None);

        assert_eq!(cache.put_or_modify("b", put, modify, Some(3)), &4);
        assert_eq!(cache.len(), 2);

        let mut iter = cache.iter();
        assert_eq!(iter.next(), Some((&"b", &4)));
        assert_eq!(iter.next(), Some((&"a", &1)));
        assert_eq!(iter.next(), None);

        assert_eq!(cache.put_or_modify("a", put, modify, None), &2);
        assert_eq!(cache.len(), 2);

        let mut iter = cache.iter();
        assert_eq!(iter.next(), Some((&"a", &2)));
        assert_eq!(iter.next(), Some((&"b", &4)));
        assert_eq!(iter.next(), None);

        assert_eq!(cache.put_or_modify("b", put, modify, Some(3)), &5);
        assert_eq!(cache.len(), 2);

        let mut iter = cache.iter();
        assert_eq!(iter.next(), Some((&"b", &5)));
        assert_eq!(iter.next(), Some((&"a", &2)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_panic_in_put_or_modify() {
        use std::panic::{catch_unwind, AssertUnwindSafe};

        let mut cache = CLruCache::new(TWO);

        let put = |_: &&str, value: usize| value;

        let modify = |_: &&str, old: &mut usize, new: usize| {
            panic!("old value: {:?} / new value: {:?}", old, new);
        };

        assert_eq!(cache.put_or_modify("a", put, modify, 3), &3);

        assert_eq!(cache.put_or_modify("b", put, modify, 5), &5);

        let mut iter = cache.iter();
        assert_eq!(iter.next(), Some((&"b", &5)));
        assert_eq!(iter.next(), Some((&"a", &3)));
        assert_eq!(iter.next(), None);

        // A panic in the modify closure will move the
        // key at the top of the cache.
        assert!(catch_unwind(AssertUnwindSafe(|| {
            cache.put_or_modify("a", put, modify, 7);
        }))
        .is_err());

        let mut iter = cache.iter();
        assert_eq!(iter.next(), Some((&"a", &3)));
        assert_eq!(iter.next(), Some((&"b", &5)));
        assert_eq!(iter.next(), None);

        let put = |_: &&str, value: usize| panic!("value: {:?}", value);

        // A panic in the put closure won't have any
        // any impact on the cache.
        assert!(catch_unwind(AssertUnwindSafe(|| {
            cache.put_or_modify("c", put, modify, 7);
        }))
        .is_err());

        let mut iter = cache.iter();
        assert_eq!(iter.next(), Some((&"a", &3)));
        assert_eq!(iter.next(), Some((&"b", &5)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_try_put_or_modify() {
        let mut cache = CLruCache::new(TWO);

        let put = |_: &&str, value: usize| {
            if value % 2 == 0 {
                Ok(value)
            } else {
                Err(value)
            }
        };

        let modify = |_: &&str, old: &mut usize, new: usize| {
            if new % 2 == 0 {
                *old = new;
                Ok(())
            } else {
                Err(new)
            }
        };

        assert_eq!(cache.try_put_or_modify("a", put, modify, 2), Ok(&mut 2));
        assert_eq!(cache.len(), 1);

        let mut iter = cache.iter();
        assert_eq!(iter.next(), Some((&"a", &2)));
        assert_eq!(iter.next(), None);

        assert_eq!(cache.try_put_or_modify("b", put, modify, 4), Ok(&mut 4));
        assert_eq!(cache.len(), 2);

        let mut iter = cache.iter();
        assert_eq!(iter.next(), Some((&"b", &4)));
        assert_eq!(iter.next(), Some((&"a", &2)));
        assert_eq!(iter.next(), None);

        // An error in the modify closure will move the
        // key at the top of the cache.
        assert_eq!(cache.try_put_or_modify("a", put, modify, 3), Err(3));
        assert_eq!(cache.len(), 2);

        let mut iter = cache.iter();
        assert_eq!(iter.next(), Some((&"a", &2)));
        assert_eq!(iter.next(), Some((&"b", &4)));
        assert_eq!(iter.next(), None);

        // An error in the put closure won't have any
        // any impact on the cache.
        assert_eq!(cache.try_put_or_modify("c", put, modify, 3), Err(3));
        assert_eq!(cache.len(), 2);

        let mut iter = cache.iter();
        assert_eq!(iter.next(), Some((&"a", &2)));
        assert_eq!(iter.next(), Some((&"b", &4)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_from_iterator() {
        let cache: CLruCache<&'static str, usize> =
            vec![("a", 1), ("b", 2), ("c", 3), ("d", 4), ("e", 5)]
                .into_iter()
                .collect();

        assert_eq!(cache.len(), 5);
        assert_eq!(cache.capacity(), 5);
        assert_eq!(cache.is_full(), true);

        assert_eq!(
            cache.into_iter().collect::<Vec<_>>(),
            vec![("e", 5), ("d", 4), ("c", 3), ("b", 2), ("a", 1)]
        );

        let cache: CLruCache<&'static str, usize> = vec![].into_iter().collect();

        assert_eq!(cache.len(), 0);
        assert_eq!(cache.capacity(), 1);
        assert_eq!(cache.is_full(), false);

        assert_eq!(cache.into_iter().collect::<Vec<_>>(), vec![]);
    }

    #[test]
    fn test_extend() {
        let mut cache = CLruCache::new(FIVE);

        cache.put("a", 1);
        cache.put("b", 2);

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.capacity(), 5);
        assert_eq!(cache.is_full(), false);

        cache.extend(vec![("c", 3), ("d", 4), ("e", 5)].into_iter());

        assert_eq!(cache.len(), 5);
        assert_eq!(cache.capacity(), 5);
        assert_eq!(cache.is_full(), true);

        assert_eq!(
            cache.into_iter().collect::<Vec<_>>(),
            vec![("e", 5), ("d", 4), ("c", 3), ("b", 2), ("a", 1)]
        );
    }

    #[derive(Debug)]
    struct StrStrScale;

    impl WeightScale<&str, &str> for StrStrScale {
        fn weight(&self, _key: &&str, value: &&str) -> usize {
            value.len()
        }
    }

    #[test]
    fn test_weighted_insert_and_get() {
        let mut cache = CLruCache::with_config(
            CLruCacheConfig::new(NonZeroUsize::new(11).unwrap()).with_scale(StrStrScale),
        );
        assert!(cache.is_empty());

        assert_eq!(cache.put_with_weight("apple", "red").unwrap(), None);
        assert_eq!(cache.put_with_weight("banana", "yellow").unwrap(), None);

        assert_eq!(cache.capacity(), 11);
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.weight(), 9);
        assert!(!cache.is_empty());
        assert!(cache.is_full()); // because of weight
        assert_eq!(cache.get(&"apple"), Some(&"red"));
        assert_eq!(cache.get(&"banana"), Some(&"yellow"));
    }

    #[test]
    fn test_zero_weight_fails() {
        let mut cache = CLruCache::with_config(
            CLruCacheConfig::new(NonZeroUsize::new(3).unwrap()).with_scale(StrStrScale),
        );

        assert!(cache.put_with_weight("apple", "red").is_err());
        assert!(cache.put_with_weight("apple", "red").is_err());
    }

    #[test]
    fn test_greater_than_max_weight_fails() {
        let mut cache = CLruCache::with_config(
            CLruCacheConfig::new(NonZeroUsize::new(3).unwrap()).with_scale(StrStrScale),
        );

        assert!(cache.put_with_weight("apple", "red").is_err());
    }

    #[test]
    fn test_weighted_insert_update() {
        let mut cache = CLruCache::with_config(
            CLruCacheConfig::new(NonZeroUsize::new(6).unwrap()).with_scale(StrStrScale),
        );

        assert_eq!(cache.put_with_weight("apple", "red").unwrap(), None);
        assert_eq!(
            cache.put_with_weight("apple", "green").unwrap(),
            Some("red")
        );

        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get(&"apple"), Some(&"green"));
    }

    #[test]
    fn test_weighted_insert_removes_oldest() {
        let mut cache = CLruCache::with_config(
            CLruCacheConfig::new(NonZeroUsize::new(16).unwrap()).with_scale(StrStrScale),
        );

        assert_eq!(cache.put_with_weight("apple", "red").unwrap(), None);
        assert_eq!(cache.put_with_weight("banana", "yellow").unwrap(), None);
        assert_eq!(cache.put_with_weight("pear", "green").unwrap(), None);

        assert!(cache.get(&"apple").is_none());
        assert_eq!(cache.get(&"banana"), Some(&"yellow"));
        assert_eq!(cache.get(&"pear"), Some(&"green"));
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.weight(), 11);
        assert_eq!(cache.capacity(), 16);
        assert!(!cache.is_full());

        // Even though we inserted "apple" into the cache earlier it has since been removed from
        // the cache so there is no current value for `insert` to return.
        assert_eq!(cache.put_with_weight("apple", "green").unwrap(), None);
        assert_eq!(cache.put_with_weight("tomato", "red").unwrap(), None);

        assert_eq!(cache.len(), 3); // tomato, apple, pear
        assert_eq!(cache.weight(), 13); //  3 + 5 + 5
        assert_eq!(cache.capacity(), 16);
        assert!(cache.is_full());

        assert_eq!(cache.get(&"pear"), Some(&"green"));
        assert_eq!(cache.get(&"apple"), Some(&"green"));
        assert_eq!(cache.get(&"tomato"), Some(&"red"));
    }

    #[test]
    fn test_weighted_clear() {
        let mut cache = CLruCache::with_config(
            CLruCacheConfig::new(NonZeroUsize::new(10).unwrap()).with_scale(StrStrScale),
        );

        assert_eq!(cache.put_with_weight("apple", "red"), Ok(None));
        assert_eq!(cache.put_with_weight("banana", "yellow"), Ok(None));

        assert_eq!(cache.len(), 1);
        assert_eq!(cache.weight(), 6);
        assert_eq!(cache.get(&"apple"), None);
        assert_eq!(cache.get(&"banana"), Some(&"yellow"));

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.weight(), 0);
    }

    #[derive(Debug)]
    struct IntStrScale;

    impl WeightScale<usize, &str> for IntStrScale {
        fn weight(&self, _key: &usize, value: &&str) -> usize {
            value.len()
        }
    }

    #[test]
    fn test_weighted_resize_larger() {
        let mut cache = CLruCache::with_config(
            CLruCacheConfig::new(NonZeroUsize::new(4).unwrap()).with_scale(IntStrScale),
        );

        assert_eq!(cache.put_with_weight(1, "a"), Ok(None));
        assert_eq!(cache.put_with_weight(2, "b"), Ok(None));
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.weight(), 2);
        assert_eq!(cache.capacity(), 4);
        assert!(cache.is_full());

        cache.resize(SIX);

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.weight(), 2);
        assert_eq!(cache.capacity(), 6);
        assert!(!cache.is_full());

        cache.resize(HEIGHT);

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.weight(), 2);
        assert_eq!(cache.capacity(), 8);
        assert!(!cache.is_full());

        assert_eq!(cache.put_with_weight(3, "c"), Ok(None));
        assert_eq!(cache.put_with_weight(4, "d"), Ok(None));

        assert_eq!(cache.len(), 4);
        assert_eq!(cache.weight(), 4);
        assert_eq!(cache.capacity(), 8);
        assert!(cache.is_full());
        assert_eq!(cache.get(&1), Some(&"a"));
        assert_eq!(cache.get(&2), Some(&"b"));
        assert_eq!(cache.get(&3), Some(&"c"));
        assert_eq!(cache.get(&4), Some(&"d"));
    }

    #[test]
    fn test_weighted_resize_smaller() {
        let mut cache = CLruCache::with_config(
            CLruCacheConfig::new(NonZeroUsize::new(8).unwrap()).with_scale(IntStrScale),
        );

        assert_eq!(cache.put_with_weight(1, "a"), Ok(None));
        assert_eq!(cache.put_with_weight(2, "b"), Ok(None));
        assert_eq!(cache.put_with_weight(3, "c"), Ok(None));
        assert_eq!(cache.put_with_weight(4, "d"), Ok(None));
        assert_eq!(cache.len(), 4);
        assert_eq!(cache.weight(), 4);
        assert_eq!(cache.capacity(), 8);
        assert!(cache.is_full());

        cache.resize(FOUR);

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.weight(), 2);
        assert_eq!(cache.capacity(), 4);
        assert!(cache.is_full());

        assert!(cache.get(&1).is_none());
        assert!(cache.get(&2).is_none());
        assert_eq!(cache.get(&3), Some(&"c"));
        assert_eq!(cache.get(&4), Some(&"d"));

        cache.resize(THREE);

        assert_eq!(cache.len(), 1);
        assert_eq!(cache.weight(), 1);
        assert_eq!(cache.capacity(), 3);
        assert!(!cache.is_full());

        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&3), None);
        assert_eq!(cache.get(&4), Some(&"d"));
    }

    #[test]
    fn test_weighted_resize_equal() {
        let mut cache = CLruCache::with_config(
            CLruCacheConfig::new(NonZeroUsize::new(8).unwrap()).with_scale(IntStrScale),
        );

        assert_eq!(cache.put_with_weight(1, "a"), Ok(None));
        assert_eq!(cache.put_with_weight(2, "b"), Ok(None));
        assert_eq!(cache.put_with_weight(3, "c"), Ok(None));
        assert_eq!(cache.put_with_weight(4, "d"), Ok(None));

        assert_eq!(cache.len(), 4);
        assert_eq!(cache.weight(), 4);
        assert_eq!(cache.capacity(), 8);
        assert!(cache.is_full());

        cache.resize(HEIGHT);

        assert_eq!(cache.len(), 4);
        assert_eq!(cache.weight(), 4);
        assert_eq!(cache.capacity(), 8);
        assert!(cache.is_full());

        assert_eq!(cache.get(&1), Some(&"a"));
        assert_eq!(cache.get(&2), Some(&"b"));
        assert_eq!(cache.get(&3), Some(&"c"));
        assert_eq!(cache.get(&4), Some(&"d"));
    }

    #[test]
    fn test_weighted_iter() {
        let mut cache = CLruCache::with_config(
            CLruCacheConfig::new(NonZeroUsize::new(8).unwrap()).with_scale(IntStrScale),
        );

        assert_eq!(cache.put_with_weight(1, "a"), Ok(None));
        assert_eq!(cache.put_with_weight(2, "b"), Ok(None));
        assert_eq!(cache.put_with_weight(3, "c"), Ok(None));
        assert_eq!(cache.put_with_weight(4, "d"), Ok(None));

        assert_eq!(cache.len(), 4);
        assert_eq!(cache.weight(), 4);
        assert_eq!(cache.capacity(), 8);
        assert!(cache.is_full());
    }

    #[test]
    fn test_weighted_iter_forwards() {
        let mut cache = CLruCache::with_config(
            CLruCacheConfig::new(NonZeroUsize::new(8).unwrap()).with_scale(IntStrScale),
        );
        assert_eq!(cache.put_with_weight(1, "a"), Ok(None));
        assert_eq!(cache.put_with_weight(2, "b"), Ok(None));
        assert_eq!(cache.put_with_weight(3, "c"), Ok(None));

        let mut iter = cache.iter();
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next(), Some((&3, &"c")));

        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next(), Some((&2, &"b")));

        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some((&1, &"a")));

        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_weighted_iter_backwards() {
        let mut cache = CLruCache::with_config(
            CLruCacheConfig::new(NonZeroUsize::new(8).unwrap()).with_scale(IntStrScale),
        );
        assert_eq!(cache.put_with_weight(1, "a"), Ok(None));
        assert_eq!(cache.put_with_weight(2, "b"), Ok(None));
        assert_eq!(cache.put_with_weight(3, "c"), Ok(None));

        let mut iter = cache.iter();
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next_back(), Some((&1, &"a")));

        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next_back(), Some((&2, &"b")));

        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next_back(), Some((&3, &"c")));

        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_weighted_iter_forwards_and_backwards() {
        let mut cache = CLruCache::with_config(
            CLruCacheConfig::new(NonZeroUsize::new(8).unwrap()).with_scale(IntStrScale),
        );
        assert_eq!(cache.put_with_weight(1, "a"), Ok(None));
        assert_eq!(cache.put_with_weight(2, "b"), Ok(None));
        assert_eq!(cache.put_with_weight(3, "c"), Ok(None));

        let mut iter = cache.iter();
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next(), Some((&3, &"c")));

        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next_back(), Some((&1, &"a")));

        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some((&2, &"b")));

        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_weighted_into_iter() {
        let mut cache = CLruCache::with_config(
            CLruCacheConfig::new(NonZeroUsize::new(10).unwrap()).with_scale(IntStrScale),
        );

        assert_eq!(cache.put_with_weight(1, "a"), Ok(None));
        assert_eq!(cache.put_with_weight(2, "b"), Ok(None));
        assert_eq!(cache.put_with_weight(3, "c"), Ok(None));
        assert_eq!(cache.put_with_weight(4, "d"), Ok(None));
        assert_eq!(cache.put_with_weight(5, "e"), Ok(None));

        let mut vec = Vec::new();
        for (k, v) in &cache {
            vec.push((k, v));
        }
        assert_eq!(
            vec,
            vec![(&5, &"e"), (&4, &"d"), (&3, &"c"), (&2, &"b"), (&1, &"a")]
        );

        assert_eq!(
            cache.into_iter().collect::<Vec<_>>(),
            vec![(5, "e"), (4, "d"), (3, "c"), (2, "b"), (1, "a")]
        );
    }

    #[test]
    fn test_is_send() {
        fn is_send<T: Send>() {}

        fn cache_is_send<K: Send, V: Send, S: Send, W: WeightScale<K, V> + Send>() {
            is_send::<K>();
            is_send::<V>();
            is_send::<S>();
            is_send::<W>();
            is_send::<CLruCache<K, V, S, W>>();
        }

        cache_is_send::<String, String, RandomState, ZeroWeightScale>();

        fn cache_in_mutex<
            K: Clone + Default + Eq + Hash + Send + 'static,
            V: Default + Send + 'static,
            S: BuildHasher + Send + 'static,
            W: WeightScale<K, V> + Send + 'static,
        >(
            cache: CLruCache<K, V, S, W>,
        ) where
            (K, V): std::fmt::Debug,
        {
            use std::sync::{Arc, Mutex};
            use std::thread;

            let mutex: Arc<Mutex<CLruCache<K, V, S, W>>> = Arc::new(Mutex::new(cache));
            let mutex2 = mutex.clone();
            let t1 = thread::spawn(move || {
                mutex
                    .lock()
                    .unwrap()
                    .put_with_weight(Default::default(), Default::default())
                    .unwrap();
            });
            let t2 = thread::spawn(move || {
                mutex2
                    .lock()
                    .unwrap()
                    .put_with_weight(Default::default(), Default::default())
                    .unwrap();
            });
            t1.join().unwrap();
            t2.join().unwrap();
        }

        let cache: CLruCache<String, String> = CLruCache::new(TWO);
        cache_in_mutex(cache);
    }

    #[test]
    fn test_is_sync() {
        fn is_sync<T: Sync>() {}

        fn cache_is_sync<K: Sync, V: Sync, S: Sync, W: WeightScale<K, V> + Sync>() {
            is_sync::<K>();
            is_sync::<V>();
            is_sync::<S>();
            is_sync::<W>();
            is_sync::<CLruCache<K, V, S, W>>();
        }

        cache_is_sync::<String, String, RandomState, ZeroWeightScale>();

        fn cache_in_rwlock<
            K: Clone + Default + Eq + Hash + Send + Sync + 'static,
            V: Default + Send + Sync + 'static,
            S: BuildHasher + Send + Sync + 'static,
            W: WeightScale<K, V> + Send + Sync + 'static,
        >(
            cache: CLruCache<K, V, S, W>,
        ) where
            (K, V): std::fmt::Debug,
        {
            use std::sync::{Arc, RwLock};
            use std::thread;

            let mutex: Arc<RwLock<CLruCache<K, V, S, W>>> = Arc::new(RwLock::new(cache));
            let mutex2 = mutex.clone();
            let t1 = thread::spawn(move || {
                mutex
                    .write()
                    .unwrap()
                    .put_with_weight(Default::default(), Default::default())
                    .unwrap();
            });
            let t2 = thread::spawn(move || {
                mutex2
                    .write()
                    .unwrap()
                    .put_with_weight(Default::default(), Default::default())
                    .unwrap();
            });
            t1.join().unwrap();
            t2.join().unwrap();
        }

        let cache: CLruCache<String, String> = CLruCache::new(TWO);
        cache_in_rwlock(cache);
    }
}
