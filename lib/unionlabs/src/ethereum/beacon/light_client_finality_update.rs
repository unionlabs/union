use macros::model;
use ssz::Ssz;

use crate::{
    ethereum::config::{
        consts::{floorlog2, FINALIZED_ROOT_INDEX},
        BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES, SYNC_COMMITTEE_SIZE,
    },
    hash::H256,
    ibc::lightclients::ethereum::{
        light_client_header::{LightClientHeader, UnboundedLightClientHeader},
        sync_aggregate::{SyncAggregate, UnboundedSyncAggregate},
    },
};

#[model]
#[derive(Ssz)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct LightClientFinalityUpdate<
    C: SYNC_COMMITTEE_SIZE + BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES,
> {
    /// Header attested to by the sync committee
    pub attested_header: LightClientHeader<C>,
    /// Finalized header corresponding to `attested_header.state_root`
    pub finalized_header: LightClientHeader<C>,
    pub finality_branch: [H256; floorlog2(FINALIZED_ROOT_INDEX)],
    /// Sync committee aggregate signature
    pub sync_aggregate: SyncAggregate<C>,
    /// Slot at which the aggregate signature was created (untrusted)
    #[serde(with = "::serde_utils::string")]
    pub signature_slot: u64,
}

#[model]
pub struct UnboundedLightClientFinalityUpdate {
    /// Header attested to by the sync committee
    pub attested_header: UnboundedLightClientHeader,
    /// Finalized header corresponding to `attested_header.state_root`
    pub finalized_header: UnboundedLightClientHeader,
    pub finality_branch: [H256; floorlog2(FINALIZED_ROOT_INDEX)],
    /// Sync committee aggregate signature
    pub sync_aggregate: UnboundedSyncAggregate,
    /// Slot at which the aggregate signature was created (untrusted)
    #[serde(with = "::serde_utils::string")]
    pub signature_slot: u64,
}
