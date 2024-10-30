use unionlabs::hash::H256;

use crate::{
    consts::{floorlog2, FINALIZED_ROOT_INDEX, NEXT_SYNC_COMMITTEE_INDEX},
    LightClientHeader, SyncAggregate, SyncCommittee,
};

pub type NextSyncCommitteeBranch = [H256; floorlog2(NEXT_SYNC_COMMITTEE_INDEX)];
pub type FinalityBranch = [H256; floorlog2(FINALIZED_ROOT_INDEX)];

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LightClientUpdate {
    /// Header attested to by the sync committee
    pub attested_header: LightClientHeader,
    pub next_sync_committee: Option<SyncCommittee>,
    pub next_sync_committee_branch: Option<NextSyncCommitteeBranch>,
    /// Finalized header corresponding to `attested_header.state_root`
    pub finalized_header: LightClientHeader,
    pub finality_branch: FinalityBranch,
    /// Sync committee aggregate signature
    pub sync_aggregate: SyncAggregate,
    /// Slot at which the aggregate signature was created (untrusted)
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub signature_slot: u64,
}
