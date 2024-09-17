use core::marker::PhantomData;
use std::num::NonZeroUsize;

use derivative::Derivative;
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};
use smallvec::{smallvec, SmallVec, ToSmallVec};
use typenum::Unsigned;

use crate::{
    tree_hash::Hash256,
    types::{tree_hash::bitfield_bytes_tree_hash_root, Error},
    DecodeError, Ssz,
};

/// Maximum number of bytes to store on the stack in a bitfield's `SmallVec`.
///
/// The default of 32 bytes is enough to take us through to ~500K validators, as the byte length of
/// attestation bitfields is roughly `N // 32 slots // 64 committes // 8 bits`.
pub const SMALLVEC_LEN: usize = 32;

/// A marker trait applied to `Variable` and `Fixed` that defines the behaviour of a `Bitfield`.
pub trait BitfieldBehaviour {}

/// A marker struct used to declare SSZ `Variable` behaviour on a `Bitfield`.
///
/// See the [`Bitfield`](struct.Bitfield.html) docs for usage.
pub struct Variable<N> {
    _phantom: PhantomData<N>,
}

/// A marker struct used to declare SSZ `Fixed` behaviour on a `Bitfield`.
///
/// See the [`Bitfield`](struct.Bitfield.html) docs for usage.
pub struct Fixed<N> {
    _phantom: PhantomData<N>,
}

impl<N: Unsigned + Clone> BitfieldBehaviour for Variable<N> {}
impl<N: Unsigned + Clone> BitfieldBehaviour for Fixed<N> {}

/// A heap-allocated, ordered, variable-length collection of `bool` values, limited to `N` bits.
pub type BitList<N> = Bitfield<Variable<N>>;

/// A heap-allocated, ordered, fixed-length collection of `bool` values, with `N` bits.
///
/// See [Bitfield](struct.Bitfield.html) documentation.
pub type BitVector<N> = Bitfield<Fixed<N>>;

/// A heap-allocated, ordered, fixed-length collection of `bool` values. Use of
/// [`BitList`](type.BitList.html) or [`BitVector`](type.BitVector.html) type aliases is preferred
/// over direct use of this struct.
///
/// The `T` type parameter is used to define length behaviour with the `Variable` or `Fixed` marker
/// structs.
///
/// The length of the Bitfield is set at instantiation (i.e., runtime, not compile time). However,
/// use with a `Variable` sets a type-level (i.e., compile-time) maximum length and `Fixed`
/// provides a type-level fixed length.
///
/// ## Example
///
/// The example uses the following crate-level type aliases:
///
/// - `BitList<N>` is an alias for `Bitfield<Variable<N>>`
/// - `BitVector<N>` is an alias for `Bitfield<Fixed<N>>`
///
/// ```
/// use ssz::types::{BitVector, BitList, typenum};
///
/// // `BitList` has a type-level maximum length. The length of the list is specified at runtime
/// // and it must be less than or equal to `N`. After instantiation, `BitList` cannot grow or
/// // shrink.
/// type BitList8 = BitList<typenum::U8>;
///
/// // Creating a `BitList` with a larger-than-`N` capacity returns `None`.
/// assert!(BitList8::with_capacity(9).is_none());
///
/// let mut bitlist = BitList8::with_capacity(4).unwrap();  // `BitList` permits a capacity of less than the maximum.
/// assert!(bitlist.set(3, true).is_ok());  // Setting inside the instantiation capacity is permitted.
/// assert!(bitlist.set(5, true).is_err());  // Setting outside that capacity is not.
///
/// // `BitVector` has a type-level fixed length. Unlike `BitList`, it cannot be instantiated with a custom length
/// // or grow/shrink.
/// type BitVector8 = BitVector<typenum::U8>;
///
/// let mut bitvector = BitVector8::new();
/// assert_eq!(bitvector.len(), 8); // `BitVector` length is fixed at the type-level.
/// assert!(bitvector.set(7, true).is_ok()); // Setting inside the capacity is permitted.
/// assert!(bitvector.set(9, true).is_err()); // Setting outside the capacity is not.
/// ```
///
/// ## Note
///
/// The internal representation of the bitfield is the same as that required by SSZ. The lowest
/// byte (by `Vec` index) stores the lowest bit-indices and the right-most bit stores the lowest
/// bit-index. E.g., `smallvec![0b0000_0001, 0b0000_0010]` has bits `0, 9` set.
#[derive(Derivative)]
#[derivative(
    Debug(bound = ""),
    Clone(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    Hash(bound = "")
)]
pub struct Bitfield<T> {
    bytes: SmallVec<[u8; SMALLVEC_LEN]>,
    len: usize,
    _phantom: PhantomData<T>,
}

impl<N: Unsigned + Clone> Bitfield<Variable<N>> {
    /// Instantiate with capacity for `num_bits` boolean values. The length cannot be grown or
    /// shrunk after instantiation.
    ///
    /// All bits are initialized to `false`.
    ///
    /// Returns `None` if `num_bits > N`.
    #[must_use]
    pub fn with_capacity(num_bits: usize) -> Option<Self> {
        if num_bits <= N::USIZE {
            Some(Self {
                bytes: smallvec![0; bytes_for_bit_len(num_bits)],
                len: num_bits,
                _phantom: PhantomData,
            })
        } else {
            None
        }
    }

    /// Consumes `self`, returning a serialized representation.
    ///
    /// The output is faithful to the SSZ encoding of `self`, such that a leading `true` bit is
    /// used to indicate the length of the bitfield.
    ///
    /// ## Example
    /// ```
    /// use ssz::types::{BitList, typenum};
    /// use smallvec::SmallVec;
    ///
    /// type BitList8 = BitList<typenum::U8>;
    ///
    /// let b = BitList8::with_capacity(4).unwrap();
    ///
    /// assert_eq!(b.into_bytes(), SmallVec::from_buf([0b0001_0000]));
    /// ```
    #[must_use]
    pub fn into_bytes(self) -> SmallVec<[u8; SMALLVEC_LEN]> {
        let len = self.len();
        let mut bytes = self.bytes;

        bytes.resize(bytes_for_bit_len(len + 1), 0);

        let mut bitfield: Bitfield<Variable<N>> = Bitfield::from_raw_bytes(bytes, len + 1)
            .unwrap_or_else(|_| {
                unreachable!(
                    "Bitfield with {} bytes must have enough capacity for {} bits.",
                    bytes_for_bit_len(len + 1),
                    len + 1
                )
            });

        bitfield
            .set(len, true)
            .expect("len must be in bounds for bitfield.");

        bitfield.bytes
    }

    /// Instantiates a new instance from `bytes`. Consumes the same format that `self.into_bytes()`
    /// produces (SSZ).
    pub fn from_bytes(bytes: SmallVec<[u8; SMALLVEC_LEN]>) -> Result<Self, BitlistFromBytesError> {
        let bytes_len = bytes.len();
        let mut initial_bitfield: Bitfield<Variable<N>> = {
            let num_bits = bytes.len() * 8;
            Bitfield::from_raw_bytes(bytes, num_bits)?
        };

        let len = initial_bitfield
            .highest_set_bit()
            .ok_or(BitlistFromBytesError::MissingLengthInformation)?;

        // The length bit should be in the last byte, or else it means we have too many bytes.
        if len / 8 + 1 != bytes_len {
            return Err(BitlistFromBytesError::InvalidByteCount {
                given: bytes_len,
                expected: len / 8 + 1,
            });
        }

        if len <= N::USIZE {
            initial_bitfield
                .set(len, false)
                .expect("Bit has been confirmed to exist");

            let mut bytes = initial_bitfield.into_raw_bytes();

            bytes.truncate(bytes_for_bit_len(len));

            Self::from_raw_bytes(bytes, len)
        } else {
            Err(OutOfBounds {
                i: N::USIZE,
                len: N::USIZE,
            }
            .into())
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum BitlistFromBytesError {
    #[error("the length cannot be known as there is not a set bit")]
    MissingLengthInformation,
    #[error("excess bits set to true")]
    ExcessBits,
    #[error("invalid number of bytes ({given}) for a given bit length (expected: {expected})")]
    InvalidByteCount { given: usize, expected: usize },
    #[error(transparent)]
    OutOfBounds(#[from] OutOfBounds),
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("index out of bounds: the length is {len}, but the index is {i}")]
pub struct OutOfBounds {
    pub len: usize,
    pub i: usize,
}

impl<N: Unsigned + Clone> Bitfield<Fixed<N>> {
    /// Instantiate a new `Bitfield` with a fixed-length of `N` bits.
    ///
    /// All bits are initialized to `false`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            bytes: smallvec![0; bytes_for_bit_len(N::USIZE)],
            len: N::USIZE,
            _phantom: PhantomData,
        }
    }

    /// Consumes `self`, returning a serialized representation.
    ///
    /// The output is faithful to the SSZ encoding of `self`.
    ///
    /// ## Example
    /// ```
    /// use ssz::types::{BitVector, typenum};
    /// use smallvec::SmallVec;
    ///
    /// type BitVector4 = BitVector<typenum::U4>;
    ///
    /// assert_eq!(BitVector4::new().into_bytes(), SmallVec::from_buf([0b0000_0000]));
    /// ```
    #[must_use]
    pub fn into_bytes(self) -> SmallVec<[u8; SMALLVEC_LEN]> {
        self.into_raw_bytes()
    }

    /// Instantiates a new instance from `bytes`. Consumes the same format that `self.into_bytes()`
    /// produces (SSZ).
    ///
    /// Returns `None` if `bytes` are not a valid encoding.
    pub fn from_bytes(bytes: SmallVec<[u8; SMALLVEC_LEN]>) -> Result<Self, BitlistFromBytesError> {
        Self::from_raw_bytes(bytes, N::USIZE)
    }
}

impl<N: Unsigned + Clone> Default for Bitfield<Fixed<N>> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: BitfieldBehaviour> Bitfield<T> {
    /// Sets the `i`'th bit to `value`.
    ///
    /// Returns `None` if `i` is out-of-bounds of `self`.
    pub fn set(&mut self, i: usize, value: bool) -> Result<(), Error> {
        let len = self.len;

        if i < len {
            let byte = self
                .bytes
                .get_mut(i / 8)
                .ok_or(Error::OutOfBounds { i, len })?;

            if value {
                *byte |= 1 << (i % 8);
            } else {
                *byte &= !(1 << (i % 8));
            }

            Ok(())
        } else {
            Err(Error::OutOfBounds { i, len: self.len })
        }
    }

    /// Returns the value of the `i`'th bit.
    ///
    /// Returns `Error` if `i` is out-of-bounds of `self`.
    pub fn get(&self, i: usize) -> Result<bool, Error> {
        if i < self.len {
            let byte = self
                .bytes
                .get(i / 8)
                .ok_or(Error::OutOfBounds { i, len: self.len })?;

            Ok(*byte & 1 << (i % 8) > 0)
        } else {
            Err(Error::OutOfBounds { i, len: self.len })
        }
    }

    /// Returns the number of bits stored in `self`.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if `self.len() == 0`.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the underlying bytes representation of the bitfield.
    #[must_use]
    pub fn into_raw_bytes(self) -> SmallVec<[u8; SMALLVEC_LEN]> {
        self.bytes
    }

    /// Returns a view into the underlying bytes representation of the bitfield.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }

    /// Instantiates from the given `bytes`, which are the same format as output from
    /// `self.into_raw_bytes()`.
    ///
    /// Returns `None` if:
    ///
    /// - `bytes` is not the minimal required bytes to represent a bitfield of `bit_len` bits.
    /// - `bit_len` is not a multiple of 8 and `bytes` contains set bits that are higher than, or
    ///   equal to `bit_len`.
    fn from_raw_bytes(
        bytes: SmallVec<[u8; SMALLVEC_LEN]>,
        bit_len: usize,
    ) -> Result<Self, BitlistFromBytesError> {
        if bit_len == 0 {
            if bytes.len() == 1 && bytes[0] == 0 {
                // A bitfield with `bit_len` 0 can only be represented by a single zero byte.
                Ok(Self {
                    bytes,
                    len: 0,
                    _phantom: PhantomData,
                })
            } else {
                Err(BitlistFromBytesError::ExcessBits)
            }
        } else if bytes.len() != bytes_for_bit_len(bit_len) {
            // The number of bytes must be the minimum required to represent `bit_len`.
            Err(BitlistFromBytesError::InvalidByteCount {
                given: bytes.len(),
                expected: bytes_for_bit_len(bit_len),
            })
        } else {
            // Ensure there are no bits higher than `bit_len` that are set to true.
            let (mask, _) = u8::MAX.overflowing_shr(8 - (bit_len as u32 % 8));

            if (bytes.last().expect("Guarded against empty bytes") & !mask) == 0 {
                Ok(Self {
                    bytes,
                    len: bit_len,
                    _phantom: PhantomData,
                })
            } else {
                Err(BitlistFromBytesError::ExcessBits)
            }
        }
    }

    /// Returns the `Some(i)` where `i` is the highest index with a set bit. Returns `None` if
    /// there are no set bits.
    #[must_use]
    pub fn highest_set_bit(&self) -> Option<usize> {
        self.bytes
            .iter()
            .enumerate()
            .rev()
            .find(|(_, byte)| **byte > 0)
            .map(|(i, byte)| i * 8 + 7 - byte.leading_zeros() as usize)
    }

    /// Returns an iterator across bitfield `bool` values, starting at the lowest index.
    #[must_use]
    pub fn iter(&self) -> BitIter<'_, T> {
        BitIter {
            bitfield: self,
            i: 0,
        }
    }

    /// Returns true if no bits are set.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.bytes.iter().all(|byte| *byte == 0)
    }

    /// Returns the number of bits that are set to `true`.
    #[must_use]
    pub fn num_set_bits(&self) -> usize {
        self.bytes
            .iter()
            .map(|byte| byte.count_ones() as usize)
            .sum()
    }
}

impl<'a, T: BitfieldBehaviour> IntoIterator for &'a Bitfield<T> {
    type IntoIter = BitIter<'a, T>;
    type Item = bool;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Returns the minimum required bytes to represent a given number of bits.
///
/// `bit_len == 0` requires a single byte.
const fn bytes_for_bit_len(bit_len: usize) -> usize {
    let v2 = (bit_len + 7) / 8;

    if v2 >= 1 {
        v2
    } else {
        1
    }
}

/// An iterator over the bits in a `Bitfield`.
pub struct BitIter<'a, T> {
    bitfield: &'a Bitfield<T>,
    i: usize,
}

impl<'a, T: BitfieldBehaviour> Iterator for BitIter<'a, T> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.bitfield.get(self.i).ok()?;
        self.i += 1;
        Some(res)
    }
}

impl<N: Unsigned + Clone> Ssz for Bitfield<Variable<N>> {
    const SSZ_FIXED_LEN: Option<NonZeroUsize> = None;

    const TREE_HASH_TYPE: crate::tree_hash::TreeHashType = crate::tree_hash::TreeHashType::List;

    fn tree_hash_root(&self) -> Hash256 {
        // Note: we use `as_slice` because it does _not_ have the length-delimiting bit set (or
        // present).
        let root = bitfield_bytes_tree_hash_root::<N>(self.as_slice());
        crate::tree_hash::mix_in_length(&root, self.len())
    }

    fn ssz_bytes_len(&self) -> NonZeroUsize {
        // We could likely do better than turning this into bytes and reading the length, however
        // it is kept this way for simplicity.
        self.clone()
            .into_bytes()
            .len()
            .try_into()
            .expect("encoded length should be > 0")
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.clone().into_bytes());
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(bytes.to_smallvec())
            .map_err(|e| DecodeError::BytesInvalid(format!("BitList failed to decode: {:?}", e)))
    }
}

impl<N: Unsigned + Clone> Ssz for Bitfield<Fixed<N>> {
    const SSZ_FIXED_LEN: Option<NonZeroUsize> = Some({
        match NonZeroUsize::new(bytes_for_bit_len(N::USIZE)) {
            Some(some) => some,
            None => unreachable!(),
        }
    });

    const TREE_HASH_TYPE: crate::tree_hash::TreeHashType = crate::tree_hash::TreeHashType::Vector;

    fn tree_hash_root(&self) -> Hash256 {
        bitfield_bytes_tree_hash_root::<N>(self.as_slice())
    }

    fn ssz_bytes_len(&self) -> NonZeroUsize {
        self.as_slice()
            .len()
            .try_into()
            .expect("encoded length should be > 0")
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.clone().into_bytes());
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(bytes.to_smallvec())
            .map_err(|e| DecodeError::BytesInvalid(format!("BitVector failed to decode: {:?}", e)))
    }
}

impl<N: Unsigned + Clone> Serialize for Bitfield<Variable<N>> {
    /// Serde serialization is compliant with the Ethereum YAML test format.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&serde_utils::to_hex(self.as_ssz_bytes()))
    }
}

impl<'de, N: Unsigned + Clone> Deserialize<'de> for Bitfield<Variable<N>> {
    /// Serde serialization is compliant with the Ethereum YAML test format.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = serde_utils::hex_string::deserialize::<_, Vec<u8>>(deserializer)?;
        Self::from_ssz_bytes(&bytes)
            .map_err(|e| serde::de::Error::custom(format!("Bitfield {:?}", e)))
    }
}

impl<N: Unsigned + Clone> Serialize for Bitfield<Fixed<N>> {
    /// Serde serialization is compliant with the Ethereum YAML test format.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&serde_utils::to_hex(self.as_ssz_bytes()))
    }
}

impl<'de, N: Unsigned + Clone> Deserialize<'de> for Bitfield<Fixed<N>> {
    /// Serde serialization is compliant with the Ethereum YAML test format.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = serde_utils::hex_string::deserialize::<_, Vec<u8>>(deserializer)?;
        Self::from_ssz_bytes(&bytes)
            .map_err(|e| serde::de::Error::custom(format!("Bitfield {:?}", e)))
    }
}

#[cfg(test)]
mod bitvector {
    use super::*;

    pub type BitVector0 = BitVector<typenum::U0>;
    pub type BitVector1 = BitVector<typenum::U1>;
    pub type BitVector4 = BitVector<typenum::U4>;
    pub type BitVector8 = BitVector<typenum::U8>;
    pub type BitVector16 = BitVector<typenum::U16>;
    pub type BitVector64 = BitVector<typenum::U64>;

    #[test]
    fn ssz_encode() {
        assert_eq!(BitVector0::new().as_ssz_bytes(), vec![0b0000_0000]);
        assert_eq!(BitVector1::new().as_ssz_bytes(), vec![0b0000_0000]);
        assert_eq!(BitVector4::new().as_ssz_bytes(), vec![0b0000_0000]);
        assert_eq!(BitVector8::new().as_ssz_bytes(), vec![0b0000_0000]);
        assert_eq!(
            BitVector16::new().as_ssz_bytes(),
            vec![0b0000_0000, 0b0000_0000]
        );

        let mut b = BitVector8::new();
        for i in 0..8 {
            b.set(i, true).unwrap();
        }
        assert_eq!(b.as_ssz_bytes(), vec![255]);

        let mut b = BitVector4::new();
        for i in 0..4 {
            b.set(i, true).unwrap();
        }
        assert_eq!(b.as_ssz_bytes(), vec![0b0000_1111]);
    }

    #[test]
    fn ssz_decode() {
        assert!(BitVector0::from_ssz_bytes(&[0b0000_0000]).is_ok());
        assert!(BitVector0::from_ssz_bytes(&[0b0000_0001]).is_err());
        assert!(BitVector0::from_ssz_bytes(&[0b0000_0010]).is_err());

        assert!(BitVector1::from_ssz_bytes(&[0b0000_0001]).is_ok());
        assert!(BitVector1::from_ssz_bytes(&[0b0000_0010]).is_err());
        assert!(BitVector1::from_ssz_bytes(&[0b0000_0100]).is_err());
        assert!(BitVector1::from_ssz_bytes(&[0b0000_0000, 0b0000_0000]).is_err());

        assert!(BitVector8::from_ssz_bytes(&[0b0000_0000]).is_ok());
        assert!(BitVector8::from_ssz_bytes(&[1, 0b0000_0000]).is_err());
        assert!(BitVector8::from_ssz_bytes(&[0b0000_0000, 1]).is_err());
        assert!(BitVector8::from_ssz_bytes(&[0b0000_0001]).is_ok());
        assert!(BitVector8::from_ssz_bytes(&[0b0000_0010]).is_ok());
        assert!(BitVector8::from_ssz_bytes(&[0b0000_0100, 0b0000_0001]).is_err());
        assert!(BitVector8::from_ssz_bytes(&[0b0000_0100, 0b0000_0010]).is_err());
        assert!(BitVector8::from_ssz_bytes(&[0b0000_0100, 0b0000_0100]).is_err());

        assert!(BitVector16::from_ssz_bytes(&[0b0000_0000]).is_err());
        assert!(BitVector16::from_ssz_bytes(&[0b0000_0000, 0b0000_0000]).is_ok());
        assert!(BitVector16::from_ssz_bytes(&[1, 0b0000_0000, 0b0000_0000]).is_err());
    }

    #[test]
    fn ssz_round_trip() {
        assert_round_trip(BitVector0::new());

        let mut b = BitVector1::new();
        b.set(0, true).unwrap();
        assert_round_trip(b);

        let mut b = BitVector8::new();
        for j in 0..8 {
            if j % 2 == 0 {
                b.set(j, true).unwrap();
            }
        }
        assert_round_trip(b);

        let mut b = BitVector8::new();
        for j in 0..8 {
            b.set(j, true).unwrap();
        }
        assert_round_trip(b);

        let mut b = BitVector16::new();
        for j in 0..16 {
            if j % 2 == 0 {
                b.set(j, true).unwrap();
            }
        }
        assert_round_trip(b);

        let mut b = BitVector16::new();
        for j in 0..16 {
            b.set(j, true).unwrap();
        }
        assert_round_trip(b);
    }

    fn assert_round_trip<T: Ssz + PartialEq + std::fmt::Debug>(t: T) {
        assert_eq!(T::from_ssz_bytes(&t.as_ssz_bytes()).unwrap(), t);
    }

    #[test]
    fn ssz_bytes_len() {
        for i in 0..64 {
            let mut bitfield = BitVector64::new();
            for j in 0..i {
                bitfield.set(j, true).expect("should set bit in bounds");
            }
            let bytes = bitfield.as_ssz_bytes();
            assert_eq!(bitfield.ssz_bytes_len().get(), bytes.len(), "i = {}", i);
        }
    }

    #[test]
    fn excess_bits_nimbus() {
        let bad = vec![0b0001_1111];

        assert!(BitVector4::from_ssz_bytes(&bad).is_err());
    }

    // Ensure that stack size of a BitVector is manageable.
    #[test]
    #[ignore = "this keeps failing and i'm not sure why"]
    fn size_of() {
        assert_eq!(std::mem::size_of::<BitVector64>(), SMALLVEC_LEN + 16);
    }
}

#[cfg(test)]
#[allow(clippy::cognitive_complexity)]
mod bitlist {
    use super::*;

    pub type BitList0 = BitList<typenum::U0>;
    pub type BitList1 = BitList<typenum::U1>;
    pub type BitList8 = BitList<typenum::U8>;
    pub type BitList16 = BitList<typenum::U16>;
    pub type BitList1024 = BitList<typenum::U1024>;

    #[test]
    fn ssz_encode() {
        assert_eq!(
            BitList0::with_capacity(0).unwrap().as_ssz_bytes(),
            vec![0b0000_0001],
        );

        assert_eq!(
            BitList1::with_capacity(0).unwrap().as_ssz_bytes(),
            vec![0b0000_0001],
        );

        assert_eq!(
            BitList1::with_capacity(1).unwrap().as_ssz_bytes(),
            vec![0b0000_0010],
        );

        assert_eq!(
            BitList8::with_capacity(8).unwrap().as_ssz_bytes(),
            vec![0b0000_0000, 0b0000_0001],
        );

        assert_eq!(
            BitList8::with_capacity(7).unwrap().as_ssz_bytes(),
            vec![0b1000_0000]
        );

        let mut b = BitList8::with_capacity(8).unwrap();
        for i in 0..8 {
            b.set(i, true).unwrap();
        }
        assert_eq!(b.as_ssz_bytes(), vec![255, 0b0000_0001]);

        let mut b = BitList8::with_capacity(8).unwrap();
        for i in 0..4 {
            b.set(i, true).unwrap();
        }
        assert_eq!(b.as_ssz_bytes(), vec![0b0000_1111, 0b0000_0001]);

        assert_eq!(
            BitList16::with_capacity(16).unwrap().as_ssz_bytes(),
            vec![0b0000_0000, 0b0000_0000, 0b0000_0001]
        );
    }

    #[test]
    fn ssz_decode() {
        assert!(BitList0::from_ssz_bytes(&[]).is_err());
        assert!(BitList1::from_ssz_bytes(&[]).is_err());
        assert!(BitList8::from_ssz_bytes(&[]).is_err());
        assert!(BitList16::from_ssz_bytes(&[]).is_err());

        assert!(BitList0::from_ssz_bytes(&[0b0000_0000]).is_err());
        assert!(BitList1::from_ssz_bytes(&[0b0000_0000, 0b0000_0000]).is_err());
        assert!(BitList8::from_ssz_bytes(&[0b0000_0000]).is_err());
        assert!(BitList16::from_ssz_bytes(&[0b0000_0000]).is_err());

        assert!(BitList0::from_ssz_bytes(&[0b0000_0001]).is_ok());
        assert!(BitList0::from_ssz_bytes(&[0b0000_0010]).is_err());

        assert!(BitList1::from_ssz_bytes(&[0b0000_0001]).is_ok());
        assert!(BitList1::from_ssz_bytes(&[0b0000_0010]).is_ok());
        assert!(BitList1::from_ssz_bytes(&[0b0000_0100]).is_err());

        assert!(BitList8::from_ssz_bytes(&[0b0000_0001]).is_ok());
        assert!(BitList8::from_ssz_bytes(&[0b0000_0010]).is_ok());
        assert!(BitList8::from_ssz_bytes(&[0b0000_0001, 0b0000_0001]).is_ok());
        assert!(BitList8::from_ssz_bytes(&[0b0000_0001, 0b0000_0010]).is_err());
        assert!(BitList8::from_ssz_bytes(&[0b0000_0001, 0b0000_0100]).is_err());
    }

    #[test]
    fn ssz_decode_extra_bytes() {
        assert!(BitList0::from_ssz_bytes(&[0b0000_0001, 0b0000_0000]).is_err());
        assert!(BitList1::from_ssz_bytes(&[0b0000_0001, 0b0000_0000]).is_err());
        assert!(BitList8::from_ssz_bytes(&[0b0000_0001, 0b0000_0000]).is_err());
        assert!(BitList16::from_ssz_bytes(&[0b0000_0001, 0b0000_0000]).is_err());
        assert!(BitList1024::from_ssz_bytes(&[0b1000_0000, 0]).is_err());
        assert!(BitList1024::from_ssz_bytes(&[0b1000_0000, 0, 0]).is_err());
        assert!(BitList1024::from_ssz_bytes(&[0b1000_0000, 0, 0, 0, 0]).is_err());
    }

    #[test]
    fn ssz_round_trip() {
        assert_round_trip(BitList0::with_capacity(0).unwrap());

        for i in 0..2 {
            assert_round_trip(BitList1::with_capacity(i).unwrap());
        }
        for i in 0..9 {
            assert_round_trip(BitList8::with_capacity(i).unwrap());
        }
        for i in 0..17 {
            assert_round_trip(BitList16::with_capacity(i).unwrap());
        }

        let mut b = BitList1::with_capacity(1).unwrap();
        b.set(0, true).unwrap();
        assert_round_trip(b);

        for i in 0..8 {
            let mut b = BitList8::with_capacity(i).unwrap();
            for j in 0..i {
                if j % 2 == 0 {
                    b.set(j, true).unwrap();
                }
            }
            assert_round_trip(b);

            let mut b = BitList8::with_capacity(i).unwrap();
            for j in 0..i {
                b.set(j, true).unwrap();
            }
            assert_round_trip(b);
        }

        for i in 0..16 {
            let mut b = BitList16::with_capacity(i).unwrap();
            for j in 0..i {
                if j % 2 == 0 {
                    b.set(j, true).unwrap();
                }
            }
            assert_round_trip(b);

            let mut b = BitList16::with_capacity(i).unwrap();
            for j in 0..i {
                b.set(j, true).unwrap();
            }
            assert_round_trip(b);
        }
    }

    fn assert_round_trip<T: Ssz + PartialEq + std::fmt::Debug>(t: T) {
        assert_eq!(T::from_ssz_bytes(&t.as_ssz_bytes()).unwrap(), t);
    }

    #[test]
    fn from_raw_bytes() {
        assert!(BitList1024::from_raw_bytes(smallvec![0b0000_0000], 0).is_ok());
        assert!(BitList1024::from_raw_bytes(smallvec![0b0000_0001], 1).is_ok());
        assert!(BitList1024::from_raw_bytes(smallvec![0b0000_0011], 2).is_ok());
        assert!(BitList1024::from_raw_bytes(smallvec![0b0000_0111], 3).is_ok());
        assert!(BitList1024::from_raw_bytes(smallvec![0b0000_1111], 4).is_ok());
        assert!(BitList1024::from_raw_bytes(smallvec![0b0001_1111], 5).is_ok());
        assert!(BitList1024::from_raw_bytes(smallvec![0b0011_1111], 6).is_ok());
        assert!(BitList1024::from_raw_bytes(smallvec![0b0111_1111], 7).is_ok());
        assert!(BitList1024::from_raw_bytes(smallvec![0b1111_1111], 8).is_ok());

        assert!(BitList1024::from_raw_bytes(smallvec![0b1111_1111, 0b0000_0001], 9).is_ok());
        assert!(BitList1024::from_raw_bytes(smallvec![0b1111_1111, 0b0000_0011], 10).is_ok());
        assert!(BitList1024::from_raw_bytes(smallvec![0b1111_1111, 0b0000_0111], 11).is_ok());
        assert!(BitList1024::from_raw_bytes(smallvec![0b1111_1111, 0b0000_1111], 12).is_ok());
        assert!(BitList1024::from_raw_bytes(smallvec![0b1111_1111, 0b0001_1111], 13).is_ok());
        assert!(BitList1024::from_raw_bytes(smallvec![0b1111_1111, 0b0011_1111], 14).is_ok());
        assert!(BitList1024::from_raw_bytes(smallvec![0b1111_1111, 0b0111_1111], 15).is_ok());
        assert!(BitList1024::from_raw_bytes(smallvec![0b1111_1111, 0b1111_1111], 16).is_ok());

        for i in 0..8 {
            assert!(BitList1024::from_raw_bytes(smallvec![], i).is_err());
            assert!(BitList1024::from_raw_bytes(smallvec![0b1111_1111], i).is_err());
            assert!(BitList1024::from_raw_bytes(smallvec![0b0000_0000, 0b1111_1110], i).is_err());
        }

        assert!(BitList1024::from_raw_bytes(smallvec![0b0000_0001], 0).is_err());

        assert!(BitList1024::from_raw_bytes(smallvec![0b0000_0001], 0).is_err());
        assert!(BitList1024::from_raw_bytes(smallvec![0b0000_0011], 1).is_err());
        assert!(BitList1024::from_raw_bytes(smallvec![0b0000_0111], 2).is_err());
        assert!(BitList1024::from_raw_bytes(smallvec![0b0000_1111], 3).is_err());
        assert!(BitList1024::from_raw_bytes(smallvec![0b0001_1111], 4).is_err());
        assert!(BitList1024::from_raw_bytes(smallvec![0b0011_1111], 5).is_err());
        assert!(BitList1024::from_raw_bytes(smallvec![0b0111_1111], 6).is_err());
        assert!(BitList1024::from_raw_bytes(smallvec![0b1111_1111], 7).is_err());

        assert!(BitList1024::from_raw_bytes(smallvec![0b1111_1111, 0b0000_0001], 8).is_err());
        assert!(BitList1024::from_raw_bytes(smallvec![0b1111_1111, 0b0000_0011], 9).is_err());
        assert!(BitList1024::from_raw_bytes(smallvec![0b1111_1111, 0b0000_0111], 10).is_err());
        assert!(BitList1024::from_raw_bytes(smallvec![0b1111_1111, 0b0000_1111], 11).is_err());
        assert!(BitList1024::from_raw_bytes(smallvec![0b1111_1111, 0b0001_1111], 12).is_err());
        assert!(BitList1024::from_raw_bytes(smallvec![0b1111_1111, 0b0011_1111], 13).is_err());
        assert!(BitList1024::from_raw_bytes(smallvec![0b1111_1111, 0b0111_1111], 14).is_err());
        assert!(BitList1024::from_raw_bytes(smallvec![0b1111_1111, 0b1111_1111], 15).is_err());
    }

    fn test_set_unset(num_bits: usize) {
        let mut bitfield = BitList1024::with_capacity(num_bits).unwrap();

        for i in 0..=num_bits {
            if i < num_bits {
                // Starts as false
                assert_eq!(bitfield.get(i), Ok(false));
                // Can be set true.
                assert!(bitfield.set(i, true).is_ok());
                assert_eq!(bitfield.get(i), Ok(true));
                // Can be set false
                assert!(bitfield.set(i, false).is_ok());
                assert_eq!(bitfield.get(i), Ok(false));
            } else {
                assert!(bitfield.get(i).is_err());
                assert!(bitfield.set(i, true).is_err());
                assert!(bitfield.get(i).is_err());
            }
        }
    }

    fn test_bytes_round_trip(num_bits: usize) {
        for i in 0..num_bits {
            let mut bitfield = BitList1024::with_capacity(num_bits).unwrap();
            bitfield.set(i, true).unwrap();

            let bytes = bitfield.clone().into_raw_bytes();
            assert_eq!(bitfield, Bitfield::from_raw_bytes(bytes, num_bits).unwrap());
        }
    }

    #[test]
    fn set_unset() {
        for i in 0..8 * 5 {
            test_set_unset(i);
        }
    }

    #[test]
    fn bytes_round_trip() {
        for i in 0..8 * 5 {
            test_bytes_round_trip(i);
        }
    }

    /// Type-specialized `smallvec` macro for testing.
    macro_rules! bytevec {
        ($($x : expr),* $(,)*) => {
            {
                let __smallvec: SmallVec<[u8; SMALLVEC_LEN]> = smallvec!($($x),*);
                __smallvec
            }
        };
    }

    #[test]
    fn into_raw_bytes() {
        let mut bitfield = BitList1024::with_capacity(9).unwrap();
        bitfield.set(0, true).unwrap();
        assert_eq!(
            bitfield.clone().into_raw_bytes(),
            bytevec![0b0000_0001, 0b0000_0000]
        );
        bitfield.set(1, true).unwrap();
        assert_eq!(
            bitfield.clone().into_raw_bytes(),
            bytevec![0b0000_0011, 0b0000_0000]
        );
        bitfield.set(2, true).unwrap();
        assert_eq!(
            bitfield.clone().into_raw_bytes(),
            bytevec![0b0000_0111, 0b0000_0000]
        );
        bitfield.set(3, true).unwrap();
        assert_eq!(
            bitfield.clone().into_raw_bytes(),
            bytevec![0b0000_1111, 0b0000_0000]
        );
        bitfield.set(4, true).unwrap();
        assert_eq!(
            bitfield.clone().into_raw_bytes(),
            bytevec![0b0001_1111, 0b0000_0000]
        );
        bitfield.set(5, true).unwrap();
        assert_eq!(
            bitfield.clone().into_raw_bytes(),
            bytevec![0b0011_1111, 0b0000_0000]
        );
        bitfield.set(6, true).unwrap();
        assert_eq!(
            bitfield.clone().into_raw_bytes(),
            bytevec![0b0111_1111, 0b0000_0000]
        );
        bitfield.set(7, true).unwrap();
        assert_eq!(
            bitfield.clone().into_raw_bytes(),
            bytevec![0b1111_1111, 0b0000_0000]
        );
        bitfield.set(8, true).unwrap();
        assert_eq!(
            bitfield.into_raw_bytes(),
            bytevec![0b1111_1111, 0b0000_0001]
        );
    }

    #[test]
    fn highest_set_bit() {
        assert_eq!(
            BitList1024::with_capacity(16).unwrap().highest_set_bit(),
            None
        );

        assert_eq!(
            BitList1024::from_raw_bytes(smallvec![0b0000_0001, 0b0000_0000], 16)
                .unwrap()
                .highest_set_bit(),
            Some(0)
        );

        assert_eq!(
            BitList1024::from_raw_bytes(smallvec![0b0000_0010, 0b0000_0000], 16)
                .unwrap()
                .highest_set_bit(),
            Some(1)
        );

        assert_eq!(
            BitList1024::from_raw_bytes(smallvec![0b0000_1000], 8)
                .unwrap()
                .highest_set_bit(),
            Some(3)
        );

        assert_eq!(
            BitList1024::from_raw_bytes(smallvec![0b0000_0000, 0b1000_0000], 16)
                .unwrap()
                .highest_set_bit(),
            Some(15)
        );
    }

    #[test]
    fn num_set_bits() {
        let a = BitList1024::from_raw_bytes(smallvec![0b1100, 0b0001], 16).unwrap();
        let b = BitList1024::from_raw_bytes(smallvec![0b1011, 0b1001], 16).unwrap();

        assert_eq!(a.num_set_bits(), 3);
        assert_eq!(b.num_set_bits(), 5);
    }

    #[test]
    fn iter() {
        let mut bitfield = BitList1024::with_capacity(9).unwrap();
        bitfield.set(2, true).unwrap();
        bitfield.set(8, true).unwrap();

        assert_eq!(
            bitfield.iter().collect::<Vec<bool>>(),
            vec![false, false, true, false, false, false, false, false, true]
        );
    }

    #[test]
    fn ssz_bytes_len() {
        for i in 1..64 {
            let mut bitfield = BitList1024::with_capacity(i).unwrap();
            for j in 0..i {
                bitfield.set(j, true).expect("should set bit in bounds");
            }
            let bytes = bitfield.as_ssz_bytes();
            assert_eq!(bitfield.ssz_bytes_len().get(), bytes.len(), "i = {}", i);
        }
    }

    // Ensure that the stack size of a BitList is manageable.
    #[test]
    #[ignore = "this keeps failing and i'm not sure why"]
    fn size_of() {
        assert_eq!(std::mem::size_of::<BitList1024>(), SMALLVEC_LEN + 16);
    }
}
