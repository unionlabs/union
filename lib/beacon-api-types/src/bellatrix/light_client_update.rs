use unionlabs::primitives::H256;

use crate::{
    altair::{SyncAggregate, SyncCommittee},
    bellatrix::LightClientHeader,
    consts::{floorlog2, FINALIZED_ROOT_GINDEX, NEXT_SYNC_COMMITTEE_GINDEX},
    slot::Slot,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LightClientUpdate {
    /// Header attested to by the sync committee
    pub attested_header: LightClientHeader,
    pub next_sync_committee: SyncCommittee,
    pub next_sync_committee_branch: [H256; floorlog2(NEXT_SYNC_COMMITTEE_GINDEX)],
    /// Finalized header corresponding to `attested_header.state_root`
    pub finalized_header: LightClientHeader,
    pub finality_branch: [H256; floorlog2(FINALIZED_ROOT_GINDEX)],
    /// Sync committee aggregate signature
    pub sync_aggregate: SyncAggregate,
    /// Slot at which the aggregate signature was created (untrusted)
    pub signature_slot: Slot,
}
