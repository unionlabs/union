use macros::model;

use crate::hash::H160;

#[model]
#[cfg_attr(feature = "ssz", derive(::ssz::Ssz))]
pub struct Withdrawal {
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub index: u64,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub validator_index: u64,
    pub address: H160,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub amount: u64,
}
