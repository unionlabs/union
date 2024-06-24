use borsh::{BorshDeserialize, BorshSerialize};
use macros::model;
use near_primitives_core::hash::CryptoHash;

use super::block_header_inner::{BlockHeaderInnerLiteView, TryFromBlockHeaderInnerLiteViewError};
use crate::errors::{required, MissingField};

#[model(proto(
    raw(protos::union::ibc::lightclients::near::v1::ConsensusState),
    into,
    from
))]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct ConsensusState {
    pub state: BlockHeaderInnerLiteView,
    pub chunk_prev_state_root: CryptoHash,
    pub timestamp: u64,
}

impl From<ConsensusState> for protos::union::ibc::lightclients::near::v1::ConsensusState {
    fn from(value: ConsensusState) -> Self {
        Self {
            state: Some(value.state.into()),
            chunk_prev_state_root: value.chunk_prev_state_root.into(),
        }
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum TryFromConsensusStateError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error(transparent)]
    State(#[from] TryFromBlockHeaderInnerLiteViewError),
    #[error("invalid chunk prev state root")]
    ChunkPrevStateRoot,
}

impl TryFrom<protos::union::ibc::lightclients::near::v1::ConsensusState> for ConsensusState {
    type Error = TryFromConsensusStateError;

    fn try_from(
        value: protos::union::ibc::lightclients::near::v1::ConsensusState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            state: required!(value.state)?
                .try_into()
                .map_err(TryFromConsensusStateError::State)?,
            chunk_prev_state_root: value
                .chunk_prev_state_root
                .as_slice()
                .try_into()
                .map_err(|_| TryFromConsensusStateError::ChunkPrevStateRoot)?,
            timestamp: 0,
        })
    }
}
