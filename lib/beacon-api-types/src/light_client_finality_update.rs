use unionlabs::hash::H256;

use crate::{
    consts::{floorlog2, FINALIZED_ROOT_INDEX},
    light_client_header::LightClientHeader,
    SyncAggregate,
};
#[cfg(feature = "ssz")]
use crate::{
    light_client_header::LightClientHeaderSsz, SyncAggregateSsz, BYTES_PER_LOGS_BLOOM,
    MAX_EXTRA_DATA_BYTES, SYNC_COMMITTEE_SIZE,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LightClientFinalityUpdate {
    /// Header attested to by the sync committee
    pub attested_header: LightClientHeader,
    /// Finalized header corresponding to `attested_header.state_root`
    pub finalized_header: LightClientHeader,
    pub finality_branch: [H256; floorlog2(FINALIZED_ROOT_INDEX)],
    /// Sync committee aggregate signature
    pub sync_aggregate: SyncAggregate,
    /// Slot at which the aggregate signature was created (untrusted)
    #[serde(with = "::serde_utils::string")]
    pub signature_slot: u64,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, ssz::Ssz)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LightClientFinalityUpdateSsz<
    C: SYNC_COMMITTEE_SIZE + BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES,
> {
    /// Header attested to by the sync committee
    pub attested_header: LightClientHeaderSsz<C>,
    /// Finalized header corresponding to `attested_header.state_root`
    pub finalized_header: LightClientHeaderSsz<C>,
    pub finality_branch: [H256; floorlog2(FINALIZED_ROOT_INDEX)],
    /// Sync committee aggregate signature
    pub sync_aggregate: SyncAggregateSsz<C>,
    /// Slot at which the aggregate signature was created (untrusted)
    #[serde(with = "::serde_utils::string")]
    pub signature_slot: u64,
}
