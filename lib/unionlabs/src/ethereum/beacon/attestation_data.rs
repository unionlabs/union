use macros::model;
use ssz::Ssz;

use crate::{ethereum::beacon::checkpoint::Checkpoint, hash::H256};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#attestationdata>
#[model]
#[derive(Ssz)]
pub struct AttestationData {
    #[serde(with = "::serde_utils::string")]
    pub slot: u64,
    #[serde(with = "::serde_utils::string")]
    pub index: u64,
    /// LMD GHOST vote
    pub beacon_block_root: H256,
    /// FFG vote
    pub source: Checkpoint,
    pub target: Checkpoint,
}
