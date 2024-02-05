use serde::{Deserialize, Serialize};

use crate::{tendermint::crypto::public_key::PublicKey, Proto};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct SimpleValidator {
    pub pub_key: PublicKey,
    // REVIEW: is this bounded the same way as Validator?
    pub voting_power: i64,
}

impl From<SimpleValidator> for protos::tendermint::types::SimpleValidator {
    fn from(value: SimpleValidator) -> Self {
        Self {
            pub_key: Some(value.pub_key.into()),
            voting_power: value.voting_power,
        }
    }
}

impl Proto for SimpleValidator {
    type Proto = protos::tendermint::types::SimpleValidator;
}
