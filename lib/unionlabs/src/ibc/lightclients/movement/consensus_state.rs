use macros::model;

use crate::hash::H256;

#[model(proto(
    raw(protos::union::ibc::lightclients::movement::v1::ConsensusState),
    into,
    from
))]
pub struct ConsensusState {
    pub state_root: H256,
    pub timestamp: u64,
    /// This is the hash of the `StateProof` which is committed to l1
    pub state_proof_hash: H256,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        errors::InvalidLength, ibc::lightclients::movement::consensus_state::ConsensusState,
    };

    impl From<ConsensusState> for protos::union::ibc::lightclients::movement::v1::ConsensusState {
        fn from(value: ConsensusState) -> Self {
            Self {
                state_root: value.state_root.into(),
                timestamp: value.timestamp,
                state_proof_hash: value.state_proof_hash.into(),
            }
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum TryFromConsensusStateError {
        #[error("invalid state root")]
        StateRoot(#[source] InvalidLength),
        #[error("invalid state proof hash")]
        StateProofHash(#[source] InvalidLength),
    }

    impl TryFrom<protos::union::ibc::lightclients::movement::v1::ConsensusState> for ConsensusState {
        type Error = TryFromConsensusStateError;

        fn try_from(
            value: protos::union::ibc::lightclients::movement::v1::ConsensusState,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                state_root: value
                    .state_root
                    .try_into()
                    .map_err(TryFromConsensusStateError::StateRoot)?,
                timestamp: value.timestamp,
                state_proof_hash: value
                    .state_proof_hash
                    .try_into()
                    .map_err(TryFromConsensusStateError::StateProofHash)?,
            })
        }
    }
}
