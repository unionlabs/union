use unionlabs::primitives::H256;

use crate::{
    altair::SyncCommittee,
    consts::{floorlog2, CURRENT_SYNC_COMMITTEE_GINDEX_ELECTRA},
    deneb::LightClientHeader,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct LightClientBootstrap {
    pub header: LightClientHeader,
    /// Current sync committee corresponding to `beacon_header.state_root`
    pub current_sync_committee: SyncCommittee,
    pub current_sync_committee_branch: [H256; floorlog2(CURRENT_SYNC_COMMITTEE_GINDEX_ELECTRA)],
}
