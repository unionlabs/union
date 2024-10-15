use macros::model;

use crate::tendermint::types::{signed_header::SignedHeader, validator_set::ValidatorSet};

#[model(proto(raw(protos::tendermint::types::LightBlock), from, into))]
pub struct LightBlock {
    pub signed_header: SignedHeader,
    pub validator_set: ValidatorSet,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        errors::{required, MissingField},
        tendermint::types::{
            light_block::LightBlock, signed_header::proto::TryFromSignedHeaderError,
            validator_set::proto::TryFromValidatorSetError,
        },
    };

    impl From<LightBlock> for protos::tendermint::types::LightBlock {
        fn from(value: LightBlock) -> Self {
            Self {
                signed_header: Some(value.signed_header.into()),
                validator_set: Some(value.validator_set.into()),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum TryFromLightBlockError {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid signed header")]
        SignedHeader(#[from] TryFromSignedHeaderError),
        #[error("invalid validator set")]
        ValidatorSet(#[from] TryFromValidatorSetError),
    }

    impl TryFrom<protos::tendermint::types::LightBlock> for LightBlock {
        type Error = TryFromLightBlockError;

        fn try_from(value: protos::tendermint::types::LightBlock) -> Result<Self, Self::Error> {
            Ok(Self {
                signed_header: required!(value.signed_header)?.try_into()?,
                validator_set: required!(value.validator_set)?.try_into()?,
            })
        }
    }
}
