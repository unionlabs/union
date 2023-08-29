use serde::{Deserialize, Serialize};

use crate::{errors::InvalidLength, ethereum::H256, Proto, TypeUrl};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PubKey {
    #[serde(with = "::serde_utils::base64")]
    pub key: H256,
}

impl Proto for PubKey {
    type Proto = protos::cosmos::crypto::bn254::PubKey;
}

impl TypeUrl for protos::cosmos::crypto::bn254::PubKey {
    const TYPE_URL: &'static str = "/cosmos.crypto.bn254.PubKey";
}

impl TryFrom<protos::cosmos::crypto::bn254::PubKey> for PubKey {
    type Error = InvalidLength;

    fn try_from(value: protos::cosmos::crypto::bn254::PubKey) -> Result<Self, Self::Error> {
        Ok(Self {
            key: value.key.try_into()?,
        })
    }
}

impl From<PubKey> for protos::cosmos::crypto::bn254::PubKey {
    fn from(value: PubKey) -> Self {
        Self {
            key: value.key.into(),
        }
    }
}
