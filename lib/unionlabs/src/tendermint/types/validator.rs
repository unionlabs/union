use serde::{Deserialize, Serialize};

use crate::{
    errors::{required, MissingField},
    tendermint::crypto::public_key::PublicKey,
    Proto, TryFromProtoErrorOf, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Validator {
    pub address: Vec<u8>,
    pub pub_key: PublicKey,
    pub voting_power: i64,
    pub proposer_priority: i64,
}

impl From<Validator> for protos::tendermint::types::Validator {
    fn from(value: Validator) -> Self {
        Self {
            address: value.address,
            pub_key: Some(value.pub_key.into()),
            voting_power: value.voting_power,
            proposer_priority: value.proposer_priority,
        }
    }
}

#[derive(Debug)]
pub enum TryFromValidatorError {
    MissingField(MissingField),
    PubKey(TryFromProtoErrorOf<PublicKey>),
}

impl TryFrom<protos::tendermint::types::Validator> for Validator {
    type Error = TryFromValidatorError;

    fn try_from(value: protos::tendermint::types::Validator) -> Result<Self, Self::Error> {
        Ok(Self {
            address: value.address,
            pub_key: required!(value.pub_key)?
                .try_into()
                .map_err(TryFromValidatorError::PubKey)?,
            voting_power: value.voting_power,
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
