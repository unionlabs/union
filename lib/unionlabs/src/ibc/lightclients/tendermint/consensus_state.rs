use macros::model;

use crate::{
    google::protobuf::timestamp::Timestamp, hash::H256,
    ibc::core::commitment::merkle_root::MerkleRoot,
};

#[model(proto(
    raw(protos::ibc::lightclients::tendermint::v1::ConsensusState),
    into,
    from
))]
pub struct ConsensusState {
    pub timestamp: Timestamp,
    pub root: MerkleRoot,
    pub next_validators_hash: H256,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        errors::{required, InvalidLength, MissingField},
        google::protobuf::timestamp::proto::TryFromTimestampError,
        ibc::{
            core::commitment::merkle_root::proto::TryFromMerkleRootError,
            lightclients::tendermint::consensus_state::ConsensusState,
        },
    };

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum TryFromConsensusStateError {
        #[error(transparent)]
        MissingField(MissingField),
        #[error("invalid root")]
        Root(#[from] TryFromMerkleRootError),
        #[error("invalid next validators hash")]
        NextValidatorsHash(#[from] InvalidLength),
        #[error("invalid timestamp")]
        Timestamp(#[from] TryFromTimestampError),
    }

    impl TryFrom<protos::ibc::lightclients::tendermint::v1::ConsensusState> for ConsensusState {
        type Error = TryFromConsensusStateError;

        fn try_from(
            value: protos::ibc::lightclients::tendermint::v1::ConsensusState,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                timestamp: required!(value.timestamp)?
                    .try_into()
                    .map_err(TryFromConsensusStateError::Timestamp)?,
                root: required!(value.root)?
                    .try_into()
                    .map_err(TryFromConsensusStateError::Root)?,
                next_validators_hash: value
                    .next_validators_hash
                    .try_into()
                    .map_err(TryFromConsensusStateError::NextValidatorsHash)?,
            })
        }
    }

    impl From<ConsensusState> for protos::ibc::lightclients::tendermint::v1::ConsensusState {
        fn from(value: ConsensusState) -> Self {
            Self {
                timestamp: Some(value.timestamp.into()),
                root: Some(value.root.into()),
                next_validators_hash: value.next_validators_hash.into(),
            }
        }
    }
}
