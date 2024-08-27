use std::{
    fmt::Debug,
    marker::PhantomData,
    num::NonZeroUsize,
    ops::{Deref, DerefMut, Div, Index, IndexMut},
    slice::SliceIndex,
};

use derivative::Derivative;
use serde::{Deserialize, Serialize};
pub use typenum;
use typenum::{NonZero, Unsigned};

use crate::{
    decode::TryFromIter,
    decode_list_of_variable_length_items, sequence_ssz_append, sequence_ssz_bytes_len,
    tree_hash::{Hash256, TreeHashType},
    types::tree_hash::vec_tree_hash_root,
    Ssz,
};

/// Emulates a SSZ `List`.
///
/// An ordered, heap-allocated, variable-length, homogeneous collection of `T`, with no more than
/// `N` values.
///
/// This struct is backed by a Rust `Vec` but constrained such that it must be instantiated with a
/// fixed number of elements and you may not add or remove elements, only modify.
///
/// The length of this struct is fixed at the type-level using
/// [typenum](https://crates.io/crates/typenum).
///
/// ## Example
///
/// ```
/// use ssz::types::{List, typenum};
///
/// let base: Vec<u64> = vec![1, 2, 3, 4];
///
/// // Create a `List` from a `Vec` that has the expected length:
/// let exact: List<_, typenum::U4> = List::try_from(base.clone()).expect("has valid length");
/// assert_eq!(&exact[..], &[1, 2, 3, 4]);
///
/// // Create a `List` from a `Vec` that is shorter than the maximum:
/// let mut long: List<_, typenum::U5> = base.try_into().expect("has valid length");
/// assert_eq!(&long[..], &[1, 2, 3, 4]);
///
/// // Push a value to if it does not exceed the maximum
/// long.push(5).unwrap();
/// assert_eq!(&long[..], &[1, 2, 3, 4, 5]);
///
/// // Push a value to if it _does_ exceed the maximum.
/// assert!(long.push(6).is_err());
/// ```
#[derive(Serialize, Deserialize, Derivative)]
#[derivative(
    Clone(bound = "T: ::core::clone::Clone"),
    PartialEq(bound = "T: ::core::cmp::PartialEq"),
    Eq(bound = "T: ::core::cmp::Eq"),
    Hash(bound = "T: ::core::hash::Hash")
)]
#[serde(transparent)]
pub struct List<T, N> {
    vec: Vec<T>,
    #[serde(skip)]
    _phantom: PhantomData<N>,
}

impl<T: Debug, N: Unsigned + NonZero> Debug for List<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(&format!("List<{}>", N::USIZE))
            .field(&self.vec)
            .finish()
    }
}

/// Maximum number of elements to pre-allocate in `try_from_iter`.
///
/// Some variable lists have *very long* maximum lengths such that we can't actually fit them
/// in memory. This value is set to 128K with the expectation that any list with a large maximum
/// length (N) will contain at least a few thousand small values. i.e. we're targeting an
/// allocation around the 1MiB to 10MiB mark.
const MAX_ELEMENTS_TO_PRE_ALLOCATE: usize = 128 * (1 << 10);

impl<T, N> List<T, N>
where
    N: Unsigned + NonZero,
{
    /// Returns `Some` if the given `vec` equals the fixed length of `Self`, returning the passed in vec on error.
    pub fn new(vec: Vec<T>) -> Result<Self, Vec<T>> {
        if vec.len() <= N::USIZE {
            Ok(Self {
                vec,
                _phantom: PhantomData,
            })
        } else {
            Err(vec)
        }
    }

    /// Returns the number of values presently in `self`.
    #[must_use]
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    /// Appends `value` to the back of `self`.
    ///
    /// Returns `Err(())` when appending `value` would exceed the maximum length.
    pub fn push(&mut self, value: T) -> Result<(), PushError> {
        if self.vec.len() < N::USIZE {
            self.vec.push(value);
            Ok(())
        } else {
            Err(PushError)
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("attempted to `.push()` to a full `List`")]
pub struct PushError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TryFromVecError {
    TooLong { max: usize, found: usize },
}

impl<T, N: Unsigned + NonZero> TryFrom<Vec<T>> for List<T, N> {
    type Error = Vec<T>;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<T, N: Unsigned + NonZero> From<List<T, N>> for Vec<T> {
    fn from(list: List<T, N>) -> Vec<T> {
        list.vec
    }
}

impl<T, N: Unsigned + NonZero> Default for List<T, N> {
    fn default() -> Self {
        Self {
            vec: Vec::default(),
            _phantom: PhantomData,
        }
    }
}

impl<T, N: Unsigned + NonZero, I: SliceIndex<[T]>> Index<I> for List<T, N> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&self.vec, index)
    }
}

impl<T, N: Unsigned + NonZero, I: SliceIndex<[T]>> IndexMut<I> for List<T, N> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut self.vec, index)
    }
}

impl<T, N: Unsigned + NonZero> Deref for List<T, N> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.vec[..]
    }
}

impl<T, N: Unsigned + NonZero> DerefMut for List<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.vec[..]
    }
}

impl<T, N: Unsigned + NonZero> AsRef<[T]> for List<T, N> {
    fn as_ref(&self) -> &[T] {
        self.vec.as_ref()
    }
}

impl<'a, T, N: Unsigned + NonZero> IntoIterator for &'a List<T, N> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T, N: Unsigned + NonZero> IntoIterator for List<T, N> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

impl<T, N> Ssz for List<T, N>
where
    T: Ssz,
    N: Unsigned + NonZero,
{
    const SSZ_FIXED_LEN: Option<NonZeroUsize> = None;

    const TREE_HASH_TYPE: TreeHashType = TreeHashType::List;

    fn tree_hash_root(&self) -> Hash256 {
        let root = vec_tree_hash_root::<T, N>(&self.vec);

        crate::tree_hash::mix_in_length(&root, self.len())
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        sequence_ssz_append(self.iter(), buf);
    }

    fn ssz_bytes_len(&self) -> NonZeroUsize {
        sequence_ssz_bytes_len(self.iter())
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, crate::DecodeError> {
        let max_len = N::USIZE;

        if bytes.is_empty() {
            Ok(Self::default())
        } else {
            match T::SSZ_FIXED_LEN {
                Some(fixed_len) => {
                    let num_items = bytes
                        .len()
                        // safe since fixed_len is non-zero
                        .div(fixed_len.get());

                    if num_items > max_len {
                        return Err(crate::DecodeError::BytesInvalid(format!(
                            "List of {} items exceeds maximum of {}",
                            num_items, max_len
                        )));
                    }

                    let vec = bytes
                        .chunks(fixed_len.get())
                        .map(T::from_ssz_bytes)
                        .collect::<Result<_, _>>()?;

                    Ok(Self {
                        vec,
                        _phantom: Default::default(),
                    })
                }
                None => decode_list_of_variable_length_items(bytes, Some(max_len)),
            }
        }
    }
}

impl<T, N: Unsigned + NonZero> TryFromIter<T> for List<T, N> {
    type Error = TryFromVecError;

    fn try_from_iter<I>(value: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = T>,
    {
        let n = N::USIZE;
        let clamped_n = std::cmp::min(MAX_ELEMENTS_TO_PRE_ALLOCATE, n);
        let iter = value.into_iter();

        // Pre-allocate up to `N` elements based on the iterator size hint.
        let (_, opt_max_len) = iter.size_hint();
        let mut vec = Vec::with_capacity(
            opt_max_len.map_or(clamped_n, |max_len| std::cmp::min(clamped_n, max_len)),
        );

        for item in iter {
            // Bail out as soon as the length tries to exceed the limit. This guards against
            // memory denial-of-service attacks.
            if vec.len() >= n {
                return Err(TryFromVecError::TooLong {
                    max: n,
                    found: vec.len(),
                });
            }
            vec.push(item);
        }
        Ok(Self::new(vec).ok().expect("length is checked above; qed;"))
    }
}

#[cfg(test)]
mod test {
    use typenum::*;

    use super::*;
    use crate::tree_hash::merkle_root;

    #[test]
    fn new() {
        let vec = vec![42; 5];
        let fixed: Result<List<u64, U4>, _> = List::new(vec);
        assert!(fixed.is_err());

        let vec = vec![42; 3];
        let fixed: Result<List<u64, U4>, _> = List::new(vec);
        assert!(fixed.is_ok());

        let vec = vec![42; 4];
        let fixed: Result<List<u64, U4>, _> = List::new(vec);
        assert!(fixed.is_ok());
    }

    #[test]
    fn indexing() {
        let vec = vec![1, 2];

        let mut fixed: List<u64, U8192> = vec.clone().try_into().unwrap();

        assert_eq!(fixed[0], 1);
        assert_eq!(&fixed[0..1], &vec[0..1]);
        assert_eq!((fixed[..]).len(), 2);

        fixed[1] = 3;
        assert_eq!(fixed[1], 3);
    }

    #[test]
    fn length() {
        assert_eq!(List::<u64, U4>::try_from(vec![42; 5]), Err(vec![42; 5]));

        let list = List::<u64, U4>::try_from(vec![42; 3]).unwrap();
        assert_eq!(&list[..], &[42, 42, 42]);

        assert!(matches!(List::<u64, U4>::try_from(vec![]), Ok(..)));
    }

    #[test]
    fn deref() {
        let vec = vec![0, 2, 4, 6];
        let fixed: List<u64, U4> = List::try_from(vec).unwrap();

        assert_eq!(fixed.first(), Some(&0));
        assert_eq!(fixed.get(3), Some(&6));
        assert_eq!(fixed.get(4), None);
    }

    #[test]
    fn encode() {
        let vec: List<u16, U2> = vec![0; 2].try_into().unwrap();
        assert_eq!(vec.as_ssz_bytes(), vec![0, 0, 0, 0]);
        assert_eq!(<List<u16, U2> as Ssz>::SSZ_FIXED_LEN, None);
    }

    fn round_trip<T: Ssz + std::fmt::Debug + PartialEq>(item: T) {
        let encoded = &item.as_ssz_bytes();
        assert_eq!(item.ssz_bytes_len().get(), encoded.len());
        assert_eq!(T::from_ssz_bytes(encoded), Ok(item));
    }

    #[test]
    fn u16_len_8() {
        round_trip::<List<u16, U8>>(vec![42; 8].try_into().unwrap());
        round_trip::<List<u16, U8>>(vec![0; 8].try_into().unwrap());
    }

    fn root_with_length(bytes: &[u8], len: usize) -> Hash256 {
        let root = merkle_root(bytes, 0);
        crate::tree_hash::mix_in_length(&root, len)
    }

    #[test]
    fn tree_hash_u8() {
        for i in 1..=1 {
            let fixed: List<u8, U1> = (vec![0; i]).try_into().unwrap();
            assert_eq!(fixed.tree_hash_root(), root_with_length(&vec![0; i], i));
        }

        for i in 1..=8 {
            let fixed: List<u8, U8> = (vec![0; i]).try_into().unwrap();
            assert_eq!(fixed.tree_hash_root(), root_with_length(&vec![0; i], i));
        }

        for i in 1..=13 {
            let fixed: List<u8, U13> = (vec![0; i]).try_into().unwrap();
            assert_eq!(fixed.tree_hash_root(), root_with_length(&vec![0; i], i));
        }

        for i in 1..=16 {
            let fixed: List<u8, U16> = (vec![0; i]).try_into().unwrap();
            assert_eq!(fixed.tree_hash_root(), root_with_length(&vec![0; i], i));
        }

        let source: Vec<u8> = (0..16).collect();
        let fixed: List<u8, U16> = source.clone().try_into().unwrap();
        assert_eq!(fixed.tree_hash_root(), root_with_length(&source, 16));
    }

    #[test]
    fn large_list_pre_allocation() {
        use std::iter;

        use typenum::U1099511627776;

        // Iterator that hints the upper bound on its length as `hint`.
        struct WonkyIterator<I> {
            hint: usize,
            iter: I,
        }

        impl<I> Iterator for WonkyIterator<I>
        where
            I: Iterator,
        {
            type Item = I::Item;

            fn next(&mut self) -> Option<Self::Item> {
                self.iter.next()
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                (0, Some(self.hint))
            }
        }

        // Very large list type that would not fit in memory.
        type LargeList = List<u64, U1099511627776>;

        let iter = iter::repeat(1).take(5);
        let wonky_iter = WonkyIterator {
            hint: U1099511627776::USIZE / 2,
            iter: iter.clone(),
        };

        // Don't explode.
        assert_eq!(
            LargeList::try_from_iter(iter).unwrap(),
            LargeList::try_from_iter(wonky_iter).unwrap()
        );
    }
}
