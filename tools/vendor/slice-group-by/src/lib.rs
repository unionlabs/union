//! Crate `slice-group-by` is a library for efficiently iterating on a slice by groups defined by
//! a function that specifies if two elements are in the same group.
//!
//! # Example: Linear Searched Immutable Groups
//!
//! You will only need to define a function that returns `true` if two elements are in the same group.
//!
//! The `LinearGroupBy` iterator will always gives contiguous elements to the predicate function.
//!
//! ```rust
//! use slice_group_by::GroupBy;
//!
//! let slice = &[1, 1, 1, 3, 3, 2, 2, 2];
//!
//! let mut iter = slice.linear_group_by_key(|x| -x);
//!
//! assert_eq!(iter.next(), Some(&[1, 1, 1][..]));
//! assert_eq!(iter.next(), Some(&[3, 3][..]));
//! assert_eq!(iter.next(), Some(&[2, 2, 2][..]));
//! assert_eq!(iter.next(), None);
//! ```
//!
//! # Example: Linear Searched Immutable Str Slices
//!
//! You will only need to define a function that returns `true` if two `char` are in the same group.
//!
//! The `LinearStrGroupBy` iterator will always gives contiguous `char` to the predicate function.
//!
//! ```rust
//! use slice_group_by::StrGroupBy;
//!
//! let string = "aaaabbbbb饰饰cccc";
//!
//! let mut iter = string.linear_group_by(|a, b| a == b);
//!
//! assert_eq!(iter.next(), Some("aaaa"));
//! assert_eq!(iter.next(), Some("bbbbb"));
//! assert_eq!(iter.next(), Some("饰饰"));
//! assert_eq!(iter.next(), Some("cccc"));
//! assert_eq!(iter.next(), None);
//! ```
//!
//! # Example: Binary Searched Mutable Groups
//!
//! It is also possible to get mutable non overlapping groups of a slice.
//!
//! The `BinaryGroupBy/Mut` and `ExponentialGroupBy/Mut` iterators will not necessarily
//! gives contiguous elements to the predicate function. The predicate function should implement
//! an order consistent with the sort order of the slice.
//!
//! ```rust
//! use slice_group_by::GroupByMut;
//!
//! let slice = &mut [1, 1, 1, 2, 2, 2, 3, 3];
//!
//! let mut iter = slice.binary_group_by_mut(|a, b| a == b);
//!
//! assert_eq!(iter.next(), Some(&mut [1, 1, 1][..]));
//! assert_eq!(iter.next(), Some(&mut [2, 2, 2][..]));
//! assert_eq!(iter.next(), Some(&mut [3, 3][..]));
//! assert_eq!(iter.next(), None);
//! ```
//!
//! # Example: Exponential Searched Mutable Groups starting from the End
//!
//! It is also possible to get mutable non overlapping groups of a slice even starting from the end of it.
//!
//! ```rust
//! use slice_group_by::GroupByMut;
//!
//! let slice = &mut [1, 1, 1, 2, 2, 2, 3, 3];
//!
//! let mut iter = slice.exponential_group_by_mut(|a, b| a == b).rev();
//!
//! assert_eq!(iter.next(), Some(&mut [3, 3][..]));
//! assert_eq!(iter.next(), Some(&mut [2, 2, 2][..]));
//! assert_eq!(iter.next(), Some(&mut [1, 1, 1][..]));
//! assert_eq!(iter.next(), None);
//! ```
//!

#![cfg_attr(feature = "nightly", feature(ptr_offset_from))]
#![cfg_attr(feature = "nightly", feature(test))]

#![cfg_attr(all(not(test), not(feature = "std")), no_std)]
#[cfg(all(not(test), not(feature = "std")))]
extern crate core as std;

macro_rules! group_by_wrapped {
    (struct $name:ident, $elem:ty) => {
        impl<'a, T: 'a> std::iter::Iterator for $name<'a, T>
        where T: PartialEq,
        {
            type Item = $elem;

            fn next(&mut self) -> Option<Self::Item> {
                self.0.next()
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                self.0.size_hint()
            }

            fn last(self) -> Option<Self::Item> {
                self.0.last()
            }
        }

        impl<'a, T: 'a> DoubleEndedIterator for $name<'a, T>
        where T: PartialEq,
        {
            fn next_back(&mut self) -> Option<Self::Item> {
                self.0.next_back()
            }
        }

        impl<'a, T: 'a> std::iter::FusedIterator for $name<'a, T>
        where T: PartialEq,
        { }
    }
}

mod linear_group;
mod binary_group;
mod exponential_group;
mod linear_str_group;

use std::cmp::{self, Ordering};

pub use self::linear_group::{
    LinearGroupByKey,
    LinearGroupBy,
    LinearGroup,
    LinearGroupByKeyMut,
    LinearGroupByMut,
    LinearGroupMut,
};

pub use self::binary_group::{
    BinaryGroupByKey,
    BinaryGroupBy,
    BinaryGroup,
    BinaryGroupByKeyMut,
    BinaryGroupByMut,
    BinaryGroupMut,
};

pub use self::exponential_group::{
    ExponentialGroupByKey,
    ExponentialGroupBy,
    ExponentialGroup,
    ExponentialGroupByKeyMut,
    ExponentialGroupByMut,
    ExponentialGroupMut,
};

pub use self::linear_str_group::{
    LinearStrGroupByKey,
    LinearStrGroupBy,
    LinearStrGroup,
    LinearStrGroupByKeyMut,
    LinearStrGroupByMut,
    LinearStrGroupMut,
};

#[cfg(feature = "nightly")]
#[inline]
unsafe fn offset_from<T>(to: *const T, from: *const T) -> usize {
    to.offset_from(from) as usize
}

#[cfg(not(feature = "nightly"))]
#[inline]
unsafe fn offset_from<T>(to: *const T, from: *const T) -> usize {
    use std::mem;
    (to as usize - from as usize) / mem::size_of::<T>()
}

/// Exponential searches this sorted slice for a given element.
///
/// If the value is found then `Ok` is returned, containing the index of the matching element;
/// if the value is not found then `Err` is returned, containing the index where a matching element
/// could be inserted while maintaining sorted order.
///
/// # Examples
///
/// Looks up a series of four elements. The first is found, with a
/// uniquely determined position; the second and third are not
/// found; the fourth could match any position in `[1, 4]`.
///
/// ```
/// use slice_group_by::exponential_search;
///
/// let s = &[0, 1, 1, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55];
///
/// assert_eq!(exponential_search(s, &13),  Ok(9));
/// assert_eq!(exponential_search(s, &4),   Err(7));
/// assert_eq!(exponential_search(s, &100), Err(13));
/// let r = exponential_search(s, &1);
/// assert!(match r { Ok(1..=4) => true, _ => false, });
/// ```
#[inline]
pub fn exponential_search<T>(slice: &[T], elem: &T) -> Result<usize, usize>
where T: Ord
{
    exponential_search_by(slice, |x| x.cmp(elem))
}

/// Binary searches this sorted slice with a comparator function.
///
/// The comparator function should implement an order consistent with the sort order of
/// the underlying slice, returning an order code that indicates whether its argument
/// is `Less`, `Equal` or `Greater` the desired target.
///
/// If the value is found then `Ok` is returned, containing the index of the matching element;
/// if the value is not found then `Err` is returned, containing the index where a matching element
/// could be inserted while maintaining sorted order.
///
/// # Examples
///
/// Looks up a series of four elements. The first is found, with a
/// uniquely determined position; the second and third are not
/// found; the fourth could match any position in `[1, 4]`.
///
/// ```
/// use slice_group_by::exponential_search_by;
///
/// let s = &[0, 1, 1, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55];
///
/// let seek = 13;
/// assert_eq!(exponential_search_by(s, |probe| probe.cmp(&seek)), Ok(9));
/// let seek = 4;
/// assert_eq!(exponential_search_by(s, |probe| probe.cmp(&seek)), Err(7));
/// let seek = 100;
/// assert_eq!(exponential_search_by(s, |probe| probe.cmp(&seek)), Err(13));
/// let seek = 1;
/// let r = exponential_search_by(s, |probe| probe.cmp(&seek));
/// assert!(match r { Ok(1..=4) => true, _ => false, });
/// ```
#[inline]
pub fn exponential_search_by<T, F>(slice: &[T], mut f: F) -> Result<usize, usize>
where F: FnMut(&T) -> Ordering,
{
    let mut index = 1;
    while index < slice.len() && f(&slice[index]) == Ordering::Less {
        index *= 2;
    }

    let half_bound = index / 2;
    let bound = cmp::min(index + 1, slice.len());

    match slice[half_bound..bound].binary_search_by(f) {
        Ok(pos) => Ok(half_bound + pos),
        Err(pos) => Err(half_bound + pos),
    }
}

/// Binary searches this sorted slice with a key extraction function.
///
/// Assumes that the slice is sorted by the key.
///
/// If the value is found then `Ok` is returned, containing the index of the matching element;
/// if the value is not found then `Err` is returned, containing the index where a matching element
/// could be inserted while maintaining sorted order.
///
/// # Examples
///
/// Looks up a series of four elements. The first is found, with a
/// uniquely determined position; the second and third are not
/// found; the fourth could match any position in `[1, 4]`.
///
/// ```
/// use slice_group_by::exponential_search_by_key;
///
/// let s = &[(0, 0), (2, 1), (4, 1), (5, 1), (3, 1),
///           (1, 2), (2, 3), (4, 5), (5, 8), (3, 13),
///           (1, 21), (2, 34), (4, 55)];
///
/// assert_eq!(exponential_search_by_key(s, &13, |&(a,b)| b),  Ok(9));
/// assert_eq!(exponential_search_by_key(s, &4, |&(a,b)| b),   Err(7));
/// assert_eq!(exponential_search_by_key(s, &100, |&(a,b)| b), Err(13));
/// let r = exponential_search_by_key(s, &1, |&(a,b)| b);
/// assert!(match r { Ok(1..=4) => true, _ => false, });
/// ```
#[inline]
pub fn exponential_search_by_key<T, B, F>(slice: &[T], b: &B, mut f: F) -> Result<usize, usize>
where F: FnMut(&T) -> B,
      B: Ord
{
    exponential_search_by(slice, |k| f(k).cmp(b))
}

/// A convenient trait to construct an iterator returning non-overlapping groups
/// defined by a predicate.
pub trait GroupBy<T>
{
    /// Returns an iterator on slice groups based that will use the given function to generate keys
    /// and determine groups based on them. It uses *linear search* to iterate over groups.
    fn linear_group_by_key<F, K>(&self, func: F) -> LinearGroupByKey<T, F>
    where F: FnMut(&T) -> K,
          K: PartialEq;

    /// Returns an iterator on slice groups using the *linear search* method.
    fn linear_group_by<P>(&self, predicate: P) -> LinearGroupBy<T, P>
    where P: FnMut(&T, &T) -> bool;

    /// Returns an iterator on slice groups based on the [`PartialEq::eq`] method of `T`,
    /// it uses *linear search* to iterate over groups.
    ///
    /// [`PartialEq::eq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html#tymethod.eq
    fn linear_group(&self) -> LinearGroup<T>
    where T: PartialEq;

    /// Returns an iterator on slice groups based that will use the given function to generate keys
    /// and determine groups based on them. It uses *binary search* to iterate over groups.
    ///
    /// The predicate function should implement an order consistent with
    /// the sort order of the slice.
    fn binary_group_by_key<F, K>(&self, func: F) -> BinaryGroupByKey<T, F>
    where F: FnMut(&T) -> K,
          K: PartialEq;

    /// Returns an iterator on slice groups using the *binary search* method.
    ///
    /// The predicate function should implement an order consistent with
    /// the sort order of the slice.
    fn binary_group_by<P>(&self, predicate: P) -> BinaryGroupBy<T, P>
    where P: FnMut(&T, &T) -> bool;

    /// Returns an iterator on slice groups based on the [`PartialEq::eq`] method of `T`,
    /// it uses *binary search* to iterate over groups.
    ///
    /// The predicate function should implement an order consistent with
    /// the sort order of the slice.
    ///
    /// [`PartialEq::eq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html#tymethod.eq
    fn binary_group(&self) -> BinaryGroup<T>
    where T: PartialEq;

    /// Returns an iterator on slice groups based that will use the given function to generate keys
    /// and determine groups based on them. It uses *exponential search* to iterate over groups.
    ///
    /// The predicate function should implement an order consistent with
    /// the sort order of the slice.
    fn exponential_group_by_key<F, K>(&self, func: F) -> ExponentialGroupByKey<T, F>
    where F: Fn(&T) -> K,
          K: PartialEq;

    /// Returns an iterator on slice groups using the *exponential search* method.
    ///
    /// The predicate function should implement an order consistent with
    /// the sort order of the slice.
    fn exponential_group_by<P>(&self, predicate: P) -> ExponentialGroupBy<T, P>
    where P: FnMut(&T, &T) -> bool;

    /// Returns an iterator on slice groups based on the [`PartialEq::eq`] method of `T`,
    /// it uses *exponential search* to iterate over groups.
    ///
    /// The predicate function should implement an order consistent with
    /// the sort order of the slice.
    ///
    /// [`PartialEq::eq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html#tymethod.eq
    fn exponential_group(&self) -> ExponentialGroup<T>
    where T: PartialEq;
}

/// A convenient trait to construct an iterator returning non-overlapping *mutable*
/// groups defined by a predicate.
pub trait GroupByMut<T>
{
    /// Returns an iterator on *mutable* slice groups based that will use the given function
    /// to generate keys and determine groups based on them. It uses *linear search*
    /// to iterate over groups.
    fn linear_group_by_key_mut<F, K>(&mut self, func: F) -> LinearGroupByKeyMut<T, F>
    where F: FnMut(&T) -> K,
          K: PartialEq;

    /// Returns an iterator on *mutable* slice groups using the *linear search* method.
    fn linear_group_by_mut<P>(&mut self, predicate: P) -> LinearGroupByMut<T, P>
    where P: FnMut(&T, &T) -> bool;

    /// Returns an iterator on *mutable* slice groups based on the [`PartialEq::eq`] method of `T`,
    /// it uses *linear search* to iterate over groups.
    ///
    /// [`PartialEq::eq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html#tymethod.eq
    fn linear_group_mut(&mut self) -> LinearGroupMut<T>
    where T: PartialEq;

    /// Returns an iterator on *mutable* slice groups based that will use the given function
    /// to generate keys and determine groups based on them. It uses *binary search*
    /// to iterate over groups.
    ///
    /// The predicate function should implement an order consistent with
    /// the sort order of the slice.
    fn binary_group_by_key_mut<F, K>(&mut self, func: F) -> BinaryGroupByKeyMut<T, F>
    where F: FnMut(&T) -> K,
          K: PartialEq;

    /// Returns an iterator on *mutable* slice groups using the *binary search* method.
    ///
    /// The predicate function should implement an order consistent with
    /// the sort order of the slice.
    fn binary_group_by_mut<P>(&mut self, predicate: P) -> BinaryGroupByMut<T, P>
    where P: FnMut(&T, &T) -> bool;

    /// Returns an iterator on *mutable* slice groups based on the [`PartialEq::eq`] method of `T`,
    /// it uses *binary search* to iterate over groups.
    ///
    /// The predicate function should implement an order consistent with
    /// the sort order of the slice.
    ///
    /// [`PartialEq::eq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html#tymethod.eq
    fn binary_group_mut(&mut self) -> BinaryGroupMut<T>
    where T: PartialEq;

    /// Returns an iterator on *mutable* slice groups based that will use the given function
    /// to generate keys and determine groups based on them. It uses *exponential search*
    /// to iterate over groups.
    ///
    /// The predicate function should implement an order consistent with
    /// the sort order of the slice.
    fn exponential_group_by_key_mut<F, K>(&mut self, func: F) -> ExponentialGroupByKeyMut<T, F>
    where F: Fn(&T) -> K,
          K: PartialEq;

    /// Returns an iterator on *mutable* slice groups using the *exponential search* method.
    ///
    /// The predicate function should implement an order consistent with
    /// the sort order of the slice.
    fn exponential_group_by_mut<P>(&mut self, predicate: P) -> ExponentialGroupByMut<T, P>
    where P: FnMut(&T, &T) -> bool;

    /// Returns an iterator on *mutable* slice groups based on the [`PartialEq::eq`] method of `T`,
    /// it uses *exponential search* to iterate over groups.
    ///
    /// The predicate function should implement an order consistent with
    /// the sort order of the slice.
    ///
    /// [`PartialEq::eq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html#tymethod.eq
    fn exponential_group_mut(&mut self) -> ExponentialGroupMut<T>
    where T: PartialEq;
}

impl<T> GroupBy<T> for [T]
{
    fn linear_group_by_key<F, K>(&self, func: F) -> LinearGroupByKey<T, F>
    where F: FnMut(&T) -> K,
          K: PartialEq
    {
        LinearGroupByKey::new(self, func)
    }

    fn linear_group_by<P>(&self, predicate: P) -> LinearGroupBy<T, P>
    where P: FnMut(&T, &T) -> bool,
    {
        LinearGroupBy::new(self, predicate)
    }

    fn linear_group(&self) -> LinearGroup<T>
    where T: PartialEq,
    {
        LinearGroup::new(self)
    }

    fn binary_group_by_key<F, K>(&self, func: F) -> BinaryGroupByKey<T, F>
    where F: FnMut(&T) -> K,
          K: PartialEq
    {
        BinaryGroupByKey::new(self, func)
    }

    fn binary_group_by<P>(&self, predicate: P) -> BinaryGroupBy<T, P>
    where P: FnMut(&T, &T) -> bool,
    {
        BinaryGroupBy::new(self, predicate)
    }

    fn binary_group(&self) -> BinaryGroup<T>
    where T: PartialEq,
    {
        BinaryGroup::new(self)
    }

    fn exponential_group_by_key<F, K>(&self, func: F) -> ExponentialGroupByKey<T, F>
    where F: Fn(&T) -> K,
          K: PartialEq
    {
        ExponentialGroupByKey::new(self, func)
    }

    fn exponential_group_by<P>(&self, predicate: P) -> ExponentialGroupBy<T, P>
    where P: FnMut(&T, &T) -> bool,
    {
        ExponentialGroupBy::new(self, predicate)
    }

    fn exponential_group(&self) -> ExponentialGroup<T>
    where T: PartialEq,
    {
        ExponentialGroup::new(self)
    }
}

impl<T> GroupByMut<T> for [T]
{
    fn linear_group_by_key_mut<F, K>(&mut self, func: F) -> LinearGroupByKeyMut<T, F>
    where F: FnMut(&T) -> K,
          K: PartialEq
    {
        LinearGroupByKeyMut::new(self, func)
    }

    fn linear_group_by_mut<P>(&mut self, predicate: P) -> LinearGroupByMut<T, P>
    where P: FnMut(&T, &T) -> bool,
    {
        LinearGroupByMut::new(self, predicate)
    }

    fn linear_group_mut(&mut self) -> LinearGroupMut<T>
    where T: PartialEq,
    {
        LinearGroupMut::new(self)
    }

    fn binary_group_by_key_mut<F, K>(&mut self, func: F) -> BinaryGroupByKeyMut<T, F>
    where F: FnMut(&T) -> K,
          K: PartialEq
    {
        BinaryGroupByKeyMut::new(self, func)
    }

    fn binary_group_by_mut<P>(&mut self, predicate: P) -> BinaryGroupByMut<T, P>
    where P: FnMut(&T, &T) -> bool,
    {
        BinaryGroupByMut::new(self, predicate)
    }

    fn binary_group_mut(&mut self) -> BinaryGroupMut<T>
    where T: PartialEq,
    {
        BinaryGroupMut::new(self)
    }

    fn exponential_group_by_key_mut<F, K>(&mut self, func: F) -> ExponentialGroupByKeyMut<T, F>
    where F: Fn(&T) -> K,
          K: PartialEq
    {
        ExponentialGroupByKeyMut::new(self, func)
    }

    fn exponential_group_by_mut<P>(&mut self, predicate: P) -> ExponentialGroupByMut<T, P>
    where P: FnMut(&T, &T) -> bool,
    {
        ExponentialGroupByMut::new(self, predicate)
    }

    fn exponential_group_mut(&mut self) -> ExponentialGroupMut<T>
    where T: PartialEq,
    {
        ExponentialGroupMut::new(self)
    }
}

/// A convenient trait to construct an iterator returning non-overlapping `str` slices
/// defined by a predicate.
pub trait StrGroupBy
{
    /// Returns an iterator on `str` groups based that will use the given function
    /// to generate keys and determine groups based on them. It uses *linear search*
    /// to iterate over groups.
    fn linear_group_by_key<F, K>(&self, func: F) -> LinearStrGroupByKey<F>
    where F: FnMut(char) -> K,
          K: PartialEq;

    /// Returns an iterator on `str` groups using the *linear search* method.
    fn linear_group_by<P>(&self, predicate: P) -> LinearStrGroupBy<P>
    where P: FnMut(char, char) -> bool;

    /// Returns an iterator on `str` groups based on the [`PartialEq::eq`] method of `char`,
    /// it uses *linear search* to iterate over groups.
    ///
    /// [`PartialEq::eq`]: https://doc.rust-lang.org/std/primitive.char.html#impl-PartialEq%3Cchar%3E
    fn linear_group(&self) -> LinearStrGroup;
}

/// A convenient trait to construct an iterator returning non-overlapping *mutable* `str` slices
/// defined by a predicate.
pub trait StrGroupByMut
{
    /// Returns an iterator on *mutable* `str` groups based that will use the given function
    /// to generate keys and determine groups based on them. It uses *linear search*
    /// to iterate over groups.
    fn linear_group_by_key_mut<F, K>(&mut self, func: F) -> LinearStrGroupByKeyMut<F>
    where F: FnMut(char) -> K,
          K: PartialEq;

    /// Returns an iterator on *mutable* `str` groups using the *linear search* method.
    fn linear_group_by_mut<P>(&mut self, predicate: P) -> LinearStrGroupByMut<P>
    where P: FnMut(char, char) -> bool;

    /// Returns an iterator on *mutable* `str` groups based on the [`PartialEq::eq`] method of `char`,
    /// it uses *linear search* to iterate over groups.
    ///
    /// [`PartialEq::eq`]: https://doc.rust-lang.org/std/primitive.char.html#impl-PartialEq%3Cchar%3E
    fn linear_group_mut(&mut self) -> LinearStrGroupMut;
}

impl StrGroupBy for str
{
    fn linear_group_by_key<F, K>(&self, func: F) -> LinearStrGroupByKey<F>
    where F: FnMut(char) -> K,
          K: PartialEq
    {
        LinearStrGroupByKey::new(self, func)
    }

    fn linear_group_by<P>(&self, predicate: P) -> LinearStrGroupBy<P>
    where P: FnMut(char, char) -> bool,
    {
        LinearStrGroupBy::new(self, predicate)
    }

    fn linear_group(&self) -> LinearStrGroup {
        LinearStrGroup::new(self)
    }
}

impl StrGroupByMut for str
{
    fn linear_group_by_key_mut<F, K>(&mut self, func: F) -> LinearStrGroupByKeyMut<F>
    where F: FnMut(char) -> K,
          K: PartialEq
    {
        LinearStrGroupByKeyMut::new(self, func)
    }

    fn linear_group_by_mut<P>(&mut self, predicate: P) -> LinearStrGroupByMut<P>
    where P: FnMut(char, char) -> bool,
    {
        LinearStrGroupByMut::new(self, predicate)
    }

    fn linear_group_mut(&mut self) -> LinearStrGroupMut {
        LinearStrGroupMut::new(self)
    }
}
