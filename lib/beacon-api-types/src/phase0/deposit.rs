use unionlabs::primitives::H256;
#[cfg(feature = "ssz")]
use {crate::chain_spec::DEPOSIT_CONTRACT_TREE_DEPTH, ssz::types::Vector};

use crate::phase0::DepositData;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""))
)]
pub struct Deposit {
    pub proof: Vec<H256>,
    pub data: DepositData,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, Eq, ssz::Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""))
)]
pub struct DepositSsz<C: DEPOSIT_CONTRACT_TREE_DEPTH> {
    /// Merkle path to deposit root
    pub proof: Vector<H256, C::DEPOSIT_CONTRACT_TREE_DEPTH>,
    pub data: DepositData,
}
