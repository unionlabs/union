use macros::model;
use ssz::{types::Vector, Ssz};

use crate::{
    ethereum::{beacon::deposit_data::DepositData, config::DEPOSIT_CONTRACT_TREE_DEPTH},
    hash::H256,
};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#deposit>
#[model]
#[derive(Ssz)]
#[cfg_attr(feature = "serde", serde(bound(serialize = "", deserialize = "")))]
pub struct Deposit<C: DEPOSIT_CONTRACT_TREE_DEPTH> {
    /// Merkle path to deposit root
    pub proof: Vector<H256, C::DEPOSIT_CONTRACT_TREE_DEPTH>,
    pub data: DepositData,
}

#[model]
pub struct UnboundedDeposit {
    pub proof: Vec<H256>,
    pub data: DepositData,
}
