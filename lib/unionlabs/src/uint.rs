#![allow(clippy::disallowed_types)] // need to access the inner type to wrap it

use core::{
    fmt::{self, Display},
    iter::Sum,
    num::NonZeroUsize,
    ops::{Add, AddAssign, Div, Rem},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use serde_utils::HEX_ENCODING_PREFIX;

use crate::{
    encoding::{Decode, Encode, Proto},
    errors::{ExpectedLength, InvalidLength},
    option_unwrap,
};

/// [`primitive_types::U256`] can't roundtrip through string conversion since it parses from hex but displays as decimal.
#[derive(
    ::macros::Debug,
    Clone,
    Copy,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
    Serialize,
    Deserialize,
)]
#[repr(transparent)]
#[debug("U256({})", self)]
pub struct U256(#[serde(with = "::serde_utils::u256_from_dec_str")] pub primitive_types::U256);

impl fmt::LowerHex for U256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.write_str(&self.to_be_hex())
        } else {
            f.write_str(&self.to_be_hex()[2..])
        }
    }
}

#[cfg(feature = "ethabi")]
mod ethabi {
    use ethers::core::abi::{
        AbiArrayType, AbiDecode, AbiEncode, AbiError, AbiType, InvalidOutputType, ParamType, Token,
        Tokenizable, TokenizableItem,
    };

    use crate::uint::U256;

    impl AbiType for U256 {
        fn param_type() -> ParamType {
            <primitive_types::U256 as AbiType>::param_type()
        }
    }

    impl AbiArrayType for U256 {}
    impl Tokenizable for U256 {
        fn from_token(token: Token) -> Result<Self, InvalidOutputType> {
            <primitive_types::U256 as Tokenizable>::from_token(token).map(Self)
        }
        fn into_token(self) -> Token {
            <primitive_types::U256 as Tokenizable>::into_token(self.0)
        }
    }

    impl TokenizableItem for U256 {}

    impl AbiDecode for U256 {
        fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, AbiError> {
            <primitive_types::U256 as AbiDecode>::decode(bytes).map(Self)
        }
    }

    impl AbiEncode for U256 {
        fn encode(self) -> Vec<u8> {
            <primitive_types::U256 as AbiEncode>::encode(self.0)
        }
    }
}

impl U256 {
    pub const MAX: Self = Self::from_limbs([u64::MAX; 4]);
    pub const ZERO: Self = Self::from_limbs([0; 4]);

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

    pub fn try_from_be_bytes(bz: &[u8]) -> Result<Self, InvalidLength> {
        let len = bz.len();

        if (0..=32).contains(&len) {
            Ok(Self(primitive_types::U256::from_big_endian(bz)))
        } else {
            Err(InvalidLength {
                expected: ExpectedLength::Between(0, 32),
                found: len,
            })
        }
    }

    #[must_use]
    pub fn from_be_bytes(bz: [u8; 32]) -> Self {
        Self(primitive_types::U256::from_big_endian(&bz))
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
        serde_utils::to_hex(self.to_be_bytes())
    }

    #[must_use]
    pub fn to_be_hex_packed(&self) -> String {
        if self.0.is_zero() {
            format!("{HEX_ENCODING_PREFIX}{}", 0)
        } else {
            // NOTE: Can't use serde_utils::to_hex here as it ensures there's an even number of bytes (by prepending a 0) which we don't want for u256
            format!(
                "{HEX_ENCODING_PREFIX}{}",
                hex::encode(self.to_be_bytes_packed()).trim_start_matches('0')
            )
        }
    }

    pub fn from_be_hex(s: impl AsRef<str>) -> Result<U256, TryFromHexError> {
        if s.as_ref().is_empty() {
            return Err(serde_utils::FromHexStringError::EmptyString.into());
        }

        s.as_ref()
            .strip_prefix(HEX_ENCODING_PREFIX)
            .ok_or_else(|| serde_utils::FromHexStringError::MissingPrefix(s.as_ref().to_owned()))
            .map_err(Into::into)
            .and_then(|maybe_hex| {
                Ok(U256::try_from_be_bytes(
                    &if maybe_hex.len() % 2 == 1 {
                        hex::decode(format!("0{maybe_hex}"))
                    } else {
                        hex::decode(maybe_hex)
                    }
                    .map_err(serde_utils::FromHexStringError::Hex)?,
                )?)
            })
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromHexError {
    #[error("error parsing hex")]
    Hex(#[from] serde_utils::FromHexStringError),
    #[error("error converting from bytes")]
    FromBytes(#[from] InvalidLength),
}

pub mod u256_big_endian_hex {
    use serde::de::{self, Deserialize};

    use crate::uint::U256;

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

// impl TryFrom<Vec<u8>> for U256 {
//     type Error = InvalidLength;

//     fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
//         if value.len() > 32 {
//             Err(InvalidLength {
//                 expected: ExpectedLength::LessThan(32),
//                 found: value.len(),
//             })
//         } else {
//             // NOTE: This can panic if len > 32, hence the check above
//             Ok(Self(primitive_types::U256::from_little_endian(&value)))
//         }
//     }
// }

// // REVIEW: Should this trim leading zeros?
// impl From<U256> for Vec<u8> {
//     fn from(value: U256) -> Self {
//         let mut slice = [0_u8; 32];
//         value.0.to_little_endian(&mut slice);
//         slice.into()
//     }
// }

impl Encode<Proto> for U256 {
    fn encode(self) -> Vec<u8> {
        self.to_be_bytes().into()
    }
}

impl Decode<Proto> for U256 {
    type Error = InvalidLength;

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::try_from_be_bytes(bytes)
    }
}

impl ssz::Ssz for U256 {
    const SSZ_FIXED_LEN: Option<NonZeroUsize> = Some(option_unwrap!(NonZeroUsize::new(32)));

    const TREE_HASH_TYPE: ssz::tree_hash::TreeHashType =
        ssz::tree_hash::TreeHashType::Basic { size: 32 };

    fn tree_hash_root(&self) -> ssz::tree_hash::Hash256 {
        let mut result = [0; 32];
        self.0.to_little_endian(&mut result[..]);
        result
    }

    fn ssz_bytes_len(&self) -> NonZeroUsize {
        option_unwrap!(NonZeroUsize::new(32))
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        let n = 32;
        let s = buf.len();

        buf.resize(s + n, 0);
        self.0.to_little_endian(&mut buf[s..]);
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::decode::DecodeError> {
        let len = bytes.len();
        let expected = 32;

        if len == expected {
            Ok(Self(primitive_types::U256::from_little_endian(bytes)))
        } else {
            Err(ssz::decode::DecodeError::InvalidByteLength {
                found: len,
                expected,
            })
        }
    }
}

impl FromStr for U256 {
    type Err = uint::FromDecStrErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        primitive_types::U256::from_dec_str(s).map(Self)
    }
}

pub use uint::FromDecStrErr;

impl Display for U256 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

impl rlp::Encodable for U256 {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.encoder().encode_value(&self.to_be_bytes_packed());
    }
}

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

    use serde::{Deserialize, Serialize};

    use crate::{
        hash::H256,
        test_utils::{assert_json_roundtrip, assert_proto_roundtrip, assert_string_roundtrip},
        uint::U256,
    };

    #[test]
    fn hex_string() {
        #[derive(Debug, Deserialize, Serialize)]
        struct T {
            #[serde(with = "super::u256_big_endian_hex")]
            u256: U256,
        }

        fn assert_big_endian_hex_roundtrip(hex: &'static str) {
            let n: u64 = {
                // assume the prefix is provided
                let hex = &hex[2..];
                let vec = if hex.len() % 2 == 1 {
                    hex::decode(format!("0{hex}"))
                } else {
                    hex::decode(hex)
                }
                .unwrap();
                let vec: Vec<_> = vec![0; 8 - vec.len()].into_iter().chain(vec).collect();
                u64::from_be_bytes(vec.try_into().unwrap())
            };

            let string = format!(r#"{{"u256":"{hex}"}}"#);
            let t = serde_json::from_str::<T>(&string).unwrap();

            dbg!(<H256>::new(t.u256.to_be_bytes()));

            assert_eq!(t.u256.0.as_u64(), n);

            let roundtrip = serde_json::to_string(&t).unwrap();

            assert_eq!(string, roundtrip);
        }

        // even length
        assert_big_endian_hex_roundtrip("0x1234");

        // odd length
        assert_big_endian_hex_roundtrip("0x56789");

        // single digit
        assert_big_endian_hex_roundtrip("0x3");

        // zero
        assert_big_endian_hex_roundtrip("0x0");
    }

    #[test]
    fn roundtrip() {
        assert_json_roundtrip(&U256::from_str("123456").unwrap());
        assert_proto_roundtrip(&U256::from_str("123456").unwrap());
        assert_string_roundtrip(&U256::from_str("123456").unwrap());
    }
}
