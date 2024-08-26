use core::fmt;

use hex::FromHex;
use serde::{de, ser, Deserialize, Serialize};

use crate::errors::{ExpectedLength, InvalidLength};

impl AsRef<[u8; HashValue::LENGTH]> for HashValue {
    fn as_ref(&self) -> &[u8; HashValue::LENGTH] {
        &self.0
    }
}

impl std::ops::Deref for HashValue {
    type Target = [u8; Self::LENGTH];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::Index<usize> for HashValue {
    type Output = u8;

    fn index(&self, s: usize) -> &u8 {
        self.0.index(s)
    }
}

impl fmt::Display for HashValueParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unable to parse HashValue")
    }
}

impl fmt::LowerHex for HashValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

/// Will print shortened (4 bytes) hash
impl fmt::Display for HashValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for byte in self.0.iter().take(4) {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl fmt::Debug for HashValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HashValue(")?;
        <Self as fmt::LowerHex>::fmt(self, f)?;
        write!(f, ")")?;
        Ok(())
    }
}

impl std::error::Error for HashValueParseError {}

/// Parse error when attempting to construct a HashValue
#[derive(Clone, Copy, Debug)]
pub struct HashValueParseError;

#[derive(Eq, PartialEq, Clone, Copy)]
pub struct HashValue(pub [u8; HashValue::LENGTH]);

impl HashValue {
    /// The length of the hash in bytes.
    pub const LENGTH: usize = 32;
    /// The length of the hash in bits.
    pub const LENGTH_IN_BITS: usize = Self::LENGTH * 8;

    /// Create a new [`HashValue`] from a byte array.
    pub fn new(hash: [u8; HashValue::LENGTH]) -> Self {
        HashValue(hash)
    }

    /// Parse a given hex string to a hash value.
    pub fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, HashValueParseError> {
        <[u8; Self::LENGTH]>::from_hex(hex)
            .map_err(|_| HashValueParseError)
            .map(Self::new)
    }

    /// Full hex representation of a given hash value.
    pub fn to_hex(&self) -> String {
        format!("{:x}", self)
    }

    /// Creates a zero-initialized instance.
    pub const fn zero() -> Self {
        HashValue([0; HashValue::LENGTH])
    }

    pub fn iter_bits(&self) -> HashValueBitIterator<'_> {
        HashValueBitIterator::new(self)
    }

    /// Returns the length of common prefix of `self` and `other` in bits.
    pub fn common_prefix_bits_len(&self, other: HashValue) -> usize {
        self.iter_bits()
            .zip(other.iter_bits())
            .take_while(|(x, y)| x == y)
            .count()
    }
}

impl From<HashValue> for Vec<u8> {
    fn from(value: HashValue) -> Self {
        value.0.as_slice().to_vec()
    }
}

impl Default for HashValue {
    fn default() -> Self {
        HashValue::new([0; HashValue::LENGTH])
    }
}

impl ser::Serialize for HashValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_hex())
        } else {
            // In order to preserve the Serde data model and help analysis tools,
            // make sure to wrap our value in a container with the same name
            // as the original type.
            #[derive(Serialize)]
            #[serde(rename = "HashValue")]
            struct Value<'a> {
                hash: &'a [u8; HashValue::LENGTH],
            }
            Value { hash: &self.0 }.serialize(serializer)
        }
    }
}

impl<'de> de::Deserialize<'de> for HashValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let encoded_hash = <String>::deserialize(deserializer)?;
            HashValue::from_hex(encoded_hash.as_str())
                .map_err(<D::Error as ::serde::de::Error>::custom)
        } else {
            // See comment in serialize.
            #[derive(Deserialize)]
            #[serde(rename = "HashValue")]
            struct Value {
                hash: [u8; HashValue::LENGTH],
            }

            let value = Value::deserialize(deserializer)
                .map_err(<D::Error as ::serde::de::Error>::custom)?;
            Ok(Self::new(value.hash))
        }
    }
}

/// An iterator over `HashValue` that generates one bit for each iteration.
pub struct HashValueBitIterator<'a> {
    /// The reference to the bytes that represent the `HashValue`.
    hash_bytes: &'a [u8],
    pos: std::ops::Range<usize>,
    // invariant hash_bytes.len() == HashValue::LENGTH;
    // invariant pos.end == hash_bytes.len() * 8;
}

impl<'a> HashValueBitIterator<'a> {
    /// Constructs a new `HashValueBitIterator` using given `HashValue`.
    fn new(hash_value: &'a HashValue) -> Self {
        HashValueBitIterator {
            hash_bytes: hash_value.as_ref(),
            pos: (0..HashValue::LENGTH_IN_BITS),
        }
    }

    /// Returns the `index`-th bit in the bytes.
    fn get_bit(&self, index: usize) -> bool {
        // debug_assert_eq!(self.hash_bytes.len(), HashValue::LENGTH); // invariant
        // debug_assert_lt!(index, HashValue::LENGTH_IN_BITS); // assumed precondition
        let pos = index / 8;
        let bit = 7 - index % 8;
        (self.hash_bytes[pos] >> bit) & 1 != 0
    }
}

impl<'a> std::iter::Iterator for HashValueBitIterator<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos.next().map(|x| self.get_bit(x))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.pos.size_hint()
    }
}

impl<'a> std::iter::DoubleEndedIterator for HashValueBitIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.pos.next_back().map(|x| self.get_bit(x))
    }
}

impl TryFrom<Vec<u8>> for HashValue {
    type Error = InvalidLength;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(HashValue::new(value.as_slice().try_into().map_err(
            |_| InvalidLength {
                expected: ExpectedLength::Exact(HashValue::LENGTH),
                found: value.len(),
            },
        )?))
    }
}
