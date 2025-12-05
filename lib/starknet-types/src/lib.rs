// TODO: Check for size (must be < modulus)

use std::fmt;

use ruint::{ParseError, aliases::U256};

#[derive(Clone, Copy, PartialEq)]
pub struct Felt(ruint::aliases::U256);

impl fmt::Debug for Felt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Felt({self})")
    }
}

impl fmt::Display for Felt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl From<u64> for Felt {
    fn from(value: u64) -> Self {
        Self(U256::from(value))
    }
}

impl From<u128> for Felt {
    fn from(value: u128) -> Self {
        Self(U256::from(value))
    }
}

impl From<starknet_crypto::Felt> for Felt {
    fn from(value: starknet_crypto::Felt) -> Self {
        Self::from_be_bytes(value.to_bytes_be())
    }
}

impl From<Felt> for starknet_crypto::Felt {
    fn from(value: Felt) -> Self {
        Self::from_bytes_be(&value.to_be_bytes())
    }
}

impl Felt {
    pub const ZERO: Self = Self::from_be_bytes([0; 32]);

    pub const fn from_be_bytes(bytes: [u8; 32]) -> Self {
        Self(U256::from_be_bytes(bytes))
    }

    pub const fn to_be_bytes(&self) -> [u8; 32] {
        self.0.to_be_bytes()
    }

    pub const fn from_le_bytes(bytes: [u8; 32]) -> Self {
        Self(U256::from_le_bytes(bytes))
    }

    pub const fn to_le_bytes(&self) -> [u8; 32] {
        self.0.to_le_bytes()
    }

    pub fn from_hex(s: &str) -> Result<Self, ParseError> {
        U256::from_str_radix(s.strip_prefix("0x").unwrap_or(s), 16).map(Felt)
    }
}

/// A node in the Merkle-Patricia tree, can be a leaf, binary node, or an edge node.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(untagged)
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum MerkleNode {
    /// Binary/branch node.
    ///
    /// An internal node whose both children are non-zero.
    BinaryNode {
        /// The hash of the left child
        left: Felt,
        /// The hash of the right child
        right: Felt,
    },
    /// Edge/leaf node.
    ///
    /// Represents a path to the highest non-zero descendant node.
    EdgeNode {
        /// An unsigned integer whose binary representation represents the path from the current node to
        /// its highest non-zero descendant (bounded by 2^251)
        path: Felt,
        /// The length of the path (bounded by 251)
        length: u8,
        /// The hash of the unique non-zero maximal-height descendant node
        child: Felt,
    },
}

#[cfg(feature = "serde")]
impl serde::Serialize for Felt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            serializer.serialize_bytes(&self.0.to_be_bytes::<32>())
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Felt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            String::deserialize(deserializer).and_then(|s| {
                U256::from_str_radix(s.strip_prefix("0x").unwrap_or(&s), 16)
                    .map(Felt)
                    .map_err(serde::de::Error::custom)
            })
        } else {
            <[u8; 32]>::deserialize(deserializer).map(Felt::from_be_bytes)
        }
    }
}

#[cfg(feature = "bincode")]
impl<Context> bincode::Decode<Context> for Felt {
    fn decode<D: bincode::de::Decoder<Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        use bincode::{
            config::{Config, Endianness, IntEncoding},
            de::read::Reader,
        };

        decoder.claim_bytes_read(32)?;

        match decoder.config().int_encoding() {
            IntEncoding::Variable => Err(bincode::error::DecodeError::Other(
                "varint encoding is not supported for Felt",
            )),
            IntEncoding::Fixed => {
                let mut bytes = [0u8; 32];
                decoder.reader().read(&mut bytes)?;
                match decoder.config().endianness() {
                    Endianness::Little => Ok(Self::from_le_bytes(bytes)),
                    Endianness::Big => Ok(Self::from_be_bytes(bytes)),
                    _ => Err(bincode::error::DecodeError::Other("unknown endianness")),
                }
            }
            _ => Err(bincode::error::DecodeError::Other("unknown int encoding")),
        }
    }
}
#[cfg(feature = "bincode")]
bincode::impl_borrow_decode!(Felt);

#[cfg(feature = "bincode")]
impl bincode::Encode for Felt {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        use bincode::{
            config::{Config, Endianness, IntEncoding},
            enc::write::Writer,
        };

        match encoder.config().int_encoding() {
            IntEncoding::Variable => Err(bincode::error::EncodeError::Other(
                "varint encoding is not supported for Felt",
            )),
            IntEncoding::Fixed => match encoder.config().endianness() {
                Endianness::Big => encoder.writer().write(&self.to_be_bytes()),
                Endianness::Little => encoder.writer().write(&self.to_le_bytes()),
                _ => Err(bincode::error::EncodeError::Other("unknown endianness")),
            },
            _ => Err(bincode::error::EncodeError::Other("unknown int encoding")),
        }
    }
}

pub fn commitment_key(mut key: [u8; 32]) -> Felt {
    // wipe the top 6 bits
    key[0] ^= 0b0000_0011;
    Felt::from_be_bytes(key)
}
