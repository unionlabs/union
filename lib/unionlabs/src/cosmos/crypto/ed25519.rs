use macros::model;
use unionlabs_bytes::{encoding::Base64, FixedBytesError, H256};

#[model(proto(raw(protos::cosmos::crypto::ed25519::PubKey), into, from))]
pub struct PubKey {
    pub key: H256<Base64>,
}

impl TryFrom<protos::cosmos::crypto::ed25519::PubKey> for PubKey {
    type Error = FixedBytesError;

    fn try_from(value: protos::cosmos::crypto::ed25519::PubKey) -> Result<Self, Self::Error> {
        Ok(Self {
            key: value.key.try_into()?,
        })
    }
}

impl From<PubKey> for protos::cosmos::crypto::ed25519::PubKey {
    fn from(value: PubKey) -> Self {
        Self {
            key: value.key.into(),
        }
    }
}
