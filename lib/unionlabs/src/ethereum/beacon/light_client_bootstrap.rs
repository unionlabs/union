use macros::model;
use ssz::{types::Vector, Ssz};
use typenum::U;

use crate::{
    ethereum::config::{
        consts::{floorlog2, CURRENT_SYNC_COMMITTEE_INDEX},
        BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES, SYNC_COMMITTEE_SIZE,
    },
    hash::H256,
    ibc::lightclients::ethereum::{
        light_client_header::{LightClientHeader, UnboundedLightClientHeader},
        sync_committee::{SyncCommittee, UnboundedSyncCommittee},
    },
};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#lightclientbootstrap>
#[model]
#[derive(Ssz)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct LightClientBootstrap<
    C: SYNC_COMMITTEE_SIZE + BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES,
> {
    pub header: LightClientHeader<C>,
    /// Current sync committee corresponding to `beacon_header.state_root`
    pub current_sync_committee: SyncCommittee<C>,
    // TODO: Update tree_hash to support const generic arrays
    pub current_sync_committee_branch: Vector<H256, U<{ floorlog2(CURRENT_SYNC_COMMITTEE_INDEX) }>>,
}

#[model]
pub struct UnboundedLightClientBootstrap {
    pub header: UnboundedLightClientHeader,
    /// Current sync committee corresponding to `beacon_header.state_root`
    pub current_sync_committee: UnboundedSyncCommittee,
    // TODO: Update tree_hash to support const generic arrays
    pub current_sync_committee_branch: Vector<H256, U<{ floorlog2(CURRENT_SYNC_COMMITTEE_INDEX) }>>,
}
