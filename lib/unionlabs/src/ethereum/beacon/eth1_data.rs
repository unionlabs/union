use macros::model;


use crate::hash::H256;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#eth1data>
#[model]
#[cfg_attr(feature = "ssz", derive(::ssz::Ssz))]
pub struct Eth1Data {
    pub deposit_root: H256,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub deposit_count: u64,
    pub block_hash: H256,
}
