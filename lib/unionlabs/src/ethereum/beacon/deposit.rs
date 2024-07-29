use macros::model;
use ssz::{types::Vector, Ssz};

use crate::ethereum::{beacon::deposit_data::DepositData, config::DEPOSIT_CONTRACT_TREE_DEPTH};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#deposit>
#[model]
#[derive(Ssz)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct Deposit<C: DEPOSIT_CONTRACT_TREE_DEPTH> {
    /// Merkle path to deposit root
    pub proof: Vector<[u8; 32], C::DEPOSIT_CONTRACT_TREE_DEPTH>,
    pub data: DepositData,
}

#[model]
pub struct UnboundedDeposit {
    pub proof: Vec<[u8; 32]>,
    pub data: DepositData,
}
