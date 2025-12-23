use core::fmt;

use ibc_union_light_client::spec::Timestamp;
use serde::{Deserialize, Serialize};
use unionlabs::primitives::Bytes;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct Attestation {
    pub chain_id: String,
    pub height: u64,
    pub timestamp: Timestamp,
    pub key: Bytes,
    pub value: AttestationValue,
}

#[derive(Debug, Clone, PartialEq, bincode::Encode, bincode::Decode)]
pub struct AttestationKey {
    pub chain_id: String,
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
        encoding::{Bincode, Json},
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

    #[test]
    fn attestation_key_bincode() {
        assert_codec_iso_bytes::<_, Bincode>(
            &AttestationKey {
                chain_id: "999".to_owned(),
                height: 1,
                key: b"key".into(),
            },
            &hex!(
                "0300000000000000" // chain id length
                "393939"           // chain id
                "0100000000000000" // height
                "0300000000000000" // key length
                "6b6579"           // b"key"
            ),
        );
    }

    #[test]
    fn attestation_json() {
        assert_codec_iso_bytes::<_, Json>(
            &Attestation {
                chain_id: "999".to_owned(),
                height: 1,
                timestamp: Timestamp::from_nanos(2),
                key: b"key".into(),
                value: AttestationValue::Existence([0x00].into()),
            },
            br#"{"chain_id":"999","height":1,"timestamp":2,"key":"0x6b6579","value":{"existence":"0x00"}}"#,
        );

        assert_codec_iso_bytes::<_, Json>(
            &Attestation {
                chain_id: "999".to_owned(),
                height: 1,
                timestamp: Timestamp::from_nanos(2),
                key: b"key".into(),
                value: AttestationValue::NonExistence,
            },
            br#"{"chain_id":"999","height":1,"timestamp":2,"key":"0x6b6579","value":"non_existence"}"#,
        );
    }

    #[test]
    fn attestation_bincode() {
        assert_codec_iso_bytes::<_, Bincode>(
            &Attestation {
                chain_id: "999".to_owned(),
                height: 1,
                timestamp: Timestamp::from_nanos(2),
                key: b"key".into(),
                value: AttestationValue::Existence([0x00].into()),
            },
            &hex!(
                "0300000000000000" // chain id length
                "393939"           // chain id
                "0100000000000000" // height
                "0200000000000000" // timestamp
                "0300000000000000" // key length
                "6b6579"           // b"key"
                "01000000"         // variant
                "0100000000000000" // byte length
                "00"               // bytes
            ),
        );

        assert_codec_iso_bytes::<_, Bincode>(
            &Attestation {
                chain_id: "999".to_owned(),
                height: 1,
                timestamp: Timestamp::from_nanos(2),
                key: b"key".into(),
                value: AttestationValue::NonExistence,
            },
            &hex!(
                "0300000000000000" // chain id length
                "393939"           // chain id
                "0100000000000000" // height
                "0200000000000000" // timestamp
                "0300000000000000" // key length
                "6b6579"           // b"key"
                "00000000"         // variant
            ),
        );
    }
}
