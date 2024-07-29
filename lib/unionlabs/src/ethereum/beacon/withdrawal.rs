use macros::model;
use ssz::Ssz;

use crate::hash::H160;

#[model]
#[derive(Ssz)]
pub struct Withdrawal {
    #[serde(with = "::serde_utils::string")]
    pub index: u64,
    #[serde(with = "::serde_utils::string")]
    pub validator_index: u64,
    pub address: H160,
    #[serde(with = "::serde_utils::string")]
    pub amount: u64,
}
