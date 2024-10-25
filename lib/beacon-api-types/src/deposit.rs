use crate::DepositData;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    bound(serialize = "", deserialize = "")
)]
pub struct Deposit {
    pub proof: Vec<H256>,
    pub data: DepositData,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, ssz::Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    bound(serialize = "", deserialize = "")
)]
pub struct DepositSsz<C: crate::DEPOSIT_CONTRACT_TREE_DEPTH> {
    /// Merkle path to deposit root
    pub proof: ssz::types::Vector<H256, C::DEPOSIT_CONTRACT_TREE_DEPTH>,
    pub data: DepositData,
}
