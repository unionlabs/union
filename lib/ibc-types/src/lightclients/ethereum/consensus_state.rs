use crate::{errors::MissingField, IntoProto, TryFromProto, TypeUrl};

#[derive(Debug, Clone)]
pub struct ConsensusState {
    pub slot: u64,
    pub storage_root: Vec<u8>,
    pub timestamp: u64,
    pub current_sync_committee: Vec<u8>,
    pub next_sync_committee: Vec<u8>,
}

impl From<ConsensusState> for protos::union::ibc::lightclients::ethereum::v1::ConsensusState {
    fn from(value: ConsensusState) -> Self {
        Self {
            slot: value.slot,
            storage_root: value.storage_root,
            timestamp: value.timestamp,
            current_sync_committee: value.current_sync_committee,
            next_sync_committee: value.next_sync_committee,
        }
    }
}

impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::ConsensusState> for ConsensusState {
    type Error = MissingField;

    fn try_from(
        value: protos::union::ibc::lightclients::ethereum::v1::ConsensusState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            slot: value.slot,
            storage_root: value.storage_root,
            timestamp: value.timestamp,
            current_sync_committee: value.current_sync_committee,
            next_sync_committee: value.next_sync_committee,
        })
    }
}

impl IntoProto for ConsensusState {
    type Proto = protos::union::ibc::lightclients::ethereum::v1::ConsensusState;
}

impl TypeUrl for protos::union::ibc::lightclients::ethereum::v1::ConsensusState {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.ethereum.v1.ConsensusState";
}

impl TryFromProto for ConsensusState {
    type Proto = protos::union::ibc::lightclients::ethereum::v1::ConsensusState;
}
