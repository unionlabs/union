use borsh::{BorshDeserialize, BorshSerialize};
use near_primitives_core::types::Balance;
use near_sdk::AccountId;

use crate::near::types::PublicKey;

#[derive(
    BorshSerialize,
    BorshDeserialize,
    serde::Serialize,
    serde::Deserialize,
    Debug,
    Clone,
    Eq,
    PartialEq,
)]
#[serde(tag = "validator_stake_struct_version")]
pub enum ValidatorStakeView {
    V1(ValidatorStakeViewV1),
}

#[derive(
    BorshSerialize,
    BorshDeserialize,
    Debug,
    Clone,
    Eq,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct ValidatorStakeViewV1 {
    pub account_id: AccountId,
    pub public_key: PublicKey,
    // TODO(aeryz): #[serde(with = "dec_format")]
    pub stake: Balance,
}
