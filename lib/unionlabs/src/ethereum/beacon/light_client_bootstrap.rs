use macros::model;

use crate::{
    ethereum::config::consts::{floorlog2, CURRENT_SYNC_COMMITTEE_INDEX},
    hash::H256,
    ibc::lightclients::ethereum::{
        light_client_header::UnboundedLightClientHeader, sync_committee::UnboundedSyncCommittee,
    },
};
#[cfg(feature = "ssz")]
use crate::{
    ethereum::config::{BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES, SYNC_COMMITTEE_SIZE},
    ibc::lightclients::ethereum::{
        light_client_header::LightClientHeader, sync_committee::SyncCommittee,
    },
};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#lightclientbootstrap>
#[cfg(feature = "ssz")]
#[model]
#[derive(::ssz::Ssz)]
#[cfg_attr(feature = "serde", serde(bound(serialize = "", deserialize = "")))]
pub struct LightClientBootstrap<
    C: SYNC_COMMITTEE_SIZE + BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES,
> {
    pub header: LightClientHeader<C>,
    /// Current sync committee corresponding to `beacon_header.state_root`
    pub current_sync_committee: SyncCommittee<C>,
    pub current_sync_committee_branch: [H256; floorlog2(CURRENT_SYNC_COMMITTEE_INDEX)],
}

#[model]
pub struct UnboundedLightClientBootstrap {
    pub header: UnboundedLightClientHeader,
    /// Current sync committee corresponding to `beacon_header.state_root`
    pub current_sync_committee: UnboundedSyncCommittee,
    pub current_sync_committee_branch: [H256; floorlog2(CURRENT_SYNC_COMMITTEE_INDEX)],
}
