use unionlabs::primitives::H256;

use crate::{phase0::Checkpoint, slot::Slot};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttestationData {
    pub slot: Slot,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub index: u64,
    /// LMD GHOST vote
    pub beacon_block_root: H256,
    /// FFG vote
    pub source: Checkpoint,
    pub target: Checkpoint,
}
