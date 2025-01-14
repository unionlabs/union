#![allow(clippy::disallowed_types)] // need to access the inner type to wrap it

use core::{
    fmt::{self, Display},
    iter::Sum,
    ops::{Add, AddAssign, Div, Rem},
    str::FromStr,
};

/// [`primitive_types::U256`] can't roundtrip through string conversion since it parses from hex but displays as decimal.
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(transparent)]
pub struct U256(pub primitive_types::U256);

impl fmt::Debug for U256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "U256({self})")
    }
}

impl Display for U256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for U256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for U256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)
            .and_then(|s| {
                primitive_types::U256::from_dec_str(&s).map_err(|err| {
                    serde::de::Error::custom(format!("failure to parse string data: {err}"))
                })
            })
            .map(Self)
    }
}

#[cfg(feature = "serde")]
#[allow(clippy::missing_errors_doc)]
pub mod u256_big_endian_hex {
    use serde::de::{self, Deserialize};

    use crate::U256;

    pub fn serialize<S>(data: &U256, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&data.to_be_hex_packed())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<U256, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).and_then(|s| -> Result<U256, D::Error> {
            U256::from_be_hex(s).map_err(de::Error::custom)
        })
    }
}

#[cfg(feature = "bincode")]
impl bincode::Decode for U256 {
    fn decode<D: bincode::de::Decoder>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        use bincode::{
            config::{Config, Endianness, IntEncoding},
            de::read::Reader,
        };

        decoder.claim_bytes_read(32)?;

        match decoder.config().int_encoding() {
            IntEncoding::Variable => Err(bincode::error::DecodeError::Other(
                "varint encoding is not supported for U256",
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
bincode::impl_borrow_decode!(U256);

#[cfg(feature = "bincode")]
impl bincode::Encode for U256 {
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
                "varint encoding is not supported for U256",
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

impl fmt::LowerHex for U256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.write_str(&self.to_be_hex())
        } else {
            f.write_str(&self.to_be_hex()[2..])
        }
    }
}

impl U256 {
    pub const MAX: Self = Self::from_limbs([u64::MAX; 4]);
    pub const ZERO: Self = Self::from_limbs([0; 4]);
    pub const ONE: Self = Self::from_limbs([0, 0, 0, 1]);

    // one day...
    // pub const fn from_const_str<const STR: &'static str>() -> Self {}
}

impl From<u32> for U256 {
    fn from(value: u32) -> Self {
        Self(primitive_types::U256::from(value))
    }
}

impl From<u64> for U256 {
    fn from(value: u64) -> Self {
        Self(primitive_types::U256::from(value))
    }
}

impl TryFrom<U256> for u64 {
    type Error = ();

    fn try_from(value: U256) -> Result<Self, Self::Error> {
        if value > U256::from(u64::MAX) {
            Err(())
        } else {
            Ok(value.0.as_u64())
        }
    }
}

impl From<primitive_types::U256> for U256 {
    fn from(value: primitive_types::U256) -> Self {
        Self(value)
    }
}

impl From<U256> for primitive_types::U256 {
    fn from(value: U256) -> Self {
        value.0
    }
}

impl U256 {
    #[must_use]
    pub fn leading_zeros(&self) -> u32 {
        self.0.leading_zeros()
    }

    #[must_use]
    pub fn to_le_bytes(&self) -> [u8; 32] {
        let mut buf = [0; 32];
        self.0.to_little_endian(&mut buf);
        buf
    }

    #[must_use]
    pub fn to_be_bytes(&self) -> [u8; 32] {
        let mut buf = [0; 32];
        self.0.to_big_endian(&mut buf);
        buf
    }

    #[must_use]
    pub fn to_be_bytes_packed(&self) -> Vec<u8> {
        let buffer = self.to_be_bytes();
        let leading_empty_bytes = (self.leading_zeros() / 8) as usize;
        buffer[leading_empty_bytes..].to_vec()
    }

    /// Attempt to convert the provided big-endian bytes into a [`U256`].
    ///
    /// # Errors
    ///
    /// See [`TryFromBytesError`] for the possible failure modes for this function.
    pub fn try_from_be_bytes(bz: &[u8]) -> Result<Self, TryFromBytesError> {
        let len = bz.len();

        if (0..=32).contains(&len) {
            Ok(Self(primitive_types::U256::from_big_endian(bz)))
        } else {
            Err(TryFromBytesError {
                expected_max_len: 32,
                found_len: len,
            })
        }
    }

    #[must_use]
    pub fn from_be_bytes(bz: [u8; 32]) -> Self {
        Self(primitive_types::U256::from_big_endian(&bz))
    }

    #[must_use]
    pub fn from_le_bytes(bz: [u8; 32]) -> Self {
        Self(primitive_types::U256::from_little_endian(&bz))
    }

    #[must_use]
    pub const fn from_limbs(limbs: [u64; 4]) -> Self {
        Self(primitive_types::U256(limbs))
    }

    #[must_use]
    pub const fn as_limbs(&self) -> [u64; 4] {
        self.0 .0
    }

    #[must_use]
    pub fn to_be_hex(&self) -> String {
        let encoded = if self == &Self::ZERO {
            "0".to_string()
        } else {
            hex::encode(self.to_be_bytes())
        };

        format!("0x{encoded}")
    }

    #[must_use]
    pub fn to_be_hex_packed(&self) -> String {
        if self.0.is_zero() {
            format!("0x{}", 0)
        } else {
            // NOTE: Can't use serde_utils::to_hex here as it ensures there's an even number of bytes (by prepending a 0) which we don't want for u256
            format!(
                "0x{}",
                hex::encode(self.to_be_bytes_packed()).trim_start_matches('0')
            )
        }
    }

    /// Attempt to convert the provided big-endian hex string into a [`U256`].
    ///
    /// # Errors
    ///
    /// See [`TryFromHexError`] for the possible failure modes for this function.
    pub fn from_be_hex(s: impl AsRef<str>) -> Result<U256, TryFromHexError> {
        if s.as_ref().is_empty() {
            return Err(TryFromHexError::EmptyString);
        }

        s.as_ref()
            .strip_prefix("0x")
            .ok_or(TryFromHexError::MissingPrefix)
            .and_then(|maybe_hex| {
                Ok(U256::try_from_be_bytes(
                    &if maybe_hex.len() % 2 == 1 {
                        hex::decode(format!("0{maybe_hex}"))
                    } else {
                        hex::decode(maybe_hex)
                    }
                    .map_err(TryFromHexError::Hex)?,
                )?)
            })
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("invalid bytes length, expected between 0..={expected_max_len} but found {found_len}")]
pub struct TryFromBytesError {
    pub expected_max_len: usize,
    pub found_len: usize,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromHexError {
    #[error("missing `0x` prefix")]
    MissingPrefix,
    #[error("cannot parse empty string as hex")]
    EmptyString,
    #[error(transparent)]
    TryFromBytes(#[from] TryFromBytesError),
    #[error(transparent)]
    Hex(#[from] hex::FromHexError),
}

impl FromStr for U256 {
    type Err = ::uint::FromDecStrErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        primitive_types::U256::from_dec_str(s).map(Self)
    }
}

pub use ::uint::FromDecStrErr;

#[cfg(feature = "rlp")]
impl rlp::Encodable for U256 {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.encoder().encode_value(&self.to_be_bytes_packed());
    }
}

#[cfg(feature = "rlp")]
impl rlp::Decodable for U256 {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        <primitive_types::U256 as rlp::Decodable>::decode(rlp).map(Self)
    }
}

impl Rem for U256 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}

impl Add for U256 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for U256 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sum for U256 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(U256::default(), |a, b| a + b)
    }
}

impl Div for U256 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

#[cfg(test)]
mod u256_tests {
    use core::str::FromStr;

    use crate::U256;

    // #[test]
    // fn hex_string() {
    //     #[derive(Debug, Deserialize, Serialize)]
    //     struct T {
    //         #[serde(with = "super::u256_big_endian_hex")]
    //         u256: U256,
    //     }

    //     fn assert_big_endian_hex_roundtrip(hex: &'static str) {
    //         let n: u64 = {
    //             // assume the prefix is provided
    //             let hex = &hex[2..];
    //             let vec = if hex.len() % 2 == 1 {
    //                 hex::decode(format!("0{hex}"))
    //             } else {
    //                 hex::decode(hex)
    //             }
    //             .unwrap();
    //             let vec: Vec<_> = vec![0; 8 - vec.len()].into_iter().chain(vec).collect();
    //             u64::from_be_bytes(vec.try_into().unwrap())
    //         };

    //         let string = format!(r#"{{"u256":"{hex}"}}"#);
    //         let t = serde_json::from_str::<T>(&string).unwrap();

    //         dbg!(<H256>::new(t.u256.to_be_bytes()));

    //         assert_eq!(t.u256.0.as_u64(), n);

    //         let roundtrip = serde_json::to_string(&t).unwrap();

    //         assert_eq!(string, roundtrip);
    //     }

    //     // even length
    //     assert_big_endian_hex_roundtrip("0x1234");

    //     // odd length
    //     assert_big_endian_hex_roundtrip("0x56789");

    //     // single digit
    //     assert_big_endian_hex_roundtrip("0x3");

    //     // zero
    //     assert_big_endian_hex_roundtrip("0x0");
    // }

    #[test]
    fn roundtrip() {
        unionlabs::test_utils::assert_json_roundtrip(&U256::from_str("123456").unwrap());
        unionlabs::test_utils::assert_string_roundtrip(&U256::from_str("123456").unwrap());
    }

    #[test]
    fn from_limbs() {
        assert_eq!(U256::from_limbs([1, 0, 0, 0]), U256::from(1_u64));
    }
}
