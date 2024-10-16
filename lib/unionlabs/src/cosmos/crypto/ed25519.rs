use macros::model;

use crate::hash::H256;

#[model(proto(raw(protos::cosmos::crypto::ed25519::PubKey), into, from))]
pub struct PubKey {
    pub key: H256<crate::hash::hash_v2::Base64>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{cosmos::crypto::ed25519::PubKey, errors::InvalidLength};

    impl TryFrom<protos::cosmos::crypto::ed25519::PubKey> for PubKey {
        type Error = InvalidLength;

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
}
