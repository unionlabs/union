use macros::model;

use crate::errors::{ExpectedLength, InvalidLength};

#[model(proto(raw(protos::cosmos::crypto::secp256k1::PubKey), into, from))]
pub struct PubKey {
    #[serde(with = "::serde_utils::base64")]
    pub key: [u8; 33],
}

impl TryFrom<protos::cosmos::crypto::secp256k1::PubKey> for PubKey {
    type Error = InvalidLength;

    fn try_from(value: protos::cosmos::crypto::secp256k1::PubKey) -> Result<Self, Self::Error> {
        Ok(Self {
            key: value
                .key
                .try_into()
                .map_err(|invalid: Vec<u8>| InvalidLength {
                    expected: ExpectedLength::Exact(33),
                    found: invalid.len(),
                })?,
        })
    }
}

impl From<PubKey> for protos::cosmos::crypto::secp256k1::PubKey {
    fn from(value: PubKey) -> Self {
        Self {
            key: value.key.into(),
        }
    }
}
