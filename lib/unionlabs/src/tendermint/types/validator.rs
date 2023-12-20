use serde::{Deserialize, Serialize};

use crate::{
    bounded::{BoundedI64, BoundedIntError},
    errors::{required, InvalidLength, MissingField},
    hash::H160,
    tendermint::crypto::public_key::PublicKey,
    Proto, TryFromProtoErrorOf, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Validator {
    #[serde(with = "::serde_utils::hex_upper_unprefixed")]
    pub address: H160,
    pub pub_key: PublicKey,
    pub voting_power: BoundedI64<0, { i64::MAX }>,
    pub proposer_priority: i64,
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

#[derive(Debug)]
pub enum TryFromValidatorError {
    MissingField(MissingField),
    Address(InvalidLength),
    VotingPower(BoundedIntError<i64>),
    PubKey(TryFromProtoErrorOf<PublicKey>),
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

impl Proto for Validator {
    type Proto = protos::tendermint::types::Validator;
}

impl TypeUrl for protos::tendermint::types::Validator {
    const TYPE_URL: &'static str = "/tendermint.types.Validator";
}
