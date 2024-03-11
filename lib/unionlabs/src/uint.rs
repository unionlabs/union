use core::{fmt::Display, str::FromStr};

use custom_debug_derive::Debug;
use rlp::Encodable;
use serde::{Deserialize, Serialize};
use serde_utils::HEX_ENCODING_PREFIX;
use tree_hash::TreeHash;

use crate::{
    encoding::{Decode, Encode, Proto},
    errors::{ExpectedLength, InvalidLength},
};

/// [`primitive_types::U256`] can't roundtrip through string conversion since it parses from hex but displays as decimal.
#[derive(
    Debug,
    Clone,
    Copy,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    ssz::Encode,
    ssz::Decode,
)]
#[cfg_attr(
    feature = "ethabi",
    derive(
        ethers_contract_derive::EthAbiType,
        ethers_contract_derive::EthAbiCodec
    )
)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[ssz(struct_behaviour = "transparent")]
#[repr(transparent)]
pub struct U256(
    #[serde(with = "::serde_utils::u256_from_dec_str")]
    #[debug(with = "::serde_utils::fmt::display")]
    pub primitive_types::U256,
);

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

impl U256 {
    #[must_use]
    pub fn leading_zeros(&self) -> u32 {
        self.0.leading_zeros()
    }

    #[must_use]
    pub fn to_little_endian(&self) -> [u8; 32] {
        let mut buf = [0; 32];
        self.0.to_little_endian(&mut buf);
        buf
    }

    #[must_use]
    pub fn to_big_endian(&self) -> [u8; 32] {
        let mut buf = [0; 32];
        self.0.to_big_endian(&mut buf);
        buf
    }

    #[must_use]
    pub fn to_packed_big_endian(&self) -> Vec<u8> {
        let buffer = self.to_big_endian();
        let leading_empty_bytes = (self.leading_zeros() / 8) as usize;
        buffer[leading_empty_bytes..].to_vec()
    }

    pub fn try_from_big_endian(bz: &[u8]) -> Result<Self, InvalidLength> {
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
    pub fn from_big_endian(bz: [u8; 32]) -> Self {
        Self(primitive_types::U256::from_big_endian(&bz))
    }

    #[must_use]
    pub fn to_big_endian_hex(&self) -> String {
        let data = self.to_big_endian();
        let data = data.as_ref();

        let encoded = hex::encode(data);

        let encoded = encoded.trim_start_matches('0');

        format!(
            "{HEX_ENCODING_PREFIX}{}",
            if encoded.is_empty() { "0" } else { encoded }
        )
    }
}

pub mod u256_big_endian_hex {
    use serde::de::{self, Deserialize};

    use crate::uint::U256;

    pub fn serialize<S>(data: &U256, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&data.to_big_endian_hex())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<U256, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <&str>::deserialize(deserializer).and_then(|s| -> Result<U256, D::Error> {
            if s.is_empty() {
                return Err(de::Error::custom(
                    serde_utils::FromHexStringError::EmptyString,
                ));
            }

            match s.strip_prefix(serde_utils::HEX_ENCODING_PREFIX) {
                Some(maybe_hex) => if maybe_hex.len() % 2 == 1 {
                    hex::decode(format!("0{maybe_hex}"))
                } else {
                    hex::decode(maybe_hex)
                }
                .map(|x| U256::try_from_big_endian(&x).map_err(de::Error::custom))
                .map_err(de::Error::custom)?,
                None => Err(de::Error::custom(
                    serde_utils::FromHexStringError::MissingPrefix(
                        String::from_utf8_lossy(s.as_ref()).into_owned(),
                    ),
                )),
            }
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
        self.to_big_endian().into()
    }
}

impl Decode<Proto> for U256 {
    type Error = InvalidLength;

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::try_from_big_endian(bytes)
    }
}

impl TreeHash for U256 {
    fn tree_hash_type() -> tree_hash::TreeHashType {
        primitive_types::U256::tree_hash_type()
    }

    fn tree_hash_packed_encoding(&self) -> tree_hash::PackedEncoding {
        self.0.tree_hash_packed_encoding()
    }

    fn tree_hash_packing_factor() -> usize {
        primitive_types::U256::tree_hash_packing_factor()
    }

    fn tree_hash_root(&self) -> tree_hash::Hash256 {
        self.0.tree_hash_root()
    }
}

impl FromStr for U256 {
    type Err = uint::FromDecStrErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        primitive_types::U256::from_dec_str(s).map(Self)
    }
}

impl Display for U256 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

impl Encodable for U256 {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.encoder().encode_value(&self.to_packed_big_endian());
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

            dbg!(H256(t.u256.to_big_endian()));

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
