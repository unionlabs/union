use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut, Index, IndexMut},
    slice::SliceIndex,
};

use derivative::Derivative;
use serde::{Deserialize, Serialize};
pub use typenum;
use typenum::{NonZero, Unsigned};

use crate::{
    tree_hash::Hash256,
    types::{tree_hash::vec_tree_hash_root, Error},
    BYTES_PER_LENGTH_OFFSET,
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
/// use ssz::types::{VariableList, typenum};
///
/// let base: Vec<u64> = vec![1, 2, 3, 4];
///
/// // Create a `VariableList` from a `Vec` that has the expected length:
/// let exact: VariableList<_, typenum::U4> = VariableList::try_from(base.clone()).expect("has valid length");
/// assert_eq!(&exact[..], &[1, 2, 3, 4]);
///
/// // Create a `VariableList` from a `Vec` that is shorter than the maximum:
/// let mut long: VariableList<_, typenum::U5> = base.try_into().expect("has valid length");
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
pub struct VariableList<T, N> {
    vec: Vec<T>,
    _phantom: PhantomData<N>,
}

impl<T: Debug, N: Unsigned + NonZero> Debug for VariableList<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(&format!("VariableList<{}>", N::USIZE))
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

impl<T, N: Unsigned + NonZero> VariableList<T, N> {
    /// Returns `Some` if the given `vec` equals the fixed length of `Self`. Otherwise returns
    /// `None`.
    pub fn new(vec: Vec<T>) -> Result<Self, Error> {
        if vec.len() <= N::to_usize() {
            Ok(Self {
                vec,
                _phantom: PhantomData,
            })
        } else {
            Err(Error::OutOfBounds {
                i: vec.len(),
                len: Self::max_len(),
            })
        }
    }

    /// Create an empty list.
    pub fn empty() -> Self {
        Self {
            vec: vec![],
            _phantom: PhantomData,
        }
    }

    /// Returns the number of values presently in `self`.
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// True if `self` does not contain any values.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the type-level maximum length.
    pub fn max_len() -> usize {
        N::to_usize()
    }

    /// Appends `value` to the back of `self`.
    ///
    /// Returns `Err(())` when appending `value` would exceed the maximum length.
    pub fn push(&mut self, value: T) -> Result<(), Error> {
        if self.vec.len() < Self::max_len() {
            self.vec.push(value);
            Ok(())
        } else {
            Err(Error::OutOfBounds {
                i: self.vec.len() + 1,
                len: Self::max_len(),
            })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TryFromVecError {
    TooLong { max: usize, found: usize },
}

impl<T, N: Unsigned + NonZero> TryFrom<Vec<T>> for VariableList<T, N> {
    type Error = TryFromVecError;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        if value.len() <= N::USIZE {
            Ok(Self {
                vec: value,
                _phantom: PhantomData,
            })
        } else {
            Err(TryFromVecError::TooLong {
                max: N::USIZE,
                found: value.len(),
            })
        }
    }
}

impl<T, N: Unsigned + NonZero> From<VariableList<T, N>> for Vec<T> {
    fn from(list: VariableList<T, N>) -> Vec<T> {
        list.vec
    }
}

impl<T, N: Unsigned + NonZero> Default for VariableList<T, N> {
    fn default() -> Self {
        Self {
            vec: Vec::default(),
            _phantom: PhantomData,
        }
    }
}

impl<T, N: Unsigned + NonZero, I: SliceIndex<[T]>> Index<I> for VariableList<T, N> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&self.vec, index)
    }
}

impl<T, N: Unsigned + NonZero, I: SliceIndex<[T]>> IndexMut<I> for VariableList<T, N> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut self.vec, index)
    }
}

impl<T, N: Unsigned + NonZero> Deref for VariableList<T, N> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.vec[..]
    }
}

impl<T, N: Unsigned + NonZero> DerefMut for VariableList<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.vec[..]
    }
}

impl<T, N: Unsigned + NonZero> AsRef<[T]> for VariableList<T, N> {
    fn as_ref(&self) -> &[T] {
        self.vec.as_ref()
    }
}

impl<'a, T, N: Unsigned + NonZero> IntoIterator for &'a VariableList<T, N> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T, N: Unsigned + NonZero> IntoIterator for VariableList<T, N> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

impl<T, N: Unsigned + NonZero> crate::tree_hash::TreeHash for VariableList<T, N>
where
    T: crate::tree_hash::TreeHash,
{
    fn tree_hash_type() -> crate::tree_hash::TreeHashType {
        crate::tree_hash::TreeHashType::List
    }

    fn tree_hash_packed_encoding(&self) -> crate::tree_hash::PackedEncoding {
        unreachable!("List should never be packed.")
    }

    fn tree_hash_packing_factor() -> usize {
        unreachable!("List should never be packed.")
    }

    fn tree_hash_root(&self) -> Hash256 {
        let root = vec_tree_hash_root::<T, N>(&self.vec);

        crate::tree_hash::mix_in_length(&root, self.len())
    }
}

impl<T, N> crate::Encode for VariableList<T, N>
where
    T: crate::Encode,
    N: Unsigned + NonZero,
{
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        crate::sequence_ssz_append(self.iter(), buf)
    }

    fn ssz_fixed_len() -> usize {
        if Self::is_ssz_fixed_len() {
            T::ssz_fixed_len() * N::USIZE
        } else {
            BYTES_PER_LENGTH_OFFSET
        }
    }

    fn ssz_bytes_len(&self) -> usize {
        crate::sequence_ssz_bytes_len(self.iter())
    }
}

impl<T, N: Unsigned + NonZero> crate::TryFromIter<T> for VariableList<T, N> {
    type Error = Error;

    fn try_from_iter<I>(value: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = T>,
    {
        let n = N::to_usize();
        let clamped_n = std::cmp::min(MAX_ELEMENTS_TO_PRE_ALLOCATE, n);
        let iter = value.into_iter();

        // Pre-allocate up to `N` elements based on the iterator size hint.
        let (_, opt_max_len) = iter.size_hint();
        let mut l = Self::new(Vec::with_capacity(
            opt_max_len.map_or(clamped_n, |max_len| std::cmp::min(clamped_n, max_len)),
        ))?;
        for item in iter {
            l.push(item)?;
        }
        Ok(l)
    }
}

impl<T, N> crate::Decode for VariableList<T, N>
where
    T: crate::Decode,
    N: Unsigned + NonZero,
{
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, crate::DecodeError> {
        let max_len = N::to_usize();

        if bytes.is_empty() {
            Ok(Self::default())
        } else if T::is_ssz_fixed_len() {
            let num_items = bytes
                .len()
                .checked_div(T::ssz_fixed_len())
                .ok_or(crate::DecodeError::ZeroLengthItem)?;

            if num_items > max_len {
                return Err(crate::DecodeError::BytesInvalid(format!(
                    "VariableList of {} items exceeds maximum of {}",
                    num_items, max_len
                )));
            }

            // REVIEW: Why not just map and collect to a result?
            let vec = bytes.chunks(T::ssz_fixed_len()).try_fold(
                Vec::with_capacity(num_items),
                |mut vec, chunk| {
                    vec.push(T::from_ssz_bytes(chunk)?);
                    Ok(vec)
                },
            )?;

            Ok(Self {
                vec,
                _phantom: Default::default(),
            })
        } else {
            crate::decode_list_of_variable_length_items(bytes, Some(max_len)).map(|vec: Vec<T>| {
                vec.try_into()
                    .expect("max length is passed to decode function; qed;")
            })
        }
    }
}

#[cfg(feature = "arbitrary")]
impl<'a, T: arbitrary::Arbitrary<'a>, N: 'static + Unsigned + NonZero> arbitrary::Arbitrary<'a>
    for VariableList<T, N>
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let max_size = N::to_usize();
        let rand = usize::arbitrary(u)?;
        let size = std::cmp::min(rand, max_size);
        let mut vec: Vec<T> = Vec::with_capacity(size);
        for _ in 0..size {
            vec.push(<T>::arbitrary(u)?);
        }
        Ok(Self::new(vec).map_err(|_| arbitrary::Error::IncorrectFormat)?)
    }
}

#[cfg(test)]
mod test {
    use typenum::*;

    use super::*;
    use crate::{tree_hash::merkle_root, *};

    #[test]
    fn new() {
        let vec = vec![42; 5];
        let fixed: Result<VariableList<u64, U4>, _> = VariableList::new(vec);
        assert!(fixed.is_err());

        let vec = vec![42; 3];
        let fixed: Result<VariableList<u64, U4>, _> = VariableList::new(vec);
        assert!(fixed.is_ok());

        let vec = vec![42; 4];
        let fixed: Result<VariableList<u64, U4>, _> = VariableList::new(vec);
        assert!(fixed.is_ok());
    }

    #[test]
    fn indexing() {
        let vec = vec![1, 2];

        let mut fixed: VariableList<u64, U8192> = vec.clone().try_into().unwrap();

        assert_eq!(fixed[0], 1);
        assert_eq!(&fixed[0..1], &vec[0..1]);
        assert_eq!((fixed[..]).len(), 2);

        fixed[1] = 3;
        assert_eq!(fixed[1], 3);
    }

    #[test]
    fn length() {
        assert_eq!(
            VariableList::<u64, U4>::try_from(vec![42; 5]),
            Err(TryFromVecError::TooLong { max: 4, found: 5 })
        );

        let list = VariableList::<u64, U4>::try_from(vec![42; 3]).unwrap();
        assert_eq!(&list[..], &[42, 42, 42]);

        let list = VariableList::<u64, U4>::try_from(vec![]).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn deref() {
        let vec = vec![0, 2, 4, 6];
        let fixed: VariableList<u64, U4> = VariableList::try_from(vec).unwrap();

        assert_eq!(fixed.first(), Some(&0));
        assert_eq!(fixed.get(3), Some(&6));
        assert_eq!(fixed.get(4), None);
    }

    #[test]
    fn encode() {
        let vec: VariableList<u16, U2> = vec![0; 2].try_into().unwrap();
        assert_eq!(vec.as_ssz_bytes(), vec![0, 0, 0, 0]);
        assert_eq!(<VariableList<u16, U2> as Encode>::ssz_fixed_len(), 4);
    }

    fn round_trip<T: Encode + Decode + std::fmt::Debug + PartialEq>(item: T) {
        let encoded = &item.as_ssz_bytes();
        assert_eq!(item.ssz_bytes_len(), encoded.len());
        assert_eq!(T::from_ssz_bytes(encoded), Ok(item));
    }

    #[test]
    fn u16_len_8() {
        round_trip::<VariableList<u16, U8>>(vec![42; 8].try_into().unwrap());
        round_trip::<VariableList<u16, U8>>(vec![0; 8].try_into().unwrap());
    }

    fn root_with_length(bytes: &[u8], len: usize) -> Hash256 {
        let root = merkle_root(bytes, 0);
        crate::tree_hash::mix_in_length(&root, len)
    }

    #[test]
    fn tree_hash_u8() {
        for i in 0..=1 {
            let fixed: VariableList<u8, U1> = (vec![0; i]).try_into().unwrap();
            assert_eq!(fixed.tree_hash_root(), root_with_length(&vec![0; i], i));
        }

        for i in 0..=8 {
            let fixed: VariableList<u8, U8> = (vec![0; i]).try_into().unwrap();
            assert_eq!(fixed.tree_hash_root(), root_with_length(&vec![0; i], i));
        }

        for i in 0..=13 {
            let fixed: VariableList<u8, U13> = (vec![0; i]).try_into().unwrap();
            assert_eq!(fixed.tree_hash_root(), root_with_length(&vec![0; i], i));
        }

        for i in 0..=16 {
            let fixed: VariableList<u8, U16> = (vec![0; i]).try_into().unwrap();
            assert_eq!(fixed.tree_hash_root(), root_with_length(&vec![0; i], i));
        }

        let source: Vec<u8> = (0..16).collect();
        let fixed: VariableList<u8, U16> = source.clone().try_into().unwrap();
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
        type N = U1099511627776;
        type List = VariableList<u64, N>;

        let iter = iter::repeat(1).take(5);
        let wonky_iter = WonkyIterator {
            hint: N::to_usize() / 2,
            iter: iter.clone(),
        };

        // Don't explode.
        assert_eq!(
            List::try_from_iter(iter).unwrap(),
            List::try_from_iter(wonky_iter).unwrap()
        );
    }
}
