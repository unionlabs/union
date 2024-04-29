use macros::model;

use crate::{bls::BlsPublicKey, errors::InvalidLength, hash::H256};

#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::ConsensusState),
    into,
    from
))]
pub struct ConsensusState {
    // REVIEW: Remove this field as this height is what is used to query the consensus state?
    pub slot: u64,
    pub state_root: H256,
    pub storage_root: H256,
    pub timestamp: u64,
    /// aggregate public key of current sync committee
    pub current_sync_committee: BlsPublicKey,
    /// aggregate public key of next sync committee
    pub next_sync_committee: Option<BlsPublicKey>,
}

impl From<ConsensusState> for protos::union::ibc::lightclients::ethereum::v1::ConsensusState {
    fn from(value: ConsensusState) -> Self {
        Self {
            slot: value.slot,
            state_root: value.state_root.into(),
            storage_root: value.storage_root.into(),
            timestamp: value.timestamp,
            current_sync_committee: value.current_sync_committee.into(),
            next_sync_committee: value
                .next_sync_committee
                .map(Into::into)
                .unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TryFromConsensusStateError {
    CurrentSyncCommittee(InvalidLength),
    NextSyncCommittee(InvalidLength),
    StorageRoot(InvalidLength),
    StateRoot(InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::ConsensusState> for ConsensusState {
    type Error = TryFromConsensusStateError;

    fn try_from(
        value: protos::union::ibc::lightclients::ethereum::v1::ConsensusState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            slot: value.slot,
            state_root: value
                .state_root
                .try_into()
                .map_err(TryFromConsensusStateError::StorageRoot)?,
            storage_root: value
                .storage_root
                .try_into()
                .map_err(TryFromConsensusStateError::StorageRoot)?,
            timestamp: value.timestamp,
            current_sync_committee: value
                .current_sync_committee
                .try_into()
                .map_err(TryFromConsensusStateError::CurrentSyncCommittee)?,
            next_sync_committee: if value.next_sync_committee.is_empty() {
                None
            } else {
                Some(
                    value
                        .next_sync_committee
                        .try_into()
                        .map_err(TryFromConsensusStateError::NextSyncCommittee)?,
                )
            },
        })
    }
}
