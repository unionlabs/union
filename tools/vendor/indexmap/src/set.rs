//! A hash set implemented using `IndexMap`

mod iter;
mod slice;

#[cfg(test)]
mod tests;

pub use self::iter::{Difference, Drain, Intersection, IntoIter, Iter, SymmetricDifference, Union};
pub use self::slice::Slice;

#[cfg(feature = "rayon")]
pub use crate::rayon::set as rayon;
use crate::TryReserveError;

#[cfg(feature = "std")]
use std::collections::hash_map::RandomState;

use crate::util::try_simplify_range;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{BuildHasher, Hash};
use core::ops::{BitAnd, BitOr, BitXor, Index, RangeBounds, Sub};

use super::{Entries, Equivalent, IndexMap};

type Bucket<T> = super::Bucket<T, ()>;

/// A hash set where the iteration order of the values is independent of their
/// hash values.
///
/// The interface is closely compatible with the standard `HashSet`, but also
/// has additional features.
///
/// # Order
///
/// The values have a consistent order that is determined by the sequence of
/// insertion and removal calls on the set. The order does not depend on the
/// values or the hash function at all. Note that insertion order and value
/// are not affected if a re-insertion is attempted once an element is
/// already present.
///
/// All iterators traverse the set *in order*.  Set operation iterators like
/// `union` produce a concatenated order, as do their matching "bitwise"
/// operators.  See their documentation for specifics.
///
/// The insertion order is preserved, with **notable exceptions** like the
/// `.remove()` or `.swap_remove()` methods. Methods such as `.sort_by()` of
/// course result in a new order, depending on the sorting order.
///
/// # Indices
///
/// The values are indexed in a compact range without holes in the range
/// `0..self.len()`. For example, the method `.get_full` looks up the index for
/// a value, and the method `.get_index` looks up the value by index.
///
/// # Examples
///
/// ```
/// use indexmap::IndexSet;
///
/// // Collects which letters appear in a sentence.
/// let letters: IndexSet<_> = "a short treatise on fungi".chars().collect();
///
/// assert!(letters.contains(&'s'));
/// assert!(letters.contains(&'t'));
/// assert!(letters.contains(&'u'));
/// assert!(!letters.contains(&'y'));
/// ```
#[cfg(feature = "std")]
pub struct IndexSet<T, S = RandomState> {
    pub(crate) map: IndexMap<T, (), S>,
}
#[cfg(not(feature = "std"))]
pub struct IndexSet<T, S> {
    pub(crate) map: IndexMap<T, (), S>,
}

impl<T, S> Clone for IndexSet<T, S>
where
    T: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        IndexSet {
            map: self.map.clone(),
        }
    }

    fn clone_from(&mut self, other: &Self) {
        self.map.clone_from(&other.map);
    }
}

impl<T, S> Entries for IndexSet<T, S> {
    type Entry = Bucket<T>;

    #[inline]
    fn into_entries(self) -> Vec<Self::Entry> {
        self.map.into_entries()
    }

    #[inline]
    fn as_entries(&self) -> &[Self::Entry] {
        self.map.as_entries()
    }

    #[inline]
    fn as_entries_mut(&mut self) -> &mut [Self::Entry] {
        self.map.as_entries_mut()
    }

    fn with_entries<F>(&mut self, f: F)
    where
        F: FnOnce(&mut [Self::Entry]),
    {
        self.map.with_entries(f);
    }
}

impl<T, S> fmt::Debug for IndexSet<T, S>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if cfg!(not(feature = "test_debug")) {
            f.debug_set().entries(self.iter()).finish()
        } else {
            // Let the inner `IndexMap` print all of its details
            f.debug_struct("IndexSet").field("map", &self.map).finish()
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<T> IndexSet<T> {
    /// Create a new set. (Does not allocate.)
    pub fn new() -> Self {
        IndexSet {
            map: IndexMap::new(),
        }
    }

    /// Create a new set with capacity for `n` elements.
    /// (Does not allocate if `n` is zero.)
    ///
    /// Computes in **O(n)** time.
    pub fn with_capacity(n: usize) -> Self {
        IndexSet {
            map: IndexMap::with_capacity(n),
        }
    }
}

impl<T, S> IndexSet<T, S> {
    /// Create a new set with capacity for `n` elements.
    /// (Does not allocate if `n` is zero.)
    ///
    /// Computes in **O(n)** time.
    pub fn with_capacity_and_hasher(n: usize, hash_builder: S) -> Self {
        IndexSet {
            map: IndexMap::with_capacity_and_hasher(n, hash_builder),
        }
    }

    /// Create a new set with `hash_builder`.
    ///
    /// This function is `const`, so it
    /// can be called in `static` contexts.
    pub const fn with_hasher(hash_builder: S) -> Self {
        IndexSet {
            map: IndexMap::with_hasher(hash_builder),
        }
    }

    /// Return the number of elements the set can hold without reallocating.
    ///
    /// This number is a lower bound; the set might be able to hold more,
    /// but is guaranteed to be able to hold at least this many.
    ///
    /// Computes in **O(1)** time.
    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }

    /// Return a reference to the set's `BuildHasher`.
    pub fn hasher(&self) -> &S {
        self.map.hasher()
    }

    /// Return the number of elements in the set.
    ///
    /// Computes in **O(1)** time.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns true if the set contains no elements.
    ///
    /// Computes in **O(1)** time.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Return an iterator over the values of the set, in their order
    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(self.as_entries())
    }

    /// Remove all elements in the set, while preserving its capacity.
    ///
    /// Computes in **O(n)** time.
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Shortens the set, keeping the first `len` elements and dropping the rest.
    ///
    /// If `len` is greater than the set's current length, this has no effect.
    pub fn truncate(&mut self, len: usize) {
        self.map.truncate(len);
    }

    /// Clears the `IndexSet` in the given index range, returning those values
    /// as a drain iterator.
    ///
    /// The range may be any type that implements `RangeBounds<usize>`,
    /// including all of the `std::ops::Range*` types, or even a tuple pair of
    /// `Bound` start and end values. To drain the set entirely, use `RangeFull`
    /// like `set.drain(..)`.
    ///
    /// This shifts down all entries following the drained range to fill the
    /// gap, and keeps the allocated memory for reuse.
    ///
    /// ***Panics*** if the starting point is greater than the end point or if
    /// the end point is greater than the length of the set.
    pub fn drain<R>(&mut self, range: R) -> Drain<'_, T>
    where
        R: RangeBounds<usize>,
    {
        Drain::new(self.map.core.drain(range))
    }

    /// Splits the collection into two at the given index.
    ///
    /// Returns a newly allocated set containing the elements in the range
    /// `[at, len)`. After the call, the original set will be left containing
    /// the elements `[0, at)` with its previous capacity unchanged.
    ///
    /// ***Panics*** if `at > len`.
    pub fn split_off(&mut self, at: usize) -> Self
    where
        S: Clone,
    {
        Self {
            map: self.map.split_off(at),
        }
    }
}

impl<T, S> IndexSet<T, S>
where
    T: Hash + Eq,
    S: BuildHasher,
{
    /// Reserve capacity for `additional` more values.
    ///
    /// Computes in **O(n)** time.
    pub fn reserve(&mut self, additional: usize) {
        self.map.reserve(additional);
    }

    /// Reserve capacity for `additional` more values, without over-allocating.
    ///
    /// Unlike `reserve`, this does not deliberately over-allocate the entry capacity to avoid
    /// frequent re-allocations. However, the underlying data structures may still have internal
    /// capacity requirements, and the allocator itself may give more space than requested, so this
    /// cannot be relied upon to be precisely minimal.
    ///
    /// Computes in **O(n)** time.
    pub fn reserve_exact(&mut self, additional: usize) {
        self.map.reserve_exact(additional);
    }

    /// Try to reserve capacity for `additional` more values.
    ///
    /// Computes in **O(n)** time.
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.map.try_reserve(additional)
    }

    /// Try to reserve capacity for `additional` more values, without over-allocating.
    ///
    /// Unlike `try_reserve`, this does not deliberately over-allocate the entry capacity to avoid
    /// frequent re-allocations. However, the underlying data structures may still have internal
    /// capacity requirements, and the allocator itself may give more space than requested, so this
    /// cannot be relied upon to be precisely minimal.
    ///
    /// Computes in **O(n)** time.
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.map.try_reserve_exact(additional)
    }

    /// Shrink the capacity of the set as much as possible.
    ///
    /// Computes in **O(n)** time.
    pub fn shrink_to_fit(&mut self) {
        self.map.shrink_to_fit();
    }

    /// Shrink the capacity of the set with a lower limit.
    ///
    /// Computes in **O(n)** time.
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.map.shrink_to(min_capacity);
    }

    /// Insert the value into the set.
    ///
    /// If an equivalent item already exists in the set, it returns
    /// `false` leaving the original value in the set and without
    /// altering its insertion order. Otherwise, it inserts the new
    /// item and returns `true`.
    ///
    /// Computes in **O(1)** time (amortized average).
    pub fn insert(&mut self, value: T) -> bool {
        self.map.insert(value, ()).is_none()
    }

    /// Insert the value into the set, and get its index.
    ///
    /// If an equivalent item already exists in the set, it returns
    /// the index of the existing item and `false`, leaving the
    /// original value in the set and without altering its insertion
    /// order. Otherwise, it inserts the new item and returns the index
    /// of the inserted item and `true`.
    ///
    /// Computes in **O(1)** time (amortized average).
    pub fn insert_full(&mut self, value: T) -> (usize, bool) {
        let (index, existing) = self.map.insert_full(value, ());
        (index, existing.is_none())
    }

    /// Return an iterator over the values that are in `self` but not `other`.
    ///
    /// Values are produced in the same order that they appear in `self`.
    pub fn difference<'a, S2>(&'a self, other: &'a IndexSet<T, S2>) -> Difference<'a, T, S2>
    where
        S2: BuildHasher,
    {
        Difference::new(self, other)
    }

    /// Return an iterator over the values that are in `self` or `other`,
    /// but not in both.
    ///
    /// Values from `self` are produced in their original order, followed by
    /// values from `other` in their original order.
    pub fn symmetric_difference<'a, S2>(
        &'a self,
        other: &'a IndexSet<T, S2>,
    ) -> SymmetricDifference<'a, T, S, S2>
    where
        S2: BuildHasher,
    {
        SymmetricDifference::new(self, other)
    }

    /// Return an iterator over the values that are in both `self` and `other`.
    ///
    /// Values are produced in the same order that they appear in `self`.
    pub fn intersection<'a, S2>(&'a self, other: &'a IndexSet<T, S2>) -> Intersection<'a, T, S2>
    where
        S2: BuildHasher,
    {
        Intersection::new(self, other)
    }

    /// Return an iterator over all values that are in `self` or `other`.
    ///
    /// Values from `self` are produced in their original order, followed by
    /// values that are unique to `other` in their original order.
    pub fn union<'a, S2>(&'a self, other: &'a IndexSet<T, S2>) -> Union<'a, T, S>
    where
        S2: BuildHasher,
    {
        Union::new(self, other)
    }

    /// Return `true` if an equivalent to `value` exists in the set.
    ///
    /// Computes in **O(1)** time (average).
    pub fn contains<Q: ?Sized>(&self, value: &Q) -> bool
    where
        Q: Hash + Equivalent<T>,
    {
        self.map.contains_key(value)
    }

    /// Return a reference to the value stored in the set, if it is present,
    /// else `None`.
    ///
    /// Computes in **O(1)** time (average).
    pub fn get<Q: ?Sized>(&self, value: &Q) -> Option<&T>
    where
        Q: Hash + Equivalent<T>,
    {
        self.map.get_key_value(value).map(|(x, &())| x)
    }

    /// Return item index and value
    pub fn get_full<Q: ?Sized>(&self, value: &Q) -> Option<(usize, &T)>
    where
        Q: Hash + Equivalent<T>,
    {
        self.map.get_full(value).map(|(i, x, &())| (i, x))
    }

    /// Return item index, if it exists in the set
    pub fn get_index_of<Q: ?Sized>(&self, value: &Q) -> Option<usize>
    where
        Q: Hash + Equivalent<T>,
    {
        self.map.get_index_of(value)
    }

    /// Adds a value to the set, replacing the existing value, if any, that is
    /// equal to the given one, without altering its insertion order. Returns
    /// the replaced value.
    ///
    /// Computes in **O(1)** time (average).
    pub fn replace(&mut self, value: T) -> Option<T> {
        self.replace_full(value).1
    }

    /// Adds a value to the set, replacing the existing value, if any, that is
    /// equal to the given one, without altering its insertion order. Returns
    /// the index of the item and its replaced value.
    ///
    /// Computes in **O(1)** time (average).
    pub fn replace_full(&mut self, value: T) -> (usize, Option<T>) {
        use super::map::Entry::*;

        match self.map.entry(value) {
            Vacant(e) => {
                let index = e.index();
                e.insert(());
                (index, None)
            }
            Occupied(e) => (e.index(), Some(e.replace_key())),
        }
    }

    /// Remove the value from the set, and return `true` if it was present.
    ///
    /// **NOTE:** This is equivalent to `.swap_remove(value)`, if you want
    /// to preserve the order of the values in the set, use `.shift_remove(value)`.
    ///
    /// Computes in **O(1)** time (average).
    pub fn remove<Q: ?Sized>(&mut self, value: &Q) -> bool
    where
        Q: Hash + Equivalent<T>,
    {
        self.swap_remove(value)
    }

    /// Remove the value from the set, and return `true` if it was present.
    ///
    /// Like `Vec::swap_remove`, the value is removed by swapping it with the
    /// last element of the set and popping it off. **This perturbs
    /// the position of what used to be the last element!**
    ///
    /// Return `false` if `value` was not in the set.
    ///
    /// Computes in **O(1)** time (average).
    pub fn swap_remove<Q: ?Sized>(&mut self, value: &Q) -> bool
    where
        Q: Hash + Equivalent<T>,
    {
        self.map.swap_remove(value).is_some()
    }

    /// Remove the value from the set, and return `true` if it was present.
    ///
    /// Like `Vec::remove`, the value is removed by shifting all of the
    /// elements that follow it, preserving their relative order.
    /// **This perturbs the index of all of those elements!**
    ///
    /// Return `false` if `value` was not in the set.
    ///
    /// Computes in **O(n)** time (average).
    pub fn shift_remove<Q: ?Sized>(&mut self, value: &Q) -> bool
    where
        Q: Hash + Equivalent<T>,
    {
        self.map.shift_remove(value).is_some()
    }

    /// Removes and returns the value in the set, if any, that is equal to the
    /// given one.
    ///
    /// **NOTE:** This is equivalent to `.swap_take(value)`, if you need to
    /// preserve the order of the values in the set, use `.shift_take(value)`
    /// instead.
    ///
    /// Computes in **O(1)** time (average).
    pub fn take<Q: ?Sized>(&mut self, value: &Q) -> Option<T>
    where
        Q: Hash + Equivalent<T>,
    {
        self.swap_take(value)
    }

    /// Removes and returns the value in the set, if any, that is equal to the
    /// given one.
    ///
    /// Like `Vec::swap_remove`, the value is removed by swapping it with the
    /// last element of the set and popping it off. **This perturbs
    /// the position of what used to be the last element!**
    ///
    /// Return `None` if `value` was not in the set.
    ///
    /// Computes in **O(1)** time (average).
    pub fn swap_take<Q: ?Sized>(&mut self, value: &Q) -> Option<T>
    where
        Q: Hash + Equivalent<T>,
    {
        self.map.swap_remove_entry(value).map(|(x, ())| x)
    }

    /// Removes and returns the value in the set, if any, that is equal to the
    /// given one.
    ///
    /// Like `Vec::remove`, the value is removed by shifting all of the
    /// elements that follow it, preserving their relative order.
    /// **This perturbs the index of all of those elements!**
    ///
    /// Return `None` if `value` was not in the set.
    ///
    /// Computes in **O(n)** time (average).
    pub fn shift_take<Q: ?Sized>(&mut self, value: &Q) -> Option<T>
    where
        Q: Hash + Equivalent<T>,
    {
        self.map.shift_remove_entry(value).map(|(x, ())| x)
    }

    /// Remove the value from the set return it and the index it had.
    ///
    /// Like `Vec::swap_remove`, the value is removed by swapping it with the
    /// last element of the set and popping it off. **This perturbs
    /// the position of what used to be the last element!**
    ///
    /// Return `None` if `value` was not in the set.
    pub fn swap_remove_full<Q: ?Sized>(&mut self, value: &Q) -> Option<(usize, T)>
    where
        Q: Hash + Equivalent<T>,
    {
        self.map.swap_remove_full(value).map(|(i, x, ())| (i, x))
    }

    /// Remove the value from the set return it and the index it had.
    ///
    /// Like `Vec::remove`, the value is removed by shifting all of the
    /// elements that follow it, preserving their relative order.
    /// **This perturbs the index of all of those elements!**
    ///
    /// Return `None` if `value` was not in the set.
    pub fn shift_remove_full<Q: ?Sized>(&mut self, value: &Q) -> Option<(usize, T)>
    where
        Q: Hash + Equivalent<T>,
    {
        self.map.shift_remove_full(value).map(|(i, x, ())| (i, x))
    }

    /// Remove the last value
    ///
    /// This preserves the order of the remaining elements.
    ///
    /// Computes in **O(1)** time (average).
    pub fn pop(&mut self) -> Option<T> {
        self.map.pop().map(|(x, ())| x)
    }

    /// Scan through each value in the set and keep those where the
    /// closure `keep` returns `true`.
    ///
    /// The elements are visited in order, and remaining elements keep their
    /// order.
    ///
    /// Computes in **O(n)** time (average).
    pub fn retain<F>(&mut self, mut keep: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.map.retain(move |x, &mut ()| keep(x))
    }

    /// Sort the set’s values by their default ordering.
    ///
    /// See [`sort_by`](Self::sort_by) for details.
    pub fn sort(&mut self)
    where
        T: Ord,
    {
        self.map.sort_keys()
    }

    /// Sort the set’s values in place using the comparison function `cmp`.
    ///
    /// Computes in **O(n log n)** time and **O(n)** space. The sort is stable.
    pub fn sort_by<F>(&mut self, mut cmp: F)
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        self.map.sort_by(move |a, _, b, _| cmp(a, b));
    }

    /// Sort the values of the set and return a by-value iterator of
    /// the values with the result.
    ///
    /// The sort is stable.
    pub fn sorted_by<F>(self, mut cmp: F) -> IntoIter<T>
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        let mut entries = self.into_entries();
        entries.sort_by(move |a, b| cmp(&a.key, &b.key));
        IntoIter::new(entries)
    }

    /// Sort the set's values by their default ordering.
    ///
    /// See [`sort_unstable_by`](Self::sort_unstable_by) for details.
    pub fn sort_unstable(&mut self)
    where
        T: Ord,
    {
        self.map.sort_unstable_keys()
    }

    /// Sort the set's values in place using the comparison function `cmp`.
    ///
    /// Computes in **O(n log n)** time. The sort is unstable.
    pub fn sort_unstable_by<F>(&mut self, mut cmp: F)
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        self.map.sort_unstable_by(move |a, _, b, _| cmp(a, b))
    }

    /// Sort the values of the set and return a by-value iterator of
    /// the values with the result.
    pub fn sorted_unstable_by<F>(self, mut cmp: F) -> IntoIter<T>
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        let mut entries = self.into_entries();
        entries.sort_unstable_by(move |a, b| cmp(&a.key, &b.key));
        IntoIter::new(entries)
    }

    /// Sort the set’s values in place using a key extraction function.
    ///
    /// During sorting, the function is called at most once per entry, by using temporary storage
    /// to remember the results of its evaluation. The order of calls to the function is
    /// unspecified and may change between versions of `indexmap` or the standard library.
    ///
    /// Computes in **O(m n + n log n + c)** time () and **O(n)** space, where the function is
    /// **O(m)**, *n* is the length of the map, and *c* the capacity. The sort is stable.
    pub fn sort_by_cached_key<K, F>(&mut self, mut sort_key: F)
    where
        K: Ord,
        F: FnMut(&T) -> K,
    {
        self.with_entries(move |entries| {
            entries.sort_by_cached_key(move |a| sort_key(&a.key));
        });
    }

    /// Reverses the order of the set’s values in place.
    ///
    /// Computes in **O(n)** time and **O(1)** space.
    pub fn reverse(&mut self) {
        self.map.reverse()
    }
}

impl<T, S> IndexSet<T, S> {
    /// Returns a slice of all the values in the set.
    ///
    /// Computes in **O(1)** time.
    pub fn as_slice(&self) -> &Slice<T> {
        Slice::from_slice(self.as_entries())
    }

    /// Converts into a boxed slice of all the values in the set.
    ///
    /// Note that this will drop the inner hash table and any excess capacity.
    pub fn into_boxed_slice(self) -> Box<Slice<T>> {
        Slice::from_boxed(self.into_entries().into_boxed_slice())
    }

    /// Get a value by index
    ///
    /// Valid indices are *0 <= index < self.len()*
    ///
    /// Computes in **O(1)** time.
    pub fn get_index(&self, index: usize) -> Option<&T> {
        self.as_entries().get(index).map(Bucket::key_ref)
    }

    /// Returns a slice of values in the given range of indices.
    ///
    /// Valid indices are *0 <= index < self.len()*
    ///
    /// Computes in **O(1)** time.
    pub fn get_range<R: RangeBounds<usize>>(&self, range: R) -> Option<&Slice<T>> {
        let entries = self.as_entries();
        let range = try_simplify_range(range, entries.len())?;
        entries.get(range).map(Slice::from_slice)
    }

    /// Get the first value
    ///
    /// Computes in **O(1)** time.
    pub fn first(&self) -> Option<&T> {
        self.as_entries().first().map(Bucket::key_ref)
    }

    /// Get the last value
    ///
    /// Computes in **O(1)** time.
    pub fn last(&self) -> Option<&T> {
        self.as_entries().last().map(Bucket::key_ref)
    }

    /// Remove the value by index
    ///
    /// Valid indices are *0 <= index < self.len()*
    ///
    /// Like `Vec::swap_remove`, the value is removed by swapping it with the
    /// last element of the set and popping it off. **This perturbs
    /// the position of what used to be the last element!**
    ///
    /// Computes in **O(1)** time (average).
    pub fn swap_remove_index(&mut self, index: usize) -> Option<T> {
        self.map.swap_remove_index(index).map(|(x, ())| x)
    }

    /// Remove the value by index
    ///
    /// Valid indices are *0 <= index < self.len()*
    ///
    /// Like `Vec::remove`, the value is removed by shifting all of the
    /// elements that follow it, preserving their relative order.
    /// **This perturbs the index of all of those elements!**
    ///
    /// Computes in **O(n)** time (average).
    pub fn shift_remove_index(&mut self, index: usize) -> Option<T> {
        self.map.shift_remove_index(index).map(|(x, ())| x)
    }

    /// Moves the position of a value from one index to another
    /// by shifting all other values in-between.
    ///
    /// * If `from < to`, the other values will shift down while the targeted value moves up.
    /// * If `from > to`, the other values will shift up while the targeted value moves down.
    ///
    /// ***Panics*** if `from` or `to` are out of bounds.
    ///
    /// Computes in **O(n)** time (average).
    pub fn move_index(&mut self, from: usize, to: usize) {
        self.map.move_index(from, to)
    }

    /// Swaps the position of two values in the set.
    ///
    /// ***Panics*** if `a` or `b` are out of bounds.
    pub fn swap_indices(&mut self, a: usize, b: usize) {
        self.map.swap_indices(a, b)
    }
}

/// Access `IndexSet` values at indexed positions.
///
/// # Examples
///
/// ```
/// use indexmap::IndexSet;
///
/// let mut set = IndexSet::new();
/// for word in "Lorem ipsum dolor sit amet".split_whitespace() {
///     set.insert(word.to_string());
/// }
/// assert_eq!(set[0], "Lorem");
/// assert_eq!(set[1], "ipsum");
/// set.reverse();
/// assert_eq!(set[0], "amet");
/// assert_eq!(set[1], "sit");
/// set.sort();
/// assert_eq!(set[0], "Lorem");
/// assert_eq!(set[1], "amet");
/// ```
///
/// ```should_panic
/// use indexmap::IndexSet;
///
/// let mut set = IndexSet::new();
/// set.insert("foo");
/// println!("{:?}", set[10]); // panics!
/// ```
impl<T, S> Index<usize> for IndexSet<T, S> {
    type Output = T;

    /// Returns a reference to the value at the supplied `index`.
    ///
    /// ***Panics*** if `index` is out of bounds.
    fn index(&self, index: usize) -> &T {
        self.get_index(index)
            .expect("IndexSet: index out of bounds")
    }
}

impl<T, S> FromIterator<T> for IndexSet<T, S>
where
    T: Hash + Eq,
    S: BuildHasher + Default,
{
    fn from_iter<I: IntoIterator<Item = T>>(iterable: I) -> Self {
        let iter = iterable.into_iter().map(|x| (x, ()));
        IndexSet {
            map: IndexMap::from_iter(iter),
        }
    }
}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl<T, const N: usize> From<[T; N]> for IndexSet<T, RandomState>
where
    T: Eq + Hash,
{
    /// # Examples
    ///
    /// ```
    /// use indexmap::IndexSet;
    ///
    /// let set1 = IndexSet::from([1, 2, 3, 4]);
    /// let set2: IndexSet<_> = [1, 2, 3, 4].into();
    /// assert_eq!(set1, set2);
    /// ```
    fn from(arr: [T; N]) -> Self {
        Self::from_iter(arr)
    }
}

impl<T, S> Extend<T> for IndexSet<T, S>
where
    T: Hash + Eq,
    S: BuildHasher,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iterable: I) {
        let iter = iterable.into_iter().map(|x| (x, ()));
        self.map.extend(iter);
    }
}

impl<'a, T, S> Extend<&'a T> for IndexSet<T, S>
where
    T: Hash + Eq + Copy + 'a,
    S: BuildHasher,
{
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iterable: I) {
        let iter = iterable.into_iter().copied();
        self.extend(iter);
    }
}

impl<T, S> Default for IndexSet<T, S>
where
    S: Default,
{
    /// Return an empty `IndexSet`
    fn default() -> Self {
        IndexSet {
            map: IndexMap::default(),
        }
    }
}

impl<T, S1, S2> PartialEq<IndexSet<T, S2>> for IndexSet<T, S1>
where
    T: Hash + Eq,
    S1: BuildHasher,
    S2: BuildHasher,
{
    fn eq(&self, other: &IndexSet<T, S2>) -> bool {
        self.len() == other.len() && self.is_subset(other)
    }
}

impl<T, S> Eq for IndexSet<T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
}

impl<T, S> IndexSet<T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    /// Returns `true` if `self` has no elements in common with `other`.
    pub fn is_disjoint<S2>(&self, other: &IndexSet<T, S2>) -> bool
    where
        S2: BuildHasher,
    {
        if self.len() <= other.len() {
            self.iter().all(move |value| !other.contains(value))
        } else {
            other.iter().all(move |value| !self.contains(value))
        }
    }

    /// Returns `true` if all elements of `self` are contained in `other`.
    pub fn is_subset<S2>(&self, other: &IndexSet<T, S2>) -> bool
    where
        S2: BuildHasher,
    {
        self.len() <= other.len() && self.iter().all(move |value| other.contains(value))
    }

    /// Returns `true` if all elements of `other` are contained in `self`.
    pub fn is_superset<S2>(&self, other: &IndexSet<T, S2>) -> bool
    where
        S2: BuildHasher,
    {
        other.is_subset(self)
    }
}

impl<T, S1, S2> BitAnd<&IndexSet<T, S2>> for &IndexSet<T, S1>
where
    T: Eq + Hash + Clone,
    S1: BuildHasher + Default,
    S2: BuildHasher,
{
    type Output = IndexSet<T, S1>;

    /// Returns the set intersection, cloned into a new set.
    ///
    /// Values are collected in the same order that they appear in `self`.
    fn bitand(self, other: &IndexSet<T, S2>) -> Self::Output {
        self.intersection(other).cloned().collect()
    }
}

impl<T, S1, S2> BitOr<&IndexSet<T, S2>> for &IndexSet<T, S1>
where
    T: Eq + Hash + Clone,
    S1: BuildHasher + Default,
    S2: BuildHasher,
{
    type Output = IndexSet<T, S1>;

    /// Returns the set union, cloned into a new set.
    ///
    /// Values from `self` are collected in their original order, followed by
    /// values that are unique to `other` in their original order.
    fn bitor(self, other: &IndexSet<T, S2>) -> Self::Output {
        self.union(other).cloned().collect()
    }
}

impl<T, S1, S2> BitXor<&IndexSet<T, S2>> for &IndexSet<T, S1>
where
    T: Eq + Hash + Clone,
    S1: BuildHasher + Default,
    S2: BuildHasher,
{
    type Output = IndexSet<T, S1>;

    /// Returns the set symmetric-difference, cloned into a new set.
    ///
    /// Values from `self` are collected in their original order, followed by
    /// values from `other` in their original order.
    fn bitxor(self, other: &IndexSet<T, S2>) -> Self::Output {
        self.symmetric_difference(other).cloned().collect()
    }
}

impl<T, S1, S2> Sub<&IndexSet<T, S2>> for &IndexSet<T, S1>
where
    T: Eq + Hash + Clone,
    S1: BuildHasher + Default,
    S2: BuildHasher,
{
    type Output = IndexSet<T, S1>;

    /// Returns the set difference, cloned into a new set.
    ///
    /// Values are collected in the same order that they appear in `self`.
    fn sub(self, other: &IndexSet<T, S2>) -> Self::Output {
        self.difference(other).cloned().collect()
    }
}
