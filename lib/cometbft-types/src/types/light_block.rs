use serde::{Deserialize, Serialize};

use crate::types::{
    commit_sig::CommitSig, signed_header::SignedHeader, validator_set::ValidatorSet,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LightBlock {
    pub signed_header: SignedHeader,
    pub validator_set: ValidatorSet,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{errors::MissingField, required};

    use crate::types::{
        light_block::LightBlock,
        signed_header::{self},
        validator_set,
    };

    impl From<LightBlock> for protos::cometbft::types::v1::LightBlock {
        fn from(value: LightBlock) -> Self {
            Self {
                signed_header: Some(value.signed_header.into()),
                validator_set: Some(value.validator_set.into()),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid signed header")]
        SignedHeader(#[from] signed_header::proto::Error),
        #[error("invalid validator set")]
        ValidatorSet(#[from] validator_set::proto::Error),
    }

    impl TryFrom<protos::cometbft::types::v1::LightBlock> for LightBlock {
        type Error = Error;

        fn try_from(value: protos::cometbft::types::v1::LightBlock) -> Result<Self, Self::Error> {
            Ok(Self {
                signed_header: required!(value.signed_header)?.try_into()?,
                validator_set: required!(value.validator_set)?.try_into()?,
            })
        }
    }
}
