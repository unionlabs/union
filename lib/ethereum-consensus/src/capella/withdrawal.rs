use crate::primitives::{ExecutionAddress, Gwei, ValidatorIndex, WithdrawalIndex};
use ssz_rs::prelude::*;

#[derive(Default, Debug, Clone, SimpleSerialize, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Withdrawal {
    #[serde(with = "crate::serde::as_string")]
    pub index: WithdrawalIndex,
    #[serde(with = "crate::serde::as_string")]
    pub validator_index: ValidatorIndex,
    pub address: ExecutionAddress,
    #[serde(with = "crate::serde::as_string")]
    pub amount: Gwei,
}
