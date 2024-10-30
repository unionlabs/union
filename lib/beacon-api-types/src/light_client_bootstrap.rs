use unionlabs::hash::H256;

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
