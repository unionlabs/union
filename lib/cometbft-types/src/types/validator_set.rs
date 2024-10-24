use serde::{Deserialize, Serialize};

use crate::types::validator::Validator;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidatorSet {
    pub validators: Vec<Validator>,
    pub proposer: Validator,
    // REVIEW: >= 0?
    pub total_voting_power: i64,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{errors::MissingField, required};

    use crate::types::{validator, validator_set::ValidatorSet};

    impl From<ValidatorSet> for protos::tendermint::types::ValidatorSet {
        fn from(value: ValidatorSet) -> Self {
            Self {
                validators: value.validators.into_iter().map(Into::into).collect(),
                proposer: Some(value.proposer.into()),
                total_voting_power: value.total_voting_power,
            }
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid validators")]
        Validators(#[source] validator::proto::Error),
        #[error("invalid proposer")]
        Proposer(#[source] validator::proto::Error),
    }

    impl TryFrom<protos::tendermint::types::ValidatorSet> for ValidatorSet {
        type Error = Error;

        fn try_from(value: protos::tendermint::types::ValidatorSet) -> Result<Self, Self::Error> {
            Ok(Self {
                validators: value
                    .validators
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Error::Validators)?,
                proposer: required!(value.proposer)?
                    .try_into()
                    .map_err(Error::Proposer)?,
                total_voting_power: value.total_voting_power,
            })
        }
    }
}
