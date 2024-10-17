use macros::model;

use crate::{
    cometbft::types::{
        signed_header::{SignedHeader, TryFromSignedHeaderError},
        validator_set::{TryFromValidatorSetError, ValidatorSet},
    },
    errors::{required, MissingField},
};

#[model(proto(raw(protos::cometbft::types::v1::LightBlock), from, into))]
pub struct LightBlock {
    pub signed_header: SignedHeader,
    pub validator_set: ValidatorSet,
}

impl From<LightBlock> for protos::cometbft::types::v1::LightBlock {
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

impl TryFrom<protos::cometbft::types::v1::LightBlock> for LightBlock {
    type Error = TryFromLightBlockError;

    fn try_from(value: protos::cometbft::types::v1::LightBlock) -> Result<Self, Self::Error> {
        Ok(Self {
            signed_header: required!(value.signed_header)?.try_into()?,
            validator_set: required!(value.validator_set)?.try_into()?,
        })
    }
}
