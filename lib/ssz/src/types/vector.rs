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
use typenum::{Const, NonZero, ToUInt, Unsigned, U};

use crate::{
    decode::TryFromIter,
    decode_list_of_variable_length_items, sequence_ssz_append, sequence_ssz_bytes_len,
    tree_hash::{Hash256, TreeHashType},
    types::tree_hash::vec_tree_hash_root,
    DecodeError, Ssz,
};

/// Emulates a SSZ `Vector` (distinct from a Rust `Vec`).
///
/// An ordered, heap-allocated, fixed-length, homogeneous collection of `T`, with `N` values.
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
/// use ssz::types::{Vector, typenum};
///
/// let base: Vec<u64> = vec![1, 2, 3, 4];
///
/// // Create a `Vector` from a `Vec` that has the expected length (an incorrect length will fail):
/// let exact: Vector<_, typenum::U4> = base.clone().try_into().expect("length is valid");
/// assert_eq!(&exact[..], &[1, 2, 3, 4]);
/// ```
#[derive(Serialize, Deserialize, Derivative)]
#[derivative(
    Clone(bound = "T: ::core::clone::Clone"),
    PartialEq(bound = "T: ::core::cmp::PartialEq"),
    Eq(bound = "T: ::core::cmp::Eq"),
    Hash(bound = "T: ::core::hash::Hash")
)]
#[serde(transparent)]
pub struct Vector<T, N> {
    vec: Vec<T>,
    _phantom: PhantomData<N>,
}

impl<T: Debug, N: Unsigned + NonZero> Debug for Vector<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(&format!("Vector<{}>", N::USIZE))
            .field(&self.vec)
            .finish()
    }
}

impl<T, N: Unsigned + NonZero> Vector<T, N> {
    /// Returns `Ok` if the given `vec` equals the fixed length of `Self`, returning the vec on error.
    pub fn new(vec: Vec<T>) -> Result<Self, Vec<T>> {
        if vec.len() == N::USIZE {
            Ok(Self {
                vec,
                _phantom: PhantomData,
            })
        } else {
            Err(vec)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TryFromVecError {
    InvalidLength { expected: usize, found: usize },
}

impl<T, N: Unsigned + NonZero> TryFrom<Vec<T>> for Vector<T, N> {
    type Error = Vec<T>;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<T, const N: usize> From<[T; N]> for Vector<T, U<N>>
where
    Const<N>: ToUInt,
{
    fn from(value: [T; N]) -> Self {
        Self {
            vec: value.into(),
            _phantom: PhantomData,
        }
    }
}

impl<T, const N: usize> From<Vector<T, U<N>>> for [T; N]
where
    Const<N>: ToUInt,
{
    fn from(value: Vector<T, U<N>>) -> Self {
        // .ok() to avoid T: Debug
        value.vec.try_into().ok().expect("N == U<N>")
    }
}

impl<T, N: Unsigned + NonZero> From<Vector<T, N>> for Vec<T> {
    fn from(vector: Vector<T, N>) -> Vec<T> {
        vector.vec
    }
}

impl<T: Default, N: Unsigned + NonZero> Default for Vector<T, N> {
    fn default() -> Self {
        Self {
            vec: (0..N::USIZE).map(|_| T::default()).collect(),
            _phantom: PhantomData,
        }
    }
}

impl<T, N: Unsigned + NonZero, I: SliceIndex<[T]>> Index<I> for Vector<T, N> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&self.vec, index)
    }
}

impl<T, N: Unsigned + NonZero, I: SliceIndex<[T]>> IndexMut<I> for Vector<T, N> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut self.vec, index)
    }
}

impl<T, N: Unsigned + NonZero> Deref for Vector<T, N> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.vec[..]
    }
}

// This implementation is required to use `get_mut` to access elements.
//
// It's safe because none of the methods on mutable slices allow changing the length
// of the backing vec.
impl<T, N: Unsigned + NonZero> DerefMut for Vector<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.vec[..]
    }
}

impl<T, N: Unsigned + NonZero> AsRef<[T]> for Vector<T, N> {
    fn as_ref(&self) -> &[T] {
        self.vec.as_ref()
    }
}

impl<'a, T, N: Unsigned + NonZero> IntoIterator for &'a Vector<T, N> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T, N: Unsigned + NonZero> IntoIterator for Vector<T, N> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

impl<T, N: Unsigned + NonZero> Ssz for Vector<T, N>
where
    T: Ssz,
{
    const SSZ_FIXED_LEN: Option<NonZeroUsize> = match T::SSZ_FIXED_LEN {
        Some(len) => match NonZeroUsize::new(len.get() * N::USIZE) {
            Some(len) => Some(len),
            None => unreachable!(),
        },
        None => None,
    };

    const TREE_HASH_TYPE: TreeHashType = TreeHashType::Vector;

    fn tree_hash_root(&self) -> Hash256 {
        vec_tree_hash_root::<T, N>(&self.vec)
    }

    fn ssz_bytes_len(&self) -> NonZeroUsize {
        sequence_ssz_bytes_len(&self.vec)
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        sequence_ssz_append(&self.vec, buf);
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            Err(DecodeError::InvalidByteLength {
                found: 0,
                expected: 1,
            })
        } else {
            match T::SSZ_FIXED_LEN {
                Some(fixed_len) => {
                    let num_items = bytes
                        .len()
                        // safe since fixed_len is non-zero
                        .div(fixed_len.get());

                    if num_items != N::USIZE {
                        return Err(DecodeError::InvalidVectorLength {
                            found: num_items,
                            expected: N::USIZE,
                        });
                    }

                    // REVIEW: Potential for DOS? the length is checked above, so it should be fine?
                    Ok(Self::new(
                        bytes
                            .chunks(fixed_len.get())
                            .map(|chunk| T::from_ssz_bytes(chunk))
                            .collect::<Result<Vec<T>, _>>()?,
                    )
                    .ok()
                    .expect("length is checked above; qed;"))
                }
                None => decode_list_of_variable_length_items(bytes, Some(N::USIZE)),
            }
        }
    }
}

impl<T, N: Unsigned + NonZero> TryFromIter<T> for Vector<T, N> {
    type Error = TryFromVecError;

    fn try_from_iter<I>(value: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = T>,
    {
        let iter = value.into_iter();

        let (_, opt_max_len) = iter.size_hint();
        let mut vec = Vec::with_capacity(
            opt_max_len.map_or(N::USIZE, |max_len| std::cmp::min(N::USIZE, max_len)),
        );

        for item in iter {
            // Bail out as soon as the length tries to exceed the limit. This guards against
            // memory denial-of-service attacks.
            if vec.len() >= N::USIZE {
                return Err(TryFromVecError::InvalidLength {
                    found: vec.len(),
                    expected: N::USIZE,
                });
            }
            vec.push(item);
        }

        Self::new(vec).map_err(|invalid| TryFromVecError::InvalidLength {
            found: invalid.len(),
            expected: N::USIZE,
        })
    }
}

#[cfg(feature = "arbitrary")]
impl<'a, T: arbitrary::Arbitrary<'a>, N: 'static + Unsigned + NonZero> arbitrary::Arbitrary<'a>
    for Vector<T, N>
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let size = N::USIZE;
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
    use crate::tree_hash::merkle_root;

    #[test]
    fn new() {
        let vec = vec![42; 5];
        let fixed: Result<Vector<u64, U4>, _> = Vector::new(vec);
        assert!(fixed.is_err());

        let vec = vec![42; 3];
        let fixed: Result<Vector<u64, U4>, _> = Vector::new(vec);
        assert!(fixed.is_err());

        let vec = vec![42; 4];
        let fixed: Result<Vector<u64, U4>, _> = Vector::new(vec);
        assert!(fixed.is_ok());
    }

    #[test]
    fn indexing() {
        let vec = vec![1, 2]
            .into_iter()
            .chain(vec![0; 8190])
            .collect::<Vec<_>>();

        let mut fixed: Vector<u64, U8192> = vec.clone().try_into().unwrap();

        assert_eq!(fixed[0], 1);
        assert_eq!(&fixed[0..1], &vec[0..1]);
        assert_eq!((fixed[..]).len(), 8192);

        fixed[1] = 3;
        assert_eq!(fixed[1], 3);
    }

    #[test]
    fn length() {
        assert_eq!(Vector::<u64, U4>::try_from(vec![42; 5]), Err(vec![42; 5]));

        assert_eq!(Vector::<u64, U4>::try_from(vec![42; 3]), Err(vec![42; 3]));

        assert_eq!(Vector::<u64, U4>::try_from(vec![]), Err(vec![]));
    }

    #[test]
    fn deref() {
        let vec = vec![0, 2, 4, 6];
        let fixed: Vector<u64, U4> = vec.try_into().unwrap();

        assert_eq!(fixed.first(), Some(&0));
        assert_eq!(fixed.get(3), Some(&6));
        assert_eq!(fixed.get(4), None);
    }

    #[test]
    fn ssz_encode() {
        let vec: Vector<u16, U2> = vec![0; 2].try_into().unwrap();
        assert_eq!(vec.as_ssz_bytes(), vec![0, 0, 0, 0]);
        assert_eq!(<Vector<u16, U2> as Ssz>::SSZ_FIXED_LEN.unwrap().get(), 4);
    }

    fn ssz_round_trip<T: Ssz + std::fmt::Debug + PartialEq>(item: T) {
        let encoded = &item.as_ssz_bytes();
        assert_eq!(item.ssz_bytes_len().get(), encoded.len());
        assert_eq!(T::from_ssz_bytes(encoded), Ok(item));
    }

    #[test]
    fn ssz_round_trip_u16_len_8() {
        ssz_round_trip::<Vector<u16, U8>>(vec![42; 8].try_into().unwrap());
        ssz_round_trip::<Vector<u16, U8>>(vec![0; 8].try_into().unwrap());
    }

    #[test]
    fn tree_hash_u8() {
        let fixed: Vector<u8, U1> = vec![0; 1].try_into().unwrap();
        assert_eq!(fixed.tree_hash_root(), merkle_root(&[0; 8], 0));

        let fixed: Vector<u8, U8> = vec![0; 8].try_into().unwrap();
        assert_eq!(fixed.tree_hash_root(), merkle_root(&[0; 8], 0));

        let fixed: Vector<u8, U16> = vec![42; 16].try_into().unwrap();
        assert_eq!(fixed.tree_hash_root(), merkle_root(&[42; 16], 0));

        let source: Vec<u8> = (0..16).collect();
        let fixed: Vector<u8, U16> = source.clone().try_into().unwrap();
        assert_eq!(fixed.tree_hash_root(), merkle_root(&source, 0));
    }
}
