use unionlabs::hash::H256;
#[cfg(feature = "ssz")]
use {crate::BYTES_PER_LOGS_BLOOM, crate::MAX_EXTRA_DATA_BYTES, crate::SYNC_COMMITTEE_SIZE};

use crate::{
    consts::{floorlog2, CURRENT_SYNC_COMMITTEE_INDEX},
    light_client_header::LightClientHeader,
    sync_committee::SyncCommittee,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LightClientBootstrap {
    pub header: LightClientHeader,
    /// Current sync committee corresponding to `beacon_header.state_root`
    pub current_sync_committee: SyncCommittee,
    pub current_sync_committee_branch: [H256; floorlog2(CURRENT_SYNC_COMMITTEE_INDEX)],
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, ssz::Ssz)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LightClientBootstrapSsz<
    C: SYNC_COMMITTEE_SIZE + BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES,
> {
    pub header: crate::LightClientHeaderSsz<C>,
    /// Current sync committee corresponding to `beacon_header.state_root`
    pub current_sync_committee: crate::SyncCommitteeSsz<C>,
    pub current_sync_committee_branch: [H256; floorlog2(CURRENT_SYNC_COMMITTEE_INDEX)],
}
