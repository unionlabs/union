use macros::model;

use crate::{
    bounded::{BoundedI64, BoundedIntError},
    errors::{required, MissingField},
    google::protobuf::timestamp::{Timestamp, TryFromTimestampError},
    tendermint::types::{
        light_block::{LightBlock, TryFromLightBlockError},
        validator::{TryFromValidatorError, Validator},
    },
};

#[model(proto(raw(protos::tendermint::types::LightClientAttackEvidence)))]
pub struct LightClientAttackEvidence {
    pub conflicting_block: LightBlock,
    pub common_height: BoundedI64<1, { i64::MAX }>,
    pub byzantine_validators: Vec<Validator>,
    pub total_voting_power: i64,
    pub timestamp: Timestamp,
}

impl From<LightClientAttackEvidence> for protos::tendermint::types::LightClientAttackEvidence {
    fn from(value: LightClientAttackEvidence) -> Self {
        Self {
            conflicting_block: Some(value.conflicting_block.into()),
            common_height: value.common_height.into(),
            byzantine_validators: value
                .byzantine_validators
                .into_iter()
                .map(Into::into)
                .collect(),
            total_voting_power: value.total_voting_power,
            timestamp: Some(value.timestamp.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromLightClientAttackEvidenceError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid conflicting block")]
    ConflictingBlock(#[from] TryFromLightBlockError),
    #[error("invalid common height")]
    CommonHeight(#[from] BoundedIntError<i64>),
    #[error("invalid byzantine validators")]
    ByzantineValidators(#[from] TryFromValidatorError),
    #[error("invalid timestamp")]
    ValidatorSet(#[from] TryFromTimestampError),
}

impl TryFrom<protos::tendermint::types::LightClientAttackEvidence> for LightClientAttackEvidence {
    type Error = TryFromLightClientAttackEvidenceError;

    fn try_from(
        value: protos::tendermint::types::LightClientAttackEvidence,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            conflicting_block: required!(value.conflicting_block)?.try_into()?,
            common_height: value.common_height.try_into()?,
            byzantine_validators: value
                .byzantine_validators
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            total_voting_power: value.total_voting_power,
            timestamp: required!(value.timestamp)?.try_into()?,
        })
    }
}
