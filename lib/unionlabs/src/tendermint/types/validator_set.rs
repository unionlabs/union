use macros::model;

use crate::{
    errors::{required, MissingField},
    tendermint::types::validator::{TryFromValidatorError, Validator},
};

#[model(proto(raw(protos::tendermint::types::ValidatorSet), into, from))]
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

#[derive(Debug, PartialEq, Clone)]
pub enum TryFromValidatorSetError {
    Validators(TryFromValidatorError),
    MissingField(MissingField),
    Proposer(TryFromValidatorError),
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
