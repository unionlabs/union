use std::cmp::Ordering;

use smallvec::{smallvec, SmallVec};

use super::{Ssz, UnionSelector, BYTES_PER_LENGTH_OFFSET, MAX_UNION_SELECTOR};

type SmallVec8<T> = SmallVec<[T; 8]>;

pub mod impls;
use std::fmt::Debug;

/// Partial variant of `std::iter::FromIterator`.
///
/// This trait is implemented for types which can be constructed from an iterator of decoded SSZ
/// values, but which may refuse values once a length limit is reached.
pub(crate) trait TryFromIter<T>: Sized {
    type Error;

    fn try_from_iter<I>(iter: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = T>;
}

/// Returned when SSZ decoding fails.
#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum DecodeError {
    #[error("invalid byte length, expected {expected} but found {found}")]
    InvalidByteLength { found: usize, expected: usize },
    #[error("invalid prefix byte length, expected {expected} but found {found}")]
    InvalidLengthPrefix { found: usize, expected: usize },
    /// A length offset pointed to a byte that was out-of-bounds (OOB).
    ///
    /// A bytes may be OOB for the following reasons:
    ///
    /// - It is `>= bytes.len()`.
    /// - When decoding variable length items, the 1st offset points "backwards" into the fixed
    ///   length items (i.e., `length[0] < BYTES_PER_LENGTH_OFFSET`).
    /// - When decoding variable-length items, the `n`'th offset was less than the `n-1`'th offset.
    #[error("offset {i} is out of bounds")]
    OutOfBoundsByte { i: usize },
    /// An offset points “backwards” into the fixed-bytes portion of the message, essentially
    /// double-decoding bytes that will also be decoded as fixed-length.
    ///
    /// https://notes.ethereum.org/ruKvDXl6QOW3gnqVYb8ezA?view#1-Offset-into-fixed-portion
    #[error("tried to read offset {0} into fixed-size portion")]
    OffsetIntoFixedPortion(usize),
    /// The first offset does not point to the byte that follows the fixed byte portion,
    /// essentially skipping a variable-length byte.
    ///
    /// https://notes.ethereum.org/ruKvDXl6QOW3gnqVYb8ezA?view#2-Skip-first-variable-byte
    #[error("first offset {0} does not point to the byte that follows the fixed-size portion")]
    OffsetSkipsVariableBytes(usize),
    /// An offset points to bytes prior to the previous offset. Depending on how you look at it,
    /// this either double-decodes bytes or makes the first offset a negative-length.
    ///
    /// https://notes.ethereum.org/ruKvDXl6QOW3gnqVYb8ezA?view#3-Offsets-are-decreasing
    #[error("offset {0} points to bytes prior to the previous offset")]
    OffsetsAreDecreasing(usize),
    /// An offset references byte indices that do not exist in the source bytes.
    ///
    /// https://notes.ethereum.org/ruKvDXl6QOW3gnqVYb8ezA?view#4-Offsets-are-out-of-bounds
    #[error("offset {0} points beyond the end of the source data")]
    OffsetOutOfBounds(usize),
    /// A variable-length list does not have a fixed portion that is cleanly divisible by
    /// `BYTES_PER_LENGTH_OFFSET`.
    #[error(
        "List length fixed-size portion not a multiple of {BYTES_PER_LENGTH_OFFSET} (found {0})"
    )]
    InvalidListFixedBytesLen(usize),
    /// The given bytes were invalid for some application-level reason.
    #[error("invalid bytes: {0}")]
    BytesInvalid(String),
    #[error("invalid vector length, expected {expected} but found {found}")]
    InvalidVectorLength { expected: usize, found: usize },
    /// The given union selector is out of bounds.
    #[error("union selector {0} is > {MAX_UNION_SELECTOR}")]
    UnionSelectorInvalid(u8),
}

/// Performs checks on the `offset` based upon the other parameters provided.
///
/// ## Detail
///
/// - `offset`: the offset bytes (e.g., result of `read_offset(..)`).
/// - `previous_offset`: unless this is the first offset in the SSZ object, the value of the
///   previously-read offset. Used to ensure offsets are not decreasing.
/// - `num_bytes`: the total number of bytes in the SSZ object. Used to ensure the offset is not
///   out of bounds.
/// - `num_fixed_bytes`: the number of fixed-bytes in the struct, if it is known. Used to ensure
///   that the first offset doesn't skip any variable bytes.
///
/// ## References
///
/// The checks here are derived from this document:
///
/// https://notes.ethereum.org/ruKvDXl6QOW3gnqVYb8ezA?view
pub(crate) fn sanitize_offset(
    offset: usize,
    previous_offset: Option<usize>,
    num_bytes: usize,
    num_fixed_bytes: Option<usize>,
) -> Result<usize, DecodeError> {
    if num_fixed_bytes.map_or(false, |fixed_bytes| offset < fixed_bytes) {
        Err(DecodeError::OffsetIntoFixedPortion(offset))
    } else if previous_offset.is_none()
        && num_fixed_bytes.map_or(false, |fixed_bytes| offset != fixed_bytes)
    {
        Err(DecodeError::OffsetSkipsVariableBytes(offset))
    } else if offset > num_bytes {
        Err(DecodeError::OffsetOutOfBounds(offset))
    } else if previous_offset.map_or(false, |prev| prev > offset) {
        Err(DecodeError::OffsetsAreDecreasing(offset))
    } else {
        Ok(offset)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Offset {
    position: usize,
    offset: usize,
}

/// Builds an `SszDecoder`.
///
/// The purpose of this struct is to split some SSZ bytes into individual slices. The builder is
/// then converted into a `SszDecoder` which decodes those values into object instances.
///
/// See [`SszDecoder`](struct.SszDecoder.html) for usage examples.
pub struct SszDecoderBuilder<'a> {
    bytes: &'a [u8],
    items: SmallVec8<&'a [u8]>,
    offsets: SmallVec8<Offset>,
    items_index: usize,
}

impl<'a> SszDecoderBuilder<'a> {
    /// Instantiate a new builder that should build a `SszDecoder` over the given `bytes` which
    /// are assumed to be the SSZ encoding of some object.
    #[must_use]
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            items: smallvec![],
            offsets: smallvec![],
            items_index: 0,
        }
    }

    /// Declares that some type `T` is the next item in `bytes`.
    pub fn register_type<T: Ssz>(&mut self) -> Result<(), DecodeError> {
        if let Some(fixed_len) = T::SSZ_FIXED_LEN {
            let start = self.items_index;
            self.items_index += fixed_len.get();

            let slice =
                self.bytes
                    .get(start..self.items_index)
                    .ok_or(DecodeError::InvalidByteLength {
                        found: self.bytes.len(),
                        expected: self.items_index,
                    })?;

            self.items.push(slice);
        } else {
            self.offsets.push(Offset {
                position: self.items.len(),
                offset: sanitize_offset(
                    read_offset(&self.bytes[self.items_index..])?,
                    self.offsets.last().map(|o| o.offset),
                    self.bytes.len(),
                    None,
                )?,
            });

            // Push an empty slice into items; it will be replaced later.
            self.items.push(&[]);

            self.items_index += BYTES_PER_LENGTH_OFFSET;
        }

        Ok(())
    }

    fn finalize(&mut self) -> Result<(), DecodeError> {
        if let Some(first_offset) = self.offsets.first().map(|o| o.offset) {
            // Check to ensure the first offset points to the byte immediately following the
            // fixed-length bytes.
            match first_offset.cmp(&self.items_index) {
                Ordering::Less => return Err(DecodeError::OffsetIntoFixedPortion(first_offset)),
                Ordering::Greater => {
                    return Err(DecodeError::OffsetSkipsVariableBytes(first_offset))
                }
                Ordering::Equal => (),
            }

            // Iterate through each pair of offsets, grabbing the slice between each of the offsets.
            for pair in self.offsets.windows(2) {
                let a = pair[0];
                let b = pair[1];

                self.items[a.position] = &self.bytes[a.offset..b.offset];
            }

            // Handle the last offset, pushing a slice from it's start through to the end of
            // `self.bytes`.
            if let Some(last) = self.offsets.last() {
                self.items[last.position] = &self.bytes[last.offset..];
            }
        } else {
            // If the container is fixed-length, ensure there are no excess bytes.
            if self.items_index != self.bytes.len() {
                return Err(DecodeError::InvalidByteLength {
                    found: self.bytes.len(),
                    expected: self.items_index,
                });
            }
        }

        Ok(())
    }

    /// Finalizes the builder, returning a `SszDecoder` that may be used to instantiate objects.
    pub fn build(mut self) -> Result<SszDecoder<'a>, DecodeError> {
        self.finalize()?;

        Ok(SszDecoder { items: self.items })
    }
}

/// Decodes some slices of SSZ into object instances. Should be instantiated using
/// [`SszDecoderBuilder`](struct.SszDecoderBuilder.html).
///
/// ## Example
///
/// ```rust
/// use ssz::{Ssz, decode::{SszDecoder, SszDecoderBuilder}};
/// use ssz::types::{typenum::U8, List};
///
/// #[derive(PartialEq, Debug, Ssz)]
/// struct Foo {
///     a: u64,
///     b: List<u16, U8>,
/// }
///
/// fn ssz_decoding_example() {
///     let foo = Foo {
///         a: 42,
///         b: vec![1, 3, 3, 7].try_into().unwrap()
///     };
///
///     let bytes = foo.as_ssz_bytes();
///
///     let mut builder = SszDecoderBuilder::new(&bytes);
///
///     builder.register_type::<u64>().unwrap();
///     builder.register_type::<List<u16, U8>>().unwrap();
///
///     let mut decoder = builder.build().unwrap();
///
///     let decoded_foo = Foo {
///         a: decoder.decode_next().unwrap(),
///         b: decoder.decode_next().unwrap(),
///     };
///
///     assert_eq!(foo, decoded_foo);
/// }
/// ```
pub struct SszDecoder<'a> {
    items: SmallVec8<&'a [u8]>,
}

impl<'a> SszDecoder<'a> {
    /// Decodes the next item.
    ///
    /// # Panics
    ///
    /// Panics when attempting to decode more items than actually exist.
    pub fn decode_next<T: Ssz>(&mut self) -> Result<T, DecodeError> {
        T::from_ssz_bytes(self.items.remove(0))
    }
}

/// Takes `bytes`, assuming it is the encoding for a SSZ union, and returns the union-selector and
/// the body (trailing bytes).
///
/// ## Errors
///
/// Returns an error if:
///
/// - `bytes` is empty.
/// - the union selector is not a valid value (i.e., larger than the maximum number of variants.
pub fn split_union_bytes(bytes: &[u8]) -> Result<(UnionSelector, &[u8]), DecodeError> {
    let (selector, tail) = bytes
        .split_first()
        .ok_or(DecodeError::OutOfBoundsByte { i: 0 })?;

    Ok((UnionSelector::new(*selector)?, tail))
}

/// Reads a `BYTES_PER_LENGTH_OFFSET`-byte length from `bytes`, where `bytes.len() >=
/// BYTES_PER_LENGTH_OFFSET`.
pub fn read_offset(bytes: &[u8]) -> Result<usize, DecodeError> {
    decode_offset(bytes.get(0..BYTES_PER_LENGTH_OFFSET).ok_or(
        DecodeError::InvalidLengthPrefix {
            found: bytes.len(),
            expected: BYTES_PER_LENGTH_OFFSET,
        },
    )?)
}

/// Decode bytes as a little-endian usize, returning an `Err` if `bytes.len() !=
/// BYTES_PER_LENGTH_OFFSET`.
fn decode_offset(bytes: &[u8]) -> Result<usize, DecodeError> {
    let len = bytes.len();
    let expected = BYTES_PER_LENGTH_OFFSET;

    if len != expected {
        Err(DecodeError::InvalidLengthPrefix {
            found: len,
            expected,
        })
    } else {
        let mut array: [u8; BYTES_PER_LENGTH_OFFSET] = std::default::Default::default();
        array.clone_from_slice(bytes);

        Ok(u32::from_le_bytes(array) as usize)
    }
}
