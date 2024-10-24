use serde::{Deserialize, Serialize};

use crate::crypto::public_key::PublicKey;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimpleValidator {
    pub pub_key: PublicKey,
    // REVIEW: is this bounded the same way as Validator?
    pub voting_power: i64,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{errors::MissingField, impl_proto_via_try_from_into, required};

    use crate::{crypto::public_key, types::simple_validator::SimpleValidator};

    impl_proto_via_try_from_into!(SimpleValidator => protos::tendermint::types::SimpleValidator);

    impl From<SimpleValidator> for protos::tendermint::types::SimpleValidator {
        fn from(value: SimpleValidator) -> Self {
            Self {
                pub_key: Some(value.pub_key.into()),
                voting_power: value.voting_power,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid pub_key")]
        PubKey(#[from] public_key::proto::Error),
    }

    impl TryFrom<protos::tendermint::types::SimpleValidator> for SimpleValidator {
        type Error = Error;

        fn try_from(
            value: protos::tendermint::types::SimpleValidator,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                pub_key: required!(value.pub_key)?.try_into()?,
                voting_power: value.voting_power,
            })
        }
    }
}
