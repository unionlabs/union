use macros::model;

use crate::{ethereum::beacon::checkpoint::Checkpoint, hash::H256};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#attestationdata>
#[model]
#[cfg_attr(feature = "ssz", derive(::ssz::Ssz))]
pub struct AttestationData {
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub slot: u64,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub index: u64,
    /// LMD GHOST vote
    pub beacon_block_root: H256,
    /// FFG vote
    pub source: Checkpoint,
    pub target: Checkpoint,
}
