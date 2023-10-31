use super::{Bucket, Entries, IndexSet, IntoIter, Iter};
use crate::util::try_simplify_range;

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::{self, Bound, Index, RangeBounds};

/// A dynamically-sized slice of values in an `IndexSet`.
///
/// This supports indexed operations much like a `[T]` slice,
/// but not any hashed operations on the values.
///
/// Unlike `IndexSet`, `Slice` does consider the order for `PartialEq`
/// and `Eq`, and it also implements `PartialOrd`, `Ord`, and `Hash`.
#[repr(transparent)]
pub struct Slice<T> {
    pub(crate) entries: [Bucket<T>],
}

// SAFETY: `Slice<T>` is a transparent wrapper around `[Bucket<T>]`,
// and reference lifetimes are bound together in function signatures.
#[allow(unsafe_code)]
impl<T> Slice<T> {
    pub(super) fn from_slice(entries: &[Bucket<T>]) -> &Self {
        unsafe { &*(entries as *const [Bucket<T>] as *const Self) }
    }

    pub(super) fn from_boxed(entries: Box<[Bucket<T>]>) -> Box<Self> {
        unsafe { Box::from_raw(Box::into_raw(entries) as *mut Self) }
    }

    fn into_boxed(self: Box<Self>) -> Box<[Bucket<T>]> {
        unsafe { Box::from_raw(Box::into_raw(self) as *mut [Bucket<T>]) }
    }
}

impl<T> Slice<T> {
    pub(crate) fn into_entries(self: Box<Self>) -> Vec<Bucket<T>> {
        self.into_boxed().into_vec()
    }

    /// Return the number of elements in the set slice.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the set slice contains no elements.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get a value by index.
    ///
    /// Valid indices are *0 <= index < self.len()*
    pub fn get_index(&self, index: usize) -> Option<&T> {
        self.entries.get(index).map(Bucket::key_ref)
    }

    /// Returns a slice of values in the given range of indices.
    ///
    /// Valid indices are *0 <= index < self.len()*
    pub fn get_range<R: RangeBounds<usize>>(&self, range: R) -> Option<&Self> {
        let range = try_simplify_range(range, self.entries.len())?;
        self.entries.get(range).map(Self::from_slice)
    }

    /// Get the first value.
    pub fn first(&self) -> Option<&T> {
        self.entries.first().map(Bucket::key_ref)
    }

    /// Get the last value.
    pub fn last(&self) -> Option<&T> {
        self.entries.last().map(Bucket::key_ref)
    }

    /// Divides one slice into two at an index.
    ///
    /// ***Panics*** if `index > len`.
    pub fn split_at(&self, index: usize) -> (&Self, &Self) {
        let (first, second) = self.entries.split_at(index);
        (Self::from_slice(first), Self::from_slice(second))
    }

    /// Returns the first value and the rest of the slice,
    /// or `None` if it is empty.
    pub fn split_first(&self) -> Option<(&T, &Self)> {
        if let [first, rest @ ..] = &self.entries {
            Some((&first.key, Self::from_slice(rest)))
        } else {
            None
        }
    }

    /// Returns the last value and the rest of the slice,
    /// or `None` if it is empty.
    pub fn split_last(&self) -> Option<(&T, &Self)> {
        if let [rest @ .., last] = &self.entries {
            Some((&last.key, Self::from_slice(rest)))
        } else {
            None
        }
    }

    /// Return an iterator over the values of the set slice.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(&self.entries)
    }
}

impl<'a, T> IntoIterator for &'a Slice<T> {
    type IntoIter = Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> IntoIterator for Box<Slice<T>> {
    type IntoIter = IntoIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self.into_entries())
    }
}

impl<T> Default for &'_ Slice<T> {
    fn default() -> Self {
        Slice::from_slice(&[])
    }
}

impl<T> Default for Box<Slice<T>> {
    fn default() -> Self {
        Slice::from_boxed(Box::default())
    }
}

impl<T: Clone> Clone for Box<Slice<T>> {
    fn clone(&self) -> Self {
        Slice::from_boxed(self.entries.to_vec().into_boxed_slice())
    }
}

impl<T: Copy> From<&Slice<T>> for Box<Slice<T>> {
    fn from(slice: &Slice<T>) -> Self {
        Slice::from_boxed(Box::from(&slice.entries))
    }
}

impl<T: fmt::Debug> fmt::Debug for Slice<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl<T: PartialEq> PartialEq for Slice<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other)
    }
}

impl<T: Eq> Eq for Slice<T> {}

impl<T: PartialOrd> PartialOrd for Slice<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T: Ord> Ord for Slice<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other)
    }
}

impl<T: Hash> Hash for Slice<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        for value in self {
            value.hash(state);
        }
    }
}

impl<T> Index<usize> for Slice<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index].key
    }
}

// We can't have `impl<I: RangeBounds<usize>> Index<I>` because that conflicts with `Index<usize>`.
// Instead, we repeat the implementations for all the core range types.
macro_rules! impl_index {
    ($($range:ty),*) => {$(
        impl<T, S> Index<$range> for IndexSet<T, S> {
            type Output = Slice<T>;

            fn index(&self, range: $range) -> &Self::Output {
                Slice::from_slice(&self.as_entries()[range])
            }
        }

        impl<T> Index<$range> for Slice<T> {
            type Output = Self;

            fn index(&self, range: $range) -> &Self::Output {
                Slice::from_slice(&self.entries[range])
            }
        }
    )*}
}
impl_index!(
    ops::Range<usize>,
    ops::RangeFrom<usize>,
    ops::RangeFull,
    ops::RangeInclusive<usize>,
    ops::RangeTo<usize>,
    ops::RangeToInclusive<usize>,
    (Bound<usize>, Bound<usize>)
);

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn slice_index() {
        fn check(vec_slice: &[i32], set_slice: &Slice<i32>, sub_slice: &Slice<i32>) {
            assert_eq!(set_slice as *const _, sub_slice as *const _);
            itertools::assert_equal(vec_slice, set_slice);
        }

        let vec: Vec<i32> = (0..10).map(|i| i * i).collect();
        let set: IndexSet<i32> = vec.iter().cloned().collect();
        let slice = set.as_slice();

        // RangeFull
        check(&vec[..], &set[..], &slice[..]);

        for i in 0usize..10 {
            // Index
            assert_eq!(vec[i], set[i]);
            assert_eq!(vec[i], slice[i]);

            // RangeFrom
            check(&vec[i..], &set[i..], &slice[i..]);

            // RangeTo
            check(&vec[..i], &set[..i], &slice[..i]);

            // RangeToInclusive
            check(&vec[..=i], &set[..=i], &slice[..=i]);

            // (Bound<usize>, Bound<usize>)
            let bounds = (Bound::Excluded(i), Bound::Unbounded);
            check(&vec[i + 1..], &set[bounds], &slice[bounds]);

            for j in i..=10 {
                // Range
                check(&vec[i..j], &set[i..j], &slice[i..j]);
            }

            for j in i..10 {
                // RangeInclusive
                check(&vec[i..=j], &set[i..=j], &slice[i..=j]);
            }
        }
    }
}
