use unionlabs::primitives::H256;

use crate::{
    altair::{LightClientHeader, SyncAggregate},
    consts::{floorlog2, FINALIZED_ROOT_GINDEX},
    slot::Slot,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LightClientFinalityUpdate {
    /// Header attested to by the sync committee
    pub attested_header: LightClientHeader,
    /// Finalized header corresponding to `attested_header.state_root`
    pub finalized_header: LightClientHeader,
    pub finality_branch: [H256; floorlog2(FINALIZED_ROOT_GINDEX)],
    /// Sync committee aggregate signature
    pub sync_aggregate: SyncAggregate,
    /// Slot at which the aggregate signature was created (untrusted)
    pub signature_slot: Slot,
}
