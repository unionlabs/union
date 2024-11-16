use unionlabs::hash::H256;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConsensusState {
    pub state_root: H256,
    pub timestamp: u64,
    /// This is the hash of the `StateProof` which is committed to l1
    pub state_proof_hash: H256,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{errors::InvalidLength, impl_proto_via_try_from_into};

    use crate::ConsensusState;

    impl_proto_via_try_from_into!(ConsensusState => protos::union::ibc::lightclients::movement::v1::ConsensusState);

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
    pub enum Error {
        #[error("invalid state root")]
        StateRoot(#[source] InvalidLength),
        #[error("invalid state proof hash")]
        StateProofHash(#[source] InvalidLength),
    }

    impl TryFrom<protos::union::ibc::lightclients::movement::v1::ConsensusState> for ConsensusState {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::movement::v1::ConsensusState,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                state_root: value.state_root.try_into().map_err(Error::StateRoot)?,
                timestamp: value.timestamp,
                state_proof_hash: value
                    .state_proof_hash
                    .try_into()
                    .map_err(Error::StateProofHash)?,
            })
        }
    }
}
