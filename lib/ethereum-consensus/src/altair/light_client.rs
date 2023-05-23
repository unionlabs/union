use crate::altair::{SyncAggregate, SyncCommittee};
use crate::phase0::BeaconBlockHeader;
use crate::primitives::{Bytes32, Version};
use ssz_rs::prelude::*;

pub const NEXT_SYNC_COMMITTEE_INDEX_FLOOR_LOG_2: usize = 5;
pub const FINALIZED_ROOT_INDEX_FLOOR_LOG_2: usize = 6;

#[derive(Default, Debug, Clone, SimpleSerialize)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LightClientUpdate<const SYNC_COMMITTEE_SIZE: usize> {
    pub attested_header: BeaconBlockHeader,
    pub next_sync_committee: SyncCommittee<SYNC_COMMITTEE_SIZE>,
    pub next_sync_committee_branch: Vector<Bytes32, NEXT_SYNC_COMMITTEE_INDEX_FLOOR_LOG_2>,
    pub finalized_header: BeaconBlockHeader,
    pub finality_branch: Vector<Bytes32, FINALIZED_ROOT_INDEX_FLOOR_LOG_2>,
    pub sync_aggregate: SyncAggregate<SYNC_COMMITTEE_SIZE>,
    pub fork_version: Version,
}
