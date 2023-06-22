#[derive(Debug, Default)]
pub struct SyncCommittee {
    pub pubkeys: Vec<Vec<u8>>,
    pub aggregate_pubkey: Vec<u8>,
}

impl From<SyncCommittee> for protos::union::ibc::lightclients::ethereum::v1::SyncCommittee {
    fn from(value: SyncCommittee) -> Self {
        Self {
            pubkeys: value.pubkeys,
            aggregate_pubkey: value.aggregate_pubkey,
        }
    }
}
