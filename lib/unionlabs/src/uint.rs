#![allow(clippy::disallowed_types)] // need to access the inner type to wrap it

use core::{
    fmt::{self, Display},
    iter::Sum,
    ops::{Add, AddAssign, Div, Rem},
    str::FromStr,
};

use serde_utils::HEX_ENCODING_PREFIX;

use crate::errors::{ExpectedLength, InvalidLength};

/// [`primitive_types::U256`] can't roundtrip through string conversion since it parses from hex but displays as decimal.
#[derive(::macros::Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, serde::Deserialize))]
#[debug("U256({:?})", self.0)]
pub struct U256(pub ruint::Uint<256, 4>);

impl fmt::LowerHex for U256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.write_str(&self.to_be_hex())
        } else {
            f.write_str(&self.to_be_hex()[2..])
        }
    }
}

// #[cfg(feature = "ethabi")]
// impl alloy_core::sol_types::SolValue for U256 {
//     type SolType = <ruint::Uint<256, 4> as alloy_core::sol_types::SolValue>::SolType;
// }

impl U256 {
    pub const MAX: Self = Self::from_limbs([u64::MAX; 4]);
    pub const ZERO: Self = Self::from_limbs([0; 4]);
    pub const ONE: Self = Self::from_limbs([1, 0, 0, 0]);

    // one day...
    // pub const fn from_const_str<const STR: &'static str>() -> Self {}
}

impl From<u64> for U256 {
    fn from(value: u64) -> Self {
        Self(ruint::Uint::from(value))
    }
}

impl TryFrom<U256> for u64 {
    type Error = ruint::FromUintError<u64>;

    fn try_from(value: U256) -> Result<Self, Self::Error> {
        value.0.try_into()
    }
}

impl U256 {
    #[must_use]
    #[allow(clippy::missing_panics_doc)] // max value is 255
    pub fn leading_zeros(&self) -> u32 {
        self.0.leading_zeros().try_into().unwrap()
    }

    #[must_use]
    pub fn to_le_bytes(&self) -> [u8; 32] {
        self.0.to_le_bytes()
    }

    #[must_use]
    pub fn to_be_bytes(&self) -> [u8; 32] {
        self.0.to_be_bytes()
    }

    #[must_use]
    pub fn to_be_bytes_packed(&self) -> Vec<u8> {
        let buffer = self.to_be_bytes();
        let leading_empty_bytes = (self.leading_zeros() / 8) as usize;
        buffer[leading_empty_bytes..].to_vec()
    }

    #[allow(clippy::missing_panics_doc)] // precondition is checked
    pub fn try_from_be_bytes(bz: &[u8]) -> Result<Self, InvalidLength> {
        let len = bz.len();

        if (0..=32).contains(&len) {
            Ok(Self(
                ruint::Uint::try_from_be_slice(bz).expect("length is in range; qed;"),
            ))
        } else {
            Err(InvalidLength {
                expected: ExpectedLength::Between(0, 32),
                found: len,
            })
        }
    }

    #[allow(clippy::missing_panics_doc)] // precondition is checked
    pub fn try_from_le_bytes(bz: &[u8]) -> Result<Self, InvalidLength> {
        let len = bz.len();

        if (0..=32).contains(&len) {
            Ok(Self(
                ruint::Uint::try_from_le_slice(bz).expect("length is in range; qed;"),
            ))
        } else {
            Err(InvalidLength {
                expected: ExpectedLength::Between(0, 32),
                found: len,
            })
        }
    }

    #[must_use]
    pub fn from_be_bytes(bz: [u8; 32]) -> Self {
        Self(ruint::Uint::from_be_bytes(bz))
    }

    #[must_use]
    pub fn from_le_bytes(bz: [u8; 32]) -> Self {
        Self(ruint::Uint::from_le_bytes(bz))
    }

    #[must_use]
    pub const fn from_limbs(limbs: [u64; 4]) -> Self {
        Self(ruint::Uint::from_limbs(limbs))
    }

    #[must_use]
    pub const fn as_limbs(&self) -> [u64; 4] {
        *self.0.as_limbs()
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

#[cfg(feature = "serde")]
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

#[cfg(feature = "ssz")]
impl ssz::Ssz for U256 {
    const SSZ_FIXED_LEN: Option<core::num::NonZeroUsize> =
        Some(crate::option_unwrap!(core::num::NonZeroUsize::new(32)));

    const TREE_HASH_TYPE: ssz::tree_hash::TreeHashType =
        ssz::tree_hash::TreeHashType::Basic { size: 32 };

    fn tree_hash_root(&self) -> ssz::tree_hash::Hash256 {
        self.0.to_le_bytes()
    }

    fn ssz_bytes_len(&self) -> core::num::NonZeroUsize {
        crate::option_unwrap!(core::num::NonZeroUsize::new(32))
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::decode::DecodeError> {
        let len = bytes.len();
        let expected = 32;

        if len == expected {
            Ok(Self::try_from_le_bytes(bytes).expect("bytes are in range; qed;"))
        } else {
            Err(ssz::decode::DecodeError::InvalidByteLength {
                found: len,
                expected,
            })
        }
    }
}

#[cfg(feature = "rlp")]
impl rlp::Encodable for U256 {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.encoder().encode_value(&self.to_be_bytes_packed());
    }
}

#[cfg(feature = "rlp")]
impl rlp::Decodable for U256 {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        <ruint::Uint<256, 4> as rlp::Decodable>::decode(rlp).map(Self)
    }
}

impl FromStr for U256 {
    type Err = ruint::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ruint::Uint::from_str_radix(s, 10).map(Self)
    }
}

impl Display for U256 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

impl Rem for U256 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0.checked_rem(rhs.0).expect("attempted to mod zero"))
    }
}

impl Add for U256 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(
            self.0
                .checked_add(rhs.0)
                .expect("attempted to add with overflow"),
        )
    }
}

impl AddAssign for U256 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sum for U256 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        // NOTE: NOT WRAPPING ADD!
        iter.fold(U256::default(), |a, b| a + b)
    }
}

impl Div for U256 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(
            self.0
                .checked_div(rhs.0)
                .expect("attempted to divide by zero"),
        )
    }
}

#[cfg(test)]
mod u256_tests {
    use core::str::FromStr;

    use serde::{Deserialize, Serialize};

    use crate::{
        test_utils::{assert_json_roundtrip, assert_proto_roundtrip, assert_string_roundtrip},
        uint::U256,
    };

    #[test]
    fn hex_string() {
        #[derive(Debug, Deserialize, Serialize)]
        struct Struct {
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
            let t = serde_json::from_str::<Struct>(&string).unwrap();

            // dbg!(<H256>::new(t.u256.to_be_bytes()));

            assert_eq!(u64::try_from(t.u256.0).unwrap(), n);

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
        assert_string_roundtrip(&U256::from_str("123456").unwrap());
    }

    #[test]
    fn one() {
        assert_eq!(U256::ONE, U256::from(1));
        assert_eq!(
            U256::ONE,
            U256::try_from_be_bytes(&1_u64.to_be_bytes()).unwrap()
        );
        assert_eq!(
            U256::ONE,
            U256::try_from_le_bytes(&1_u64.to_le_bytes()).unwrap()
        );
        assert_eq!(U256::ONE, U256::from_str("1").unwrap());
        assert_eq!(U256::ONE, U256::from_be_hex("0x1").unwrap());
        assert_eq!(U256::ONE, U256::from_be_hex("0x01").unwrap());
        assert_eq!(U256::ONE, U256::from_be_hex("0x001").unwrap());
        assert_eq!(U256::ONE, U256::from_be_hex("0x00000000001").unwrap());
        assert_eq!(
            U256::ONE,
            U256::from_be_hex("0x0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap()
        );
    }
}
