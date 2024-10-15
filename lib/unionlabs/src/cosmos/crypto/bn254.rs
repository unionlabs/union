use macros::model;

use crate::hash::{hash_v2::Base64, H256};

#[model(proto(raw(protos::cosmos::crypto::bn254::PubKey), into, from))]
pub struct PubKey {
    pub key: H256<Base64>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{cosmos::crypto::bn254::PubKey, errors::InvalidLength};

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
}
