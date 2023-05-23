use crate::altair::{
    SyncAggregate, SyncCommittee, FINALIZED_ROOT_INDEX_FLOOR_LOG_2,
    NEXT_SYNC_COMMITTEE_INDEX_FLOOR_LOG_2,
};
use crate::capella::ExecutionPayloadHeader;
use crate::phase0::BeaconBlockHeader;
use crate::primitives::{Bytes32, Slot};
use ssz_rs::prelude::*;

pub const EXECUTION_BRANCH_INDEX_FLOOR_LOG_2: usize = 4;

#[derive(Default, Debug, Clone, SimpleSerialize)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LightClientUpdate<
    const SYNC_COMMITTEE_SIZE: usize,
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
> {
    pub attested_header: LightClientHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    pub next_sync_committee: Option<SyncCommittee<SYNC_COMMITTEE_SIZE>>,
    pub next_sync_committee_branch: Vector<Bytes32, NEXT_SYNC_COMMITTEE_INDEX_FLOOR_LOG_2>,
    pub finalized_header: LightClientHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    pub finality_branch: Vector<Bytes32, FINALIZED_ROOT_INDEX_FLOOR_LOG_2>,
    pub sync_aggregate: SyncAggregate<SYNC_COMMITTEE_SIZE>,
    pub signature_slot: Slot,
}

#[derive(Default, Debug, Clone, SimpleSerialize)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LightClientHeader<const BYTES_PER_LOGS_BLOOM: usize, const MAX_EXTRA_DATA_BYTES: usize> {
    pub beacon: BeaconBlockHeader,
    pub execution: ExecutionPayloadHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    pub execution_branch: Vector<Bytes32, EXECUTION_BRANCH_INDEX_FLOOR_LOG_2>,
}
