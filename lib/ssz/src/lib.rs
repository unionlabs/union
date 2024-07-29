//! Provides encoding (serialization) and decoding (deserialization) in the SimpleSerialize (SSZ)
//! format designed for use in Ethereum 2.0.
//!
//! Adheres to the Ethereum 2.0 [SSZ
//! specification](https://github.com/ethereum/consensus-specs/blob/v1.4.0/ssz/simple-serialize.md)
//! at v1.4.0.
//!
//! ## Example
//!
//! ```rust
//! use ssz::Ssz;
//! use ssz::types::{typenum::U8, List};
//!
//! #[derive(PartialEq, Debug, Ssz)]
//! struct Foo {
//!     a: u64,
//!     b: List<u16, U8>,
//! }
//!
//! fn ssz_encode_decode_example() {
//!     let foo = Foo {
//!         a: 42,
//!         b: vec![1, 3, 3, 7].try_into().unwrap()
//!     };
//!
//!     let ssz_bytes: Vec<u8> = foo.as_ssz_bytes();
//!
//!     let decoded_foo = Foo::from_ssz_bytes(&ssz_bytes).unwrap();
//!
//!     assert_eq!(foo, decoded_foo);
//! }
//! ```

pub mod decode;
pub mod encode;
mod union_selector;

pub mod tree_hash;

pub use union_selector::UnionSelector;

extern crate ssz_derive;
pub use ssz_derive::*;

use crate::{
    decode::DecodeError,
    tree_hash::{Hash256, TreeHashType},
    types::tree_hash::vec_tree_hash_root,
};

pub mod types;

/// The number of bytes used to represent an offset.
pub const BYTES_PER_LENGTH_OFFSET: usize = 4;
// REVIEW: Why not just use usize?
/// The maximum value that can be represented using `BYTES_PER_LENGTH_OFFSET`.
#[cfg(target_pointer_width = "32")]
pub const MAX_LENGTH_VALUE: usize = (u32::MAX >> (8 * (4 - BYTES_PER_LENGTH_OFFSET))) as usize;
#[cfg(target_pointer_width = "64")]
pub const MAX_LENGTH_VALUE: usize = (u64::MAX >> (8 * (8 - BYTES_PER_LENGTH_OFFSET))) as usize;

/// The number of bytes used to indicate the variant of a union.
pub const BYTES_PER_UNION_SELECTOR: usize = 1;
/// The highest possible union selector value (higher values are reserved for backwards compatible
/// extensions).
pub const MAX_UNION_SELECTOR: u8 = 127;

pub trait Ssz: Sized {
    /// Some(length) if this object has a fixed length, None if it is variable length.
    const SSZ_FIXED_LEN: Option<NonZeroUsize>;

    const TREE_HASH_TYPE: TreeHashType;

    fn tree_hash_root(&self) -> Hash256;

    /// Append the encoding `self` to `buf`.
    ///
    /// Note, variable length objects need only to append their "variable length" portion, they do
    /// not need to provide their offset.
    fn ssz_append(&self, buf: &mut Vec<u8>);

    /// Returns the size (in bytes) when `self` is serialized.
    ///
    /// Returns the same value as `self.as_ssz_bytes().len()` but this method is significantly more
    /// efficient.
    fn ssz_bytes_len(&self) -> NonZeroUsize;

    /// Returns the full-form encoding of this object.
    ///
    /// The default implementation of this method should suffice for most cases.
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];

        self.ssz_append(&mut buf);

        buf
    }

    /// Attempts to decode `Self` from `bytes`, returning a `DecodeError` on failure.
    ///
    /// The supplied bytes must be the exact length required to decode `Self`, excess bytes will
    /// result in an error.
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError>;
}

use std::{fmt::Debug, iter, num::NonZeroUsize, ops::Div};

use itertools::process_results;
use typenum::{NonZero, Unsigned, U};

use crate::{
    decode::{read_offset, sanitize_offset, TryFromIter},
    encode::SszEncoder,
    types::Vector,
};

macro_rules! impl_ssz_for_uint {
    ($T:ident) => {
        impl Ssz for $T {
            const SSZ_FIXED_LEN: Option<NonZeroUsize> =
                Some(option_unwrap!(NonZeroUsize::new(($T::BITS / 8) as usize)));

            const TREE_HASH_TYPE: TreeHashType = TreeHashType::Basic {
                size: {
                    if ($T::BITS / 8) > 32 {
                        panic!("invalid basic type size")
                    }

                    ($T::BITS / 8) as u8
                },
            };

            fn tree_hash_root(&self) -> Hash256 {
                let mut bytes = [0; crate::tree_hash::HASHSIZE];
                bytes[0..(($T::BITS / 8) as usize)].copy_from_slice(&self.to_le_bytes());
                bytes
            }

            fn ssz_bytes_len(&self) -> NonZeroUsize {
                // TODO: Replace with inline_const once stable
                NonZeroUsize::new(($T::BITS / 8) as usize).expect("value is > 0; qed;")
            }

            fn ssz_append(&self, buf: &mut Vec<u8>) {
                buf.extend_from_slice(&self.to_le_bytes());
            }

            fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
                let len = bytes.len();
                let expected = ($T::BITS / 8) as usize;

                if len != expected {
                    Err(DecodeError::InvalidByteLength {
                        found: len,
                        expected,
                    })
                } else {
                    Ok(Self::from_le_bytes(
                        bytes.try_into().expect("length is checked above"),
                    ))
                }
            }
        }
    };
}

impl_ssz_for_uint!(u8);
impl_ssz_for_uint!(u16);
impl_ssz_for_uint!(u32);
impl_ssz_for_uint!(u64);
impl_ssz_for_uint!(u128);
impl_ssz_for_uint!(usize);

/// Compute the encoded length of a vector-like sequence of `T`.
///
/// NOTE: Assumes that I is non-empty, panicking if it is.
pub(crate) fn sequence_ssz_bytes_len<'a, I, T>(iter: I) -> NonZeroUsize
where
    I: IntoIterator<Item = &'a T>,
    <I as IntoIterator>::IntoIter: Iterator + ExactSizeIterator,
    T: Ssz + 'a,
{
    let iter = iter.into_iter();
    // Compute length before doing any iteration.
    let length = iter.len();
    NonZeroUsize::new(
        T::SSZ_FIXED_LEN
            .map(|fixed_len| fixed_len.get() * length)
            .unwrap_or_else(|| {
                iter.map(|item| item.ssz_bytes_len().get()).sum::<usize>()
                    + (BYTES_PER_LENGTH_OFFSET * length)
            }),
    )
    .expect("iterator should not be empty")
}

/// Ssz a vector-like sequence of `T`.
pub fn sequence_ssz_append<'a, I, T>(iter: I, buf: &mut Vec<u8>)
where
    I: IntoIterator<Item = &'a T>,
    <I as IntoIterator>::IntoIter: Iterator + ExactSizeIterator,
    T: Ssz + 'a,
{
    let iter = iter.into_iter();

    match T::SSZ_FIXED_LEN {
        Some(fixed_len) => {
            buf.reserve(fixed_len.get() * iter.len());

            for item in iter {
                item.ssz_append(buf);
            }
        }
        None => {
            let mut encoder = SszEncoder::container(buf, iter.len() * BYTES_PER_LENGTH_OFFSET);

            for item in iter {
                encoder.append(item);
            }

            encoder.finalize();
        }
    }
}

impl<T: Ssz, const N: usize> Ssz for [T; N]
where
    typenum::Const<N>: typenum::ToUInt,
    U<N>: Unsigned + NonZero,
{
    const SSZ_FIXED_LEN: Option<NonZeroUsize> = match T::SSZ_FIXED_LEN {
        Some(len) => match NonZeroUsize::new(len.get() * N) {
            Some(len) => Some(len),
            None => unreachable!(),
        },
        None => None,
    };

    const TREE_HASH_TYPE: TreeHashType = TreeHashType::Vector;

    fn tree_hash_root(&self) -> Hash256 {
        vec_tree_hash_root::<T, U<N>>(self)
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        sequence_ssz_append::<_, T>(self, buf);
    }

    fn ssz_bytes_len(&self) -> NonZeroUsize {
        sequence_ssz_bytes_len(self)
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

                    if num_items != N {
                        return Err(DecodeError::InvalidByteLength {
                            found: num_items,
                            expected: N,
                        });
                    }

                    // REVIEW: Potential for DOS? the length is checked above, so it should be fine?
                    bytes
                        .chunks(fixed_len.get())
                        .map(|chunk| T::from_ssz_bytes(chunk))
                        .collect::<Result<Vec<T>, _>>()
                        .and_then(|vec| {
                            vec.try_into()
                                .map_err(|e: Vec<T>| DecodeError::InvalidVectorLength {
                                    expected: N,
                                    found: e.len(),
                                })
                        })
                }
                None => decode_list_of_variable_length_items::<T, Vector<T, typenum::U<N>>>(
                    bytes,
                    Some(N),
                )
                .map(Into::into),
            }
        }
    }
}

impl Ssz for bool {
    const SSZ_FIXED_LEN: Option<NonZeroUsize> = match NonZeroUsize::new(1) {
        Some(x) => Some(x),
        None => ::core::unreachable!(),
    };

    const TREE_HASH_TYPE: TreeHashType = u8::TREE_HASH_TYPE;

    fn tree_hash_root(&self) -> Hash256 {
        (*self as u8).tree_hash_root()
    }

    fn ssz_bytes_len(&self) -> NonZeroUsize {
        NonZeroUsize::new(1).unwrap()
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&(*self as u8).to_le_bytes());
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let len = bytes.len();

        if len != 1 {
            Err(DecodeError::InvalidByteLength {
                found: len,
                expected: 1,
            })
        } else {
            match bytes[0] {
                0b0000_0000 => Ok(false),
                0b0000_0001 => Ok(true),
                _ => Err(DecodeError::BytesInvalid(format!(
                    "Out-of-range for boolean: {}",
                    bytes[0]
                ))),
            }
        }
    }
}

impl Ssz for NonZeroUsize {
    const SSZ_FIXED_LEN: Option<NonZeroUsize> = usize::SSZ_FIXED_LEN;

    const TREE_HASH_TYPE: TreeHashType = usize::TREE_HASH_TYPE;

    fn tree_hash_root(&self) -> Hash256 {
        self.get().tree_hash_root()
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        self.get().ssz_append(buf);
    }

    fn ssz_bytes_len(&self) -> NonZeroUsize {
        std::mem::size_of::<usize>().try_into().unwrap()
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let x = usize::from_ssz_bytes(bytes)?;

        // TODO: Just use ::new().ok_or_else() lol
        if x == 0 {
            Err(DecodeError::BytesInvalid(
                "NonZeroUsize cannot be zero.".to_string(),
            ))
        } else {
            // `unwrap` is safe here as `NonZeroUsize::new()` succeeds if `x > 0` and this path
            // never executes when `x == 0`.
            Ok(NonZeroUsize::new(x).unwrap())
        }
    }
}

/// Decodes `bytes` as if it were a list of variable-length items.
///
/// The `ssz::SszDecoder` can also perform this functionality, however this function is
/// significantly faster as it is optimized to read same-typed items whilst `ssz::SszDecoder`
/// supports reading items of differing types.
pub(crate) fn decode_list_of_variable_length_items<T: Ssz, Container: TryFromIter<T>>(
    bytes: &[u8],
    max_len: Option<usize>,
) -> Result<Container, DecodeError>
where
    <Container as TryFromIter<T>>::Error: Debug,
{
    if bytes.is_empty() {
        return Container::try_from_iter(iter::empty()).map_err(|e| {
            DecodeError::BytesInvalid(format!("Error trying to collect empty list: {:?}", e))
        });
    }

    let first_offset = read_offset(bytes)?;
    sanitize_offset(first_offset, None, bytes.len(), Some(first_offset))?;

    if first_offset % BYTES_PER_LENGTH_OFFSET != 0 || first_offset < BYTES_PER_LENGTH_OFFSET {
        return Err(DecodeError::InvalidListFixedBytesLen(first_offset));
    }

    let num_items = first_offset / BYTES_PER_LENGTH_OFFSET;

    if max_len.map_or(false, |max| num_items > max) {
        return Err(DecodeError::BytesInvalid(format!(
            "Variable length list of {} items exceeds maximum of {:?}",
            num_items, max_len
        )));
    }

    let mut offset = first_offset;
    process_results(
        (1..=num_items).map(|i| {
            let slice_option = if i == num_items {
                bytes.get(offset..)
            } else {
                let start = offset;

                let next_offset = read_offset(&bytes[(i * BYTES_PER_LENGTH_OFFSET)..])?;
                offset =
                    sanitize_offset(next_offset, Some(offset), bytes.len(), Some(first_offset))?;

                bytes.get(start..offset)
            };

            let slice = slice_option.ok_or(DecodeError::OutOfBoundsByte { i: offset })?;
            T::from_ssz_bytes(slice)
        }),
        |iter| Container::try_from_iter(iter),
    )?
    .map_err(|e| DecodeError::BytesInvalid(format!("Error collecting into container: {:?}", e)))
}

// Useful in const contexts in place of `.unwrap()`
macro_rules! option_unwrap {
    ($expr:expr) => {{
        // assign to a const here so this can't be called in non-const contexts
        const _: () = match $expr {
            Some(_) => {}
            None => panic!("called `Option::unwrap()` on an `None` value"),
        };

        match $expr {
            Some(some) => some,
            None => panic!("called `Option::unwrap()` on an `None` value"),
        }
    }};
}
pub(crate) use option_unwrap;

#[cfg(test)]
mod tests {
    use super::*;

    mod encode {
        use super::*;

        #[test]
        fn ssz_encode_u8() {
            assert_eq!(0_u8.as_ssz_bytes(), vec![0]);
            assert_eq!(1_u8.as_ssz_bytes(), vec![1]);
            assert_eq!(100_u8.as_ssz_bytes(), vec![100]);
            assert_eq!(255_u8.as_ssz_bytes(), vec![255]);
        }

        #[test]
        fn ssz_encode_u16() {
            assert_eq!(1_u16.as_ssz_bytes(), vec![1, 0]);
            assert_eq!(100_u16.as_ssz_bytes(), vec![100, 0]);
            assert_eq!((1_u16 << 8).as_ssz_bytes(), vec![0, 1]);
            assert_eq!(65535_u16.as_ssz_bytes(), vec![255, 255]);
        }

        #[test]
        fn ssz_encode_u32() {
            assert_eq!(1_u32.as_ssz_bytes(), vec![1, 0, 0, 0]);
            assert_eq!(100_u32.as_ssz_bytes(), vec![100, 0, 0, 0]);
            assert_eq!((1_u32 << 16).as_ssz_bytes(), vec![0, 0, 1, 0]);
            assert_eq!((1_u32 << 24).as_ssz_bytes(), vec![0, 0, 0, 1]);
            assert_eq!((!0_u32).as_ssz_bytes(), vec![255, 255, 255, 255]);
        }

        #[test]
        fn ssz_encode_u64() {
            assert_eq!(1_u64.as_ssz_bytes(), vec![1, 0, 0, 0, 0, 0, 0, 0]);
            assert_eq!(
                (!0_u64).as_ssz_bytes(),
                vec![255, 255, 255, 255, 255, 255, 255, 255]
            );
        }

        #[test]
        fn ssz_encode_usize() {
            assert_eq!(1_usize.as_ssz_bytes(), vec![1, 0, 0, 0, 0, 0, 0, 0]);
            assert_eq!(
                (!0_usize).as_ssz_bytes(),
                vec![255, 255, 255, 255, 255, 255, 255, 255]
            );
        }

        #[test]
        fn ssz_encode_bool() {
            assert_eq!(true.as_ssz_bytes(), vec![1]);
            assert_eq!(false.as_ssz_bytes(), vec![0]);
        }

        #[test]
        fn ssz_encode_u8_array_4() {
            assert_eq!([0_u8, 0, 0, 0].as_ssz_bytes(), vec![0; 4]);
            assert_eq!([1_u8, 0, 0, 0].as_ssz_bytes(), vec![1, 0, 0, 0]);
            assert_eq!([1_u8, 2, 3, 4].as_ssz_bytes(), vec![1, 2, 3, 4]);
        }
    }

    mod decode {
        use super::*;

        // Note: decoding of valid bytes is generally tested "indirectly" in the `/tests` dir, by
        // encoding then decoding the element.

        #[test]
        fn invalid_u8_array_4() {
            assert_eq!(
                <[u8; 4]>::from_ssz_bytes(&[0; 3]),
                Err(DecodeError::InvalidByteLength {
                    found: 3,
                    expected: 4
                })
            );

            assert_eq!(
                <[u8; 4]>::from_ssz_bytes(&[0; 5]),
                Err(DecodeError::InvalidByteLength {
                    found: 5,
                    expected: 4
                })
            );
        }

        #[test]
        fn invalid_bool() {
            assert_eq!(
                bool::from_ssz_bytes(&[0; 2]),
                Err(DecodeError::InvalidByteLength {
                    found: 2,
                    expected: 1
                })
            );

            assert_eq!(
                bool::from_ssz_bytes(&[]),
                Err(DecodeError::InvalidByteLength {
                    found: 0,
                    expected: 1
                })
            );

            if let Err(DecodeError::BytesInvalid(_)) = bool::from_ssz_bytes(&[2]) {
                // Success.
            } else {
                panic!("Did not return error on invalid bool val")
            }
        }

        #[test]
        fn u16() {
            assert_eq!(u16::from_ssz_bytes(&[0, 0]), Ok(0));
            assert_eq!(u16::from_ssz_bytes(&[16, 0]), Ok(16));
            assert_eq!(u16::from_ssz_bytes(&[0, 1]), Ok(256));
            assert_eq!(u16::from_ssz_bytes(&[255, 255]), Ok(65535));

            assert_eq!(
                u16::from_ssz_bytes(&[255]),
                Err(DecodeError::InvalidByteLength {
                    found: 1,
                    expected: 2
                })
            );

            assert_eq!(
                u16::from_ssz_bytes(&[]),
                Err(DecodeError::InvalidByteLength {
                    found: 0,
                    expected: 2
                })
            );

            assert_eq!(
                u16::from_ssz_bytes(&[0, 1, 2]),
                Err(DecodeError::InvalidByteLength {
                    found: 3,
                    expected: 2
                })
            );
        }
    }
}
