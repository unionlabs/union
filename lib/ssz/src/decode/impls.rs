use core::num::NonZeroUsize;
use std::{fmt::Debug, iter, sync::Arc};

use itertools::process_results;

use super::*;
use crate::decode::try_from_iter::TryCollect;

macro_rules! impl_decodable_for_uint {
    ($type: ident) => {
        impl Decode for $type {
            fn is_ssz_fixed_len() -> bool {
                true
            }

            fn ssz_fixed_len() -> usize {
                ($type::BITS / 8) as usize
            }

            fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
                let len = bytes.len();
                let expected = <Self as Decode>::ssz_fixed_len();

                if len != expected {
                    Err(DecodeError::InvalidByteLength { len, expected })
                } else {
                    Ok(Self::from_le_bytes(
                        bytes.try_into().expect("length is checked above"),
                    ))
                }
            }
        }
    };
}

impl_decodable_for_uint!(u8);
impl_decodable_for_uint!(u16);
impl_decodable_for_uint!(u32);
impl_decodable_for_uint!(u64);
impl_decodable_for_uint!(u128);
impl_decodable_for_uint!(usize);

macro_rules! impl_decode_for_tuples {
    ($(
        $Tuple:ident {
            $(($idx:tt) -> $T:ident)+
        }
    )+) => {
        $(
            impl<$($T: Decode),+> Decode for ($($T,)+) {
                fn is_ssz_fixed_len() -> bool {
                    $(
                        <$T as Decode>::is_ssz_fixed_len() &&
                    )*
                        true
                }

                fn ssz_fixed_len() -> usize {
                    if <Self as Decode>::is_ssz_fixed_len() {
                        $(
                            <$T as Decode>::ssz_fixed_len() +
                        )*
                            0
                    } else {
                        BYTES_PER_LENGTH_OFFSET
                    }
                }

                fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
                    let mut builder = SszDecoderBuilder::new(bytes);

                    $(
                        builder.register_type::<$T>()?;
                    )*

                    let mut decoder = builder.build()?;

                    Ok(($(
                            decoder.decode_next::<$T>()?,
                        )*
                    ))
                }
            }
        )+
    }
}

impl_decode_for_tuples! {
    Tuple2 {
        (0) -> A
        (1) -> B
    }
    Tuple3 {
        (0) -> A
        (1) -> B
        (2) -> C
    }
    Tuple4 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
    }
    Tuple5 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
    }
    Tuple6 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
    }
    Tuple7 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
    }
    Tuple8 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
    }
    Tuple9 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
        (8) -> I
    }
    Tuple10 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
        (8) -> I
        (9) -> J
    }
    Tuple11 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
        (8) -> I
        (9) -> J
        (10) -> K
    }
    Tuple12 {
        (0) -> A
        (1) -> B
        (2) -> C
        (3) -> D
        (4) -> E
        (5) -> F
        (6) -> G
        (7) -> H
        (8) -> I
        (9) -> J
        (10) -> K
        (11) -> L
    }
}

impl Decode for bool {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        1
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let len = bytes.len();
        let expected = <Self as Decode>::ssz_fixed_len();

        if len != expected {
            Err(DecodeError::InvalidByteLength { len, expected })
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

impl Decode for NonZeroUsize {
    fn is_ssz_fixed_len() -> bool {
        <usize as Decode>::is_ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        <usize as Decode>::ssz_fixed_len()
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let x = usize::from_ssz_bytes(bytes)?;

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

impl<T: Decode> Decode for Option<T> {
    fn is_ssz_fixed_len() -> bool {
        false
    }
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let (selector, body) = split_union_bytes(bytes)?;
        match selector.into() {
            0u8 => Ok(None),
            1u8 => <T as Decode>::from_ssz_bytes(body).map(Option::Some),
            other => Err(DecodeError::UnionSelectorInvalid(other)),
        }
    }
}

impl<T: Decode> Decode for Arc<T> {
    fn is_ssz_fixed_len() -> bool {
        T::is_ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        T::ssz_fixed_len()
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        T::from_ssz_bytes(bytes).map(Arc::new)
    }
}

// Copied directly from ssz::types::FixedVector
impl<T: Decode + Debug, const N: usize> Decode for [T; N] {
    fn is_ssz_fixed_len() -> bool {
        T::is_ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        if Self::is_ssz_fixed_len() {
            T::ssz_fixed_len() * N
        } else {
            BYTES_PER_LENGTH_OFFSET
        }
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            Err(DecodeError::InvalidByteLength {
                len: 0,
                expected: 1,
            })
        } else if T::is_ssz_fixed_len() {
            let num_items = bytes
                .len()
                .checked_div(T::ssz_fixed_len())
                .ok_or(DecodeError::ZeroLengthItem)?;

            if num_items != N {
                return Err(DecodeError::InvalidByteLength {
                    len: num_items,
                    expected: N,
                });
            }

            // REVIEW: Potential for DOS? the length is checked above, so it should be fine?
            bytes
                .chunks(T::ssz_fixed_len())
                .map(|chunk| T::from_ssz_bytes(chunk))
                .collect::<Result<Vec<T>, _>>()
                .and_then(|vec| {
                    vec.try_into().map_err(|e: Vec<T>| {
                        DecodeError::BytesInvalid(format!(
                            "Wrong number of FixedVector elements, expected {N} but found {}: {e:?}",
                            e.len()
                        ))
                    })
                })
        } else {
            let vec: Vec<T> = decode_list_of_variable_length_items(bytes, Some(N))?;
            vec.try_into().map_err(|e| {
                DecodeError::BytesInvalid(format!("Wrong number of FixedVector elements: {:?}", e))
            })
        }
    }
}

/// Decodes `bytes` as if it were a list of variable-length items.
///
/// The `ssz::SszDecoder` can also perform this functionality, however this function is
/// significantly faster as it is optimized to read same-typed items whilst `ssz::SszDecoder`
/// supports reading items of differing types.
pub fn decode_list_of_variable_length_items<T: Decode, Container: TryFromIter<T>>(
    bytes: &[u8],
    max_len: Option<usize>,
) -> Result<Container, DecodeError> {
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
        |iter| iter.try_collect(),
    )?
    .map_err(|e| DecodeError::BytesInvalid(format!("Error collecting into container: {:?}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: decoding of valid bytes is generally tested "indirectly" in the `/tests` dir, by
    // encoding then decoding the element.

    #[test]
    fn invalid_u8_array_4() {
        assert_eq!(
            <[u8; 4]>::from_ssz_bytes(&[0; 3]),
            Err(DecodeError::InvalidByteLength {
                len: 3,
                expected: 4
            })
        );

        assert_eq!(
            <[u8; 4]>::from_ssz_bytes(&[0; 5]),
            Err(DecodeError::InvalidByteLength {
                len: 5,
                expected: 4
            })
        );
    }

    #[test]
    fn invalid_bool() {
        assert_eq!(
            bool::from_ssz_bytes(&[0; 2]),
            Err(DecodeError::InvalidByteLength {
                len: 2,
                expected: 1
            })
        );

        assert_eq!(
            bool::from_ssz_bytes(&[]),
            Err(DecodeError::InvalidByteLength {
                len: 0,
                expected: 1
            })
        );

        if let Err(DecodeError::BytesInvalid(_)) = bool::from_ssz_bytes(&[2]) {
            // Success.
        } else {
            panic!("Did not return error on invalid bool val")
        }
    }

    // #[test]
    // fn invalid_h256() {
    //     assert_eq!(
    //         H256::from_ssz_bytes(&[0; 33]),
    //         Err(DecodeError::InvalidByteLength {
    //             len: 33,
    //             expected: 32
    //         })
    //     );

    //     assert_eq!(
    //         H256::from_ssz_bytes(&[0; 31]),
    //         Err(DecodeError::InvalidByteLength {
    //             len: 31,
    //             expected: 32
    //         })
    //     );
    // }

    #[test]
    fn u16() {
        assert_eq!(<u16>::from_ssz_bytes(&[0, 0]), Ok(0));
        assert_eq!(<u16>::from_ssz_bytes(&[16, 0]), Ok(16));
        assert_eq!(<u16>::from_ssz_bytes(&[0, 1]), Ok(256));
        assert_eq!(<u16>::from_ssz_bytes(&[255, 255]), Ok(65535));

        assert_eq!(
            <u16>::from_ssz_bytes(&[255]),
            Err(DecodeError::InvalidByteLength {
                len: 1,
                expected: 2
            })
        );

        assert_eq!(
            <u16>::from_ssz_bytes(&[]),
            Err(DecodeError::InvalidByteLength {
                len: 0,
                expected: 2
            })
        );

        assert_eq!(
            <u16>::from_ssz_bytes(&[0, 1, 2]),
            Err(DecodeError::InvalidByteLength {
                len: 3,
                expected: 2
            })
        );
    }

    #[test]
    fn tuple() {
        assert_eq!(<(u16, u16)>::from_ssz_bytes(&[0, 0, 0, 0]), Ok((0, 0)));
        assert_eq!(<(u16, u16)>::from_ssz_bytes(&[16, 0, 17, 0]), Ok((16, 17)));
        assert_eq!(<(u16, u16)>::from_ssz_bytes(&[0, 1, 2, 0]), Ok((256, 2)));
        assert_eq!(
            <(u16, u16)>::from_ssz_bytes(&[255, 255, 0, 0]),
            Ok((65535, 0))
        );
    }
}
