use macros::proto;
use serde::{Deserialize, Serialize};

use crate::{errors::InvalidLength, hash::H256};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[proto(raw = protos::cosmos::crypto::bn254::PubKey, into, from)]
pub struct PubKey {
    #[serde(with = "::serde_utils::base64")]
    pub key: H256,
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
