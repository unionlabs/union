use macros::model;

use crate::{
    ethereum::config::consts::{floorlog2, FINALIZED_ROOT_INDEX},
    hash::H256,
    ibc::lightclients::ethereum::{
        light_client_header::UnboundedLightClientHeader, sync_aggregate::UnboundedSyncAggregate,
    },
};
#[cfg(feature = "ssz")]
use crate::{
    ethereum::config::{BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES, SYNC_COMMITTEE_SIZE},
    ibc::lightclients::ethereum::{
        light_client_header::LightClientHeader, sync_aggregate::SyncAggregate,
    },
};

#[cfg(feature = "ssz")]
#[model]
#[derive(::ssz::Ssz)]
#[cfg_attr(feature = "serde", serde(bound(serialize = "", deserialize = "")))]
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
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
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
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub signature_slot: u64,
}
