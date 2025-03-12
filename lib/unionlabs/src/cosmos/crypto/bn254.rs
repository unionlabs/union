use serde::{Deserialize, Serialize};

use crate::primitives::{encoding::Base64, H256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PubKey {
    pub key: H256<Base64>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use super::PubKey;
    use crate::{impl_proto_via_try_from_into, primitives::FixedBytesError};

    impl_proto_via_try_from_into!(PubKey => protos::cosmos::crypto::bn254::PubKey);

    impl TryFrom<protos::cosmos::crypto::bn254::PubKey> for PubKey {
        type Error = FixedBytesError;

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
