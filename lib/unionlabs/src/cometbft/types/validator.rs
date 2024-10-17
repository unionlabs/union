use macros::model;

use crate::{
    bounded::{BoundedI64, BoundedIntError},
    cometbft::crypto::public_key::{PublicKey, TryFromPublicKeyError},
    errors::{required, InvalidLength, MissingField},
    hash::H160,
};

// TODO: Abstract over the pubkey type here somehow? Such that we don't have duplicate fields
#[model(proto(raw(protos::cometbft::types::v1::Validator), from))]
pub struct Validator {
    pub address: H160,
    pub pub_key: PublicKey,
    pub voting_power: BoundedI64<0, { i64::MAX }>,
    pub proposer_priority: i64,
}

impl From<Validator> for protos::cometbft::types::v1::Validator {
    fn from(value: Validator) -> Self {
        Self {
            address: value.address.into(),
            pub_key: Some(value.pub_key.into()),
            voting_power: value.voting_power.into(),
            proposer_priority: value.proposer_priority,
            pub_key_bytes: todo!(),
            pub_key_type: todo!(),
        }
    }
}

impl From<Validator> for protos::tendermint::types::Validator {
    fn from(value: Validator) -> Self {
        Self {
            address: value.address.into(),
            pub_key: Some(value.pub_key.into()),
            voting_power: value.voting_power.into(),
            proposer_priority: value.proposer_priority,
        }
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromValidatorError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid address")]
    Address(#[source] InvalidLength),
    #[error("invalid voting power")]
    VotingPower(#[source] BoundedIntError<i64>),
    #[error("invalid pubkey")]
    PubKey(#[source] TryFromPublicKeyError),
}

impl TryFrom<protos::cometbft::types::v1::Validator> for Validator {
    type Error = TryFromValidatorError;

    fn try_from(value: protos::cometbft::types::v1::Validator) -> Result<Self, Self::Error> {
        Ok(Self {
            address: value
                .address
                .try_into()
                .map_err(TryFromValidatorError::Address)?,
            pub_key: required!(value.pub_key)?
                .try_into()
                .map_err(TryFromValidatorError::PubKey)?,
            voting_power: value
                .voting_power
                .try_into()
                .map_err(TryFromValidatorError::VotingPower)?,
            proposer_priority: value.proposer_priority,
        })
    }
}

impl TryFrom<protos::tendermint::types::Validator> for Validator {
    type Error = TryFromValidatorError;

    fn try_from(value: protos::tendermint::types::Validator) -> Result<Self, Self::Error> {
        Ok(Self {
            address: value
                .address
                .try_into()
                .map_err(TryFromValidatorError::Address)?,
            pub_key: required!(value.pub_key)?
                .try_into()
                .map_err(TryFromValidatorError::PubKey)?,
            voting_power: value
                .voting_power
                .try_into()
                .map_err(TryFromValidatorError::VotingPower)?,
            proposer_priority: value.proposer_priority,
        })
    }
}
