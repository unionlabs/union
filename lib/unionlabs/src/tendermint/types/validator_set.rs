use serde::{Deserialize, Serialize};

use crate::{
    errors::{required, MissingField},
    tendermint::types::validator::Validator,
    Proto, TryFromProtoErrorOf, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidatorSet {
    pub validators: Vec<Validator>,
    pub proposer: Validator,
    // REVIEW: >= 0?
    pub total_voting_power: i64,
}

impl From<ValidatorSet> for protos::tendermint::types::ValidatorSet {
    fn from(value: ValidatorSet) -> Self {
        Self {
            validators: value.validators.into_iter().map(Into::into).collect(),
            proposer: Some(value.proposer.into()),
            total_voting_power: value.total_voting_power,
        }
    }
}

#[derive(Debug)]
pub enum TryFromValidatorSetError {
    Validators(TryFromProtoErrorOf<Validator>),
    MissingField(MissingField),
    Proposer(TryFromProtoErrorOf<Validator>),
}

impl TryFrom<protos::tendermint::types::ValidatorSet> for ValidatorSet {
    type Error = TryFromValidatorSetError;

    fn try_from(value: protos::tendermint::types::ValidatorSet) -> Result<Self, Self::Error> {
        Ok(Self {
            validators: value
                .validators
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()
                .map_err(TryFromValidatorSetError::Validators)?,
            proposer: required!(value.proposer)?
                .try_into()
                .map_err(TryFromValidatorSetError::Proposer)?,
            total_voting_power: value.total_voting_power,
        })
    }
}

impl Proto for ValidatorSet {
    type Proto = protos::tendermint::types::ValidatorSet;
}

impl TypeUrl for protos::tendermint::types::ValidatorSet {
    const TYPE_URL: &'static str = "/tendermint.types.ValidatorSet";
}
