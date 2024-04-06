use std::cmp::Ordering;

use smallvec::{smallvec, SmallVec};

use super::*;

type SmallVec8<T> = SmallVec<[T; 8]>;

pub mod impls;
pub mod try_from_iter;

/// Returned when SSZ decoding fails.
#[derive(Debug, PartialEq, Clone)]
pub enum DecodeError {
    /// The bytes supplied were too short to be decoded into the specified type.
    InvalidByteLength { len: usize, expected: usize },
    /// The given bytes were too short to be read as a length prefix.
    InvalidLengthPrefix { len: usize, expected: usize },
    /// A length offset pointed to a byte that was out-of-bounds (OOB).
    ///
    /// A bytes may be OOB for the following reasons:
    ///
    /// - It is `>= bytes.len()`.
    /// - When decoding variable length items, the 1st offset points "backwards" into the fixed
    /// length items (i.e., `length[0] < BYTES_PER_LENGTH_OFFSET`).
    /// - When decoding variable-length items, the `n`'th offset was less than the `n-1`'th offset.
    OutOfBoundsByte { i: usize },
    /// An offset points “backwards” into the fixed-bytes portion of the message, essentially
    /// double-decoding bytes that will also be decoded as fixed-length.
    ///
    /// https://notes.ethereum.org/ruKvDXl6QOW3gnqVYb8ezA?view#1-Offset-into-fixed-portion
    OffsetIntoFixedPortion(usize),
    /// The first offset does not point to the byte that follows the fixed byte portion,
    /// essentially skipping a variable-length byte.
    ///
    /// https://notes.ethereum.org/ruKvDXl6QOW3gnqVYb8ezA?view#2-Skip-first-variable-byte
    OffsetSkipsVariableBytes(usize),
    /// An offset points to bytes prior to the previous offset. Depending on how you look at it,
    /// this either double-decodes bytes or makes the first offset a negative-length.
    ///
    /// https://notes.ethereum.org/ruKvDXl6QOW3gnqVYb8ezA?view#3-Offsets-are-decreasing
    OffsetsAreDecreasing(usize),
    /// An offset references byte indices that do not exist in the source bytes.
    ///
    /// https://notes.ethereum.org/ruKvDXl6QOW3gnqVYb8ezA?view#4-Offsets-are-out-of-bounds
    OffsetOutOfBounds(usize),
    /// A variable-length list does not have a fixed portion that is cleanly divisible by
    /// `BYTES_PER_LENGTH_OFFSET`.
    InvalidListFixedBytesLen(usize),
    /// Some item has a `ssz_fixed_len` of zero. This is illegal.
    ZeroLengthItem,
    /// The given bytes were invalid for some application-level reason.
    BytesInvalid(String),
    /// The given union selector is out of bounds.
    UnionSelectorInvalid(u8),
}

/// Performs checks on the `offset` based upon the other parameters provided.
///
/// ## Detail
///
/// - `offset`: the offset bytes (e.g., result of `read_offset(..)`).
/// - `previous_offset`: unless this is the first offset in the SSZ object, the value of the
/// previously-read offset. Used to ensure offsets are not decreasing.
/// - `num_bytes`: the total number of bytes in the SSZ object. Used to ensure the offset is not
/// out of bounds.
/// - `num_fixed_bytes`: the number of fixed-bytes in the struct, if it is known. Used to ensure
/// that the first offset doesn't skip any variable bytes.
///
/// ## References
///
/// The checks here are derived from this document:
///
/// https://notes.ethereum.org/ruKvDXl6QOW3gnqVYb8ezA?view
pub fn sanitize_offset(
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

/// Provides SSZ decoding (de-serialization) via the `from_ssz_bytes(&bytes)` method.
///
/// See `examples/` for manual implementations or the crate root for implementations using
/// `#[derive(Decode)]`.
pub trait Decode: Sized {
    /// Returns `true` if this object has a fixed-length.
    ///
    /// I.e., there are no variable length items in this object or any of it's contained objects.
    fn is_ssz_fixed_len() -> bool;

    /// The number of bytes this object occupies in the fixed-length portion of the SSZ bytes.
    ///
    /// By default, this is set to `BYTES_PER_LENGTH_OFFSET` which is suitable for variable length
    /// objects, but not fixed-length objects. Fixed-length objects _must_ return a value which
    /// represents their length.
    fn ssz_fixed_len() -> usize {
        BYTES_PER_LENGTH_OFFSET
    }

    /// Attempts to decode `Self` from `bytes`, returning a `DecodeError` on failure.
    ///
    /// The supplied bytes must be the exact length required to decode `Self`, excess bytes will
    /// result in an error.
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError>;
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
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            items: smallvec![],
            offsets: smallvec![],
            items_index: 0,
        }
    }

    /// Registers a variable-length object as the next item in `bytes`, without specifying the
    /// actual type.
    ///
    /// ## Notes
    ///
    /// Use of this function is generally discouraged since it cannot detect if some type changes
    /// from variable to fixed length.
    ///
    /// Use `Self::register_type` wherever possible.
    pub fn register_anonymous_variable_length_item(&mut self) -> Result<(), DecodeError> {
        struct Anonymous;

        impl Decode for Anonymous {
            fn is_ssz_fixed_len() -> bool {
                false
            }

            fn from_ssz_bytes(_bytes: &[u8]) -> Result<Self, DecodeError> {
                unreachable!("Anonymous should never be decoded")
            }
        }

        self.register_type::<Anonymous>()
    }

    /// Declares that some type `T` is the next item in `bytes`.
    pub fn register_type<T: Decode>(&mut self) -> Result<(), DecodeError> {
        self.register_type_parameterized(T::is_ssz_fixed_len(), T::ssz_fixed_len())
    }

    /// Declares that a type with the given parameters is the next item in `bytes`.
    pub fn register_type_parameterized(
        &mut self,
        is_ssz_fixed_len: bool,
        ssz_fixed_len: usize,
    ) -> Result<(), DecodeError> {
        if is_ssz_fixed_len {
            let start = self.items_index;
            self.items_index += ssz_fixed_len;

            let slice =
                self.bytes
                    .get(start..self.items_index)
                    .ok_or(DecodeError::InvalidByteLength {
                        len: self.bytes.len(),
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
                self.items[last.position] = &self.bytes[last.offset..]
            }
        } else {
            // If the container is fixed-length, ensure there are no excess bytes.
            if self.items_index != self.bytes.len() {
                return Err(DecodeError::InvalidByteLength {
                    len: self.bytes.len(),
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
/// use ssz::{Decode, Encode, SszDecoder, SszDecoderBuilder};
/// use ssz::types::{typenum::U8, VariableList};
///
/// #[derive(PartialEq, Debug, Encode, Decode)]
/// struct Foo {
///     a: u64,
///     b: VariableList<u16, U8>,
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
///     builder.register_type::<VariableList<u16, U8>>().unwrap();
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
///
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
    pub fn decode_next<T: Decode>(&mut self) -> Result<T, DecodeError> {
        self.decode_next_with(|slice| T::from_ssz_bytes(slice))
    }

    /// Decodes the next item using the provided function.
    pub fn decode_next_with<T, F>(&mut self, f: F) -> Result<T, DecodeError>
    where
        F: FnOnce(&'a [u8]) -> Result<T, DecodeError>,
    {
        f(self.items.remove(0))
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
    let selector = bytes
        .first()
        .copied()
        .ok_or(DecodeError::OutOfBoundsByte { i: 0 })
        .and_then(UnionSelector::new)?;
    let body = bytes
        .get(1..)
        .ok_or(DecodeError::OutOfBoundsByte { i: 1 })?;
    Ok((selector, body))
}

/// Reads a `BYTES_PER_LENGTH_OFFSET`-byte length from `bytes`, where `bytes.len() >=
/// BYTES_PER_LENGTH_OFFSET`.
pub fn read_offset(bytes: &[u8]) -> Result<usize, DecodeError> {
    decode_offset(bytes.get(0..BYTES_PER_LENGTH_OFFSET).ok_or(
        DecodeError::InvalidLengthPrefix {
            len: bytes.len(),
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
        Err(DecodeError::InvalidLengthPrefix { len, expected })
    } else {
        let mut array: [u8; BYTES_PER_LENGTH_OFFSET] = std::default::Default::default();
        array.clone_from_slice(bytes);

        Ok(u32::from_le_bytes(array) as usize)
    }
}
