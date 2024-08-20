use macros::model;

use crate::{errors::InvalidLength, hash::H256};

#[model(proto(
    raw(protos::union::ibc::lightclients::scroll::v1::ConsensusState),
    into,
    from
))]
pub struct ConsensusState {
    pub state_root: H256,
    pub timestamp: u64,
    pub ibc_storage_root: H256,
}

impl From<ConsensusState> for protos::union::ibc::lightclients::scroll::v1::ConsensusState {
    fn from(value: ConsensusState) -> Self {
        Self {
            state_root: value.state_root.into(),
            timestamp: value.timestamp,
            ibc_storage_root: value.ibc_storage_root.into(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromConsensusStateError {
    #[error("invalid state root")]
    StateRoot(#[source] InvalidLength),
    #[error("invalid ibc storage root")]
    IbcStorageRoot(#[source] InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::scroll::v1::ConsensusState> for ConsensusState {
    type Error = TryFromConsensusStateError;

    fn try_from(
        value: protos::union::ibc::lightclients::scroll::v1::ConsensusState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            state_root: value
                .state_root
                .try_into()
                .map_err(TryFromConsensusStateError::IbcStorageRoot)?,
            timestamp: value.timestamp,
            ibc_storage_root: value
                .ibc_storage_root
                .try_into()
                .map_err(TryFromConsensusStateError::IbcStorageRoot)?,
        })
    }
}
