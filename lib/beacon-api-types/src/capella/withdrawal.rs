use unionlabs::primitives::H160;

use crate::custom_types::{Gwei, ValidatorIndex, WithdrawalIndex};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Withdrawal {
    pub index: WithdrawalIndex,
    pub validator_index: ValidatorIndex,
    pub address: H160,
    pub amount: Gwei,
}
