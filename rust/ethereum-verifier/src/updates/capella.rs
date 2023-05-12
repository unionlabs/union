pub use super::bellatrix::ExecutionUpdateInfo;
use super::{ConsensusUpdate, LightClientBootstrap};
use core::ops::Deref;
use ethereum_consensus::{
    beacon::{BeaconBlockHeader, Slot},
    capella::LightClientUpdate,
    compute::hash_tree_root,
    sync_protocol::{
        SyncAggregate, SyncCommittee, CURRENT_SYNC_COMMITTEE_DEPTH, EXECUTION_PAYLOAD_DEPTH,
        FINALIZED_ROOT_DEPTH, NEXT_SYNC_COMMITTEE_DEPTH,
    },
    types::H256,
};

#[derive(Clone, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct LightClientBootstrapInfo<
    const SYNC_COMMITTEE_SIZE: usize,
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
>(
    pub  ethereum_consensus::capella::LightClientBootstrap<
        SYNC_COMMITTEE_SIZE,
        BYTES_PER_LOGS_BLOOM,
        MAX_EXTRA_DATA_BYTES,
    >,
);

impl<
        const SYNC_COMMITTEE_SIZE: usize,
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
    > Deref
    for LightClientBootstrapInfo<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>
{
    type Target = ethereum_consensus::capella::LightClientBootstrap<
        SYNC_COMMITTEE_SIZE,
        BYTES_PER_LOGS_BLOOM,
        MAX_EXTRA_DATA_BYTES,
    >;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<
        const SYNC_COMMITTEE_SIZE: usize,
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
    > LightClientBootstrap<SYNC_COMMITTEE_SIZE>
    for LightClientBootstrapInfo<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>
{
    fn beacon_header(&self) -> &BeaconBlockHeader {
        &self.0.header.beacon
    }
    fn current_sync_committee(&self) -> &SyncCommittee<SYNC_COMMITTEE_SIZE> {
        &self.0.current_sync_committee
    }
    fn current_sync_committee_branch(&self) -> [H256; CURRENT_SYNC_COMMITTEE_DEPTH] {
        self.0.current_sync_committee_branch.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct ConsensusUpdateInfo<
    const SYNC_COMMITTEE_SIZE: usize,
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
>(pub LightClientUpdate<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>);

impl<
        const SYNC_COMMITTEE_SIZE: usize,
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
    > Deref
    for ConsensusUpdateInfo<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>
{
    type Target =
        LightClientUpdate<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<
        const SYNC_COMMITTEE_SIZE: usize,
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
    > ConsensusUpdate<SYNC_COMMITTEE_SIZE>
    for ConsensusUpdateInfo<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>
{
    fn attested_beacon_header(&self) -> &BeaconBlockHeader {
        &self.attested_header.beacon
    }
    fn next_sync_committee(&self) -> Option<&SyncCommittee<SYNC_COMMITTEE_SIZE>> {
        self.next_sync_committee.as_ref().map(|c| &c.0)
    }
    fn next_sync_committee_branch(&self) -> Option<[H256; NEXT_SYNC_COMMITTEE_DEPTH]> {
        self.next_sync_committee.as_ref().map(|c| c.1.clone())
    }
    fn finalized_beacon_header(&self) -> &BeaconBlockHeader {
        &self.finalized_header.beacon
    }
    fn finalized_beacon_header_branch(&self) -> [H256; FINALIZED_ROOT_DEPTH] {
        self.finality_branch.clone()
    }
    fn finalized_execution_root(&self) -> H256 {
        hash_tree_root(self.finalized_header.execution.clone())
            .unwrap()
            .0
            .into()
    }
    fn finalized_execution_branch(&self) -> [H256; EXECUTION_PAYLOAD_DEPTH] {
        self.finalized_header.execution_branch.clone()
    }
    fn sync_aggregate(&self) -> &SyncAggregate<SYNC_COMMITTEE_SIZE> {
        &self.sync_aggregate
    }
    fn signature_slot(&self) -> Slot {
        self.signature_slot
    }
}
