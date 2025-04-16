use unionlabs::primitives::H256;

use crate::{
    custom_types::{CommitteeIndex, Slot},
    phase0::Checkpoint,
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct AttestationData {
    pub slot: Slot,
    pub index: CommitteeIndex,
    /// LMD GHOST vote
    pub beacon_block_root: H256,
    /// FFG vote
    pub source: Checkpoint,
    pub target: Checkpoint,
}
