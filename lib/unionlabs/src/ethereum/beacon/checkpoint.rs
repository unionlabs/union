use macros::model;


use crate::hash::H256;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#checkpoint>
#[model]
#[cfg_attr(feature = "ssz", derive(::ssz::Ssz))]
pub struct Checkpoint {
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub epoch: u64,
    pub root: H256,
}
