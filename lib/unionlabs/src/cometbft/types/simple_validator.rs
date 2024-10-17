use macros::model;

use crate::cometbft::crypto::public_key::PublicKey;

#[model(proto(raw(protos::cometbft::types::v1::SimpleValidator), from))]
pub struct SimpleValidator {
    pub pub_key: PublicKey,
    // REVIEW: is this bounded the same way as Validator?
    pub voting_power: i64,
}

impl From<SimpleValidator> for protos::cometbft::types::v1::SimpleValidator {
    fn from(value: SimpleValidator) -> Self {
        Self {
            pub_key: Some(value.pub_key.into()),
            voting_power: value.voting_power,
        }
    }
}
