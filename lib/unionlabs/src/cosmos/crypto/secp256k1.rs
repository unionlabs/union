use macros::model;

use crate::primitives::{encoding::Base64, FixedBytesError, Hash};

#[model(proto(raw(protos::cosmos::crypto::secp256k1::PubKey), into, from))]
pub struct PubKey {
    pub key: Hash<33, Base64>,
}

impl TryFrom<protos::cosmos::crypto::secp256k1::PubKey> for PubKey {
    type Error = FixedBytesError;

    fn try_from(value: protos::cosmos::crypto::secp256k1::PubKey) -> Result<Self, Self::Error> {
        Ok(Self {
            key: value.key.try_into()?,
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
