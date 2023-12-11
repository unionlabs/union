use serde::{Deserialize, Serialize};

use crate::{
    errors::{ExpectedLength, InvalidLength},
    Proto, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PubKey {
    #[serde(with = "::serde_utils::base64")]
    pub key: [u8; 33],
}

impl Proto for PubKey {
    type Proto = protos::cosmos::crypto::secp256k1::PubKey;
}

impl TypeUrl for protos::cosmos::crypto::secp256k1::PubKey {
    const TYPE_URL: &'static str = "/cosmos.crypto.secp256k1.PubKey";
}

impl TryFrom<protos::cosmos::crypto::secp256k1::PubKey> for PubKey {
    type Error = InvalidLength;

    fn try_from(value: protos::cosmos::crypto::secp256k1::PubKey) -> Result<Self, Self::Error> {
        Ok(Self {
            key: value
                .key
                .try_into()
                .map_err(|invalid: Vec<u8>| InvalidLength {
                    expected: ExpectedLength::Exact(32),
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
