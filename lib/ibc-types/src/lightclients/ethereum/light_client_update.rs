use crate::lightclients::ethereum::{
    light_client_header::LightClientHeader, sync_aggregate::SyncAggregate,
    sync_committee::SyncCommittee,
};

#[derive(Debug)]
pub struct LightClientUpdate {
    pub attested_header: LightClientHeader,
    pub next_sync_committee: SyncCommittee,
    // TODO(benluelo): vec<bytes32>
    pub next_sync_committee_branch: Vec<Vec<u8>>,
    pub finalized_header: LightClientHeader,
    pub finality_branch: Vec<Vec<u8>>,
    pub sync_aggregate: SyncAggregate,
    pub signature_slot: u64,
}

impl From<LightClientUpdate> for protos::union::ibc::lightclients::ethereum::v1::LightClientUpdate {
    fn from(value: LightClientUpdate) -> Self {
        Self {
            attested_header: Some(value.attested_header.into()),
            next_sync_committee: Some(value.next_sync_committee.into()),
            next_sync_committee_branch: value.next_sync_committee_branch,
            finalized_header: Some(value.finalized_header.into()),
            finality_branch: value.finality_branch,
            sync_aggregate: Some(value.sync_aggregate.into()),
            signature_slot: value.signature_slot,
        }
    }
}
