use ethereum_consensus::{
    beacon::{BeaconBlockHeader, Checkpoint, Root, Slot},
    bls::Signature,
    capella::{LightClientBootstrap, LightClientHeader, LightClientUpdate},
    sync_protocol::FINALIZED_ROOT_DEPTH,
    sync_protocol::{
        SyncAggregate, SyncCommittee, CURRENT_SYNC_COMMITTEE_DEPTH, NEXT_SYNC_COMMITTEE_DEPTH,
    },
    types::{H256, U64},
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenesisDataResponse {
    pub data: GenesisData,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenesisData {
    pub genesis_validators_root: Root,
    pub genesis_time: U64,
    pub genesis_fork_version: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BeaconBlockRootResponse {
    pub data: BeaconBlockRoot,
    pub execution_optimistic: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BeaconBlockRoot {
    pub root: Root,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BeaconHeaderResponse {
    pub data: BeaconHeaderData,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BeaconHeaderData {
    pub root: Root,
    pub canonical: bool,
    pub header: BeaconHeaderSignature,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BeaconHeaderSignature {
    pub message: BeaconBlockHeader,
    pub signature: Signature,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FinalityCheckpointsResponse {
    pub data: FinalityCheckpoints,
    pub execution_optimistic: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FinalityCheckpoints {
    pub previous_justified: Checkpoint,
    pub current_justified: Checkpoint,
    pub finalized: Checkpoint,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LightClientFinalityUpdateResponse<
    const SYNC_COMMITTEE_SIZE: usize,
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
> {
    pub data: LightClientFinalityUpdateData<
        SYNC_COMMITTEE_SIZE,
        BYTES_PER_LOGS_BLOOM,
        MAX_EXTRA_DATA_BYTES,
    >,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]

pub struct LightClientFinalityUpdateData<
    const SYNC_COMMITTEE_SIZE: usize,
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
> {
    /// Header attested to by the sync committee
    pub attested_header: LightClientHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    /// Finalized header corresponding to `attested_header.state_root`
    pub finalized_header: LightClientHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    pub finality_branch: [H256; FINALIZED_ROOT_DEPTH],
    /// Sync committee aggregate signature
    pub sync_aggregate: SyncAggregate<SYNC_COMMITTEE_SIZE>,
    /// Slot at which the aggregate signature was created (untrusted)
    pub signature_slot: Slot,
}

impl<
        const SYNC_COMMITTEE_SIZE: usize,
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
    >
    From<
        LightClientFinalityUpdateData<
            SYNC_COMMITTEE_SIZE,
            BYTES_PER_LOGS_BLOOM,
            MAX_EXTRA_DATA_BYTES,
        >,
    > for LightClientUpdate<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>
{
    fn from(
        value: LightClientFinalityUpdateData<
            SYNC_COMMITTEE_SIZE,
            BYTES_PER_LOGS_BLOOM,
            MAX_EXTRA_DATA_BYTES,
        >,
    ) -> Self {
        Self {
            attested_header: value.attested_header,
            next_sync_committee: None,
            finalized_header: value.finalized_header,
            finality_branch: value.finality_branch,
            sync_aggregate: value.sync_aggregate,
            signature_slot: value.signature_slot,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LightClientBootstrapResponse<
    const SYNC_COMMITTEE_SIZE: usize,
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
> {
    pub data:
        LightClientBootstrapData<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LightClientBootstrapData<
    const SYNC_COMMITTEE_SIZE: usize,
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
> {
    pub header: LightClientHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    pub current_sync_committee: SyncCommittee<SYNC_COMMITTEE_SIZE>,
    pub current_sync_committee_branch: [H256; CURRENT_SYNC_COMMITTEE_DEPTH],
}

impl<
        const SYNC_COMMITTEE_SIZE: usize,
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
    >
    From<LightClientBootstrapData<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>>
    for LightClientBootstrap<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>
{
    fn from(
        value: LightClientBootstrapData<
            SYNC_COMMITTEE_SIZE,
            BYTES_PER_LOGS_BLOOM,
            MAX_EXTRA_DATA_BYTES,
        >,
    ) -> Self {
        Self {
            header: value.header,
            current_sync_committee: value.current_sync_committee,
            current_sync_committee_branch: value.current_sync_committee_branch,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LightClientUpdatesResponse<
    const SYNC_COMMITTEE_SIZE: usize,
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
>(
    pub  Vec<
        LightClientUpdateResponse<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    >,
);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LightClientUpdateResponse<
    const SYNC_COMMITTEE_SIZE: usize,
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
> {
    pub version: String,
    pub data:
        LightClientUpdateData<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LightClientUpdateData<
    const SYNC_COMMITTEE_SIZE: usize,
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
> {
    pub attested_header: LightClientHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    pub next_sync_committee: SyncCommittee<SYNC_COMMITTEE_SIZE>,
    pub next_sync_committee_branch: [H256; NEXT_SYNC_COMMITTEE_DEPTH],
    pub finalized_header: LightClientHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    pub finality_branch: [H256; FINALIZED_ROOT_DEPTH],
    pub sync_aggregate: SyncAggregate<SYNC_COMMITTEE_SIZE>,
    pub signature_slot: Slot,
}

impl<
        const SYNC_COMMITTEE_SIZE: usize,
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
    > From<LightClientUpdateData<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>>
    for LightClientUpdate<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>
{
    fn from(
        value: LightClientUpdateData<
            SYNC_COMMITTEE_SIZE,
            BYTES_PER_LOGS_BLOOM,
            MAX_EXTRA_DATA_BYTES,
        >,
    ) -> Self {
        let next_sync_committee = if value.next_sync_committee == Default::default() {
            None
        } else {
            Some((value.next_sync_committee, value.next_sync_committee_branch))
        };
        Self {
            attested_header: value.attested_header,
            next_sync_committee,
            finalized_header: value.finalized_header,
            finality_branch: value.finality_branch,
            sync_aggregate: value.sync_aggregate,
            signature_slot: value.signature_slot,
        }
    }
}
