use core::fmt;

use ibc_union_light_client::spec::Timestamp;
use serde::{Deserialize, Serialize};
use unionlabs::primitives::Bytes;

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct Attestation {
    pub height: u64,
    pub timestamp: Timestamp,
    pub key: Bytes,
    pub value: AttestationValue,
}

#[derive(Debug, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct AttestationKey {
    pub height: u64,
    pub key: Bytes,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[repr(u8)]
pub enum AttestationValue {
    NonExistence = 0,
    Existence(Bytes) = 1,
}

impl fmt::Display for AttestationValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonExistence => write!(f, "non-existence"),
            Self::Existence(value) => write!(f, "existence:{value}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use unionlabs::{
        encoding::{Bincode, EncodeAs, Json},
        test_utils::assert_codec_iso_bytes,
    };

    use super::*;

    #[test]
    fn attestation_value_json() {
        assert_codec_iso_bytes::<_, Json>(&AttestationValue::NonExistence, br#""non_existence""#);

        assert_codec_iso_bytes::<_, Json>(
            &AttestationValue::Existence([0x00].into()),
            br#"{"existence":"0x00"}"#,
        );
    }

    #[test]
    fn attestation_value_bincode() {
        assert_codec_iso_bytes::<_, Bincode>(&AttestationValue::NonExistence, &hex!("00000000"));

        assert_codec_iso_bytes::<_, Bincode>(
            &AttestationValue::Existence([0x00].into()),
            &hex!(
                "01000000"         // variant
                "0100000000000000" // byte length
                "00"               // bytes
            ),
        );
    }
}
