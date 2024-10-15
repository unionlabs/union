use macros::model;

use crate::{google::protobuf::timestamp::Timestamp, hash::H256};

#[model(proto(
    raw(protos::union::ibc::lightclients::berachain::v1::ConsensusState),
    into,
    from
))]
pub struct ConsensusState {
    /// Timestamp of the execution layer.
    pub eth_timestamp: u64,
    /// Timestamp of the cometbft beacon node (consensus layer).
    pub comet_timestamp: Timestamp,

    /// Storage root of the execution layer.
    pub eth_storage_root: H256,
    /// Next validators hash of the cometbft beacon node (consensus layer).
    pub comet_next_validators_hash: H256,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        errors::{required, InvalidLength, MissingField},
        google::protobuf::timestamp::proto::TryFromTimestampError,
        ibc::lightclients::berachain::consensus_state::ConsensusState,
    };

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum TryFromConsensusStateError {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid comet timestamp")]
        CometTimestamp(#[source] TryFromTimestampError),
        #[error("invalid comet next validators hash")]
        CometNextValidatorsHash(#[source] InvalidLength),
        #[error("invalid max clock drift")]
        EthStorageRoot(#[source] InvalidLength),
    }

    impl TryFrom<protos::union::ibc::lightclients::berachain::v1::ConsensusState> for ConsensusState {
        type Error = TryFromConsensusStateError;

        fn try_from(
            value: protos::union::ibc::lightclients::berachain::v1::ConsensusState,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                eth_timestamp: value.eth_timestamp,
                comet_timestamp: required!(value.comet_timestamp)?
                    .try_into()
                    .map_err(TryFromConsensusStateError::CometTimestamp)?,
                eth_storage_root: value
                    .eth_storage_root
                    .try_into()
                    .map_err(TryFromConsensusStateError::EthStorageRoot)?,
                comet_next_validators_hash: value
                    .comet_next_validators_hash
                    .try_into()
                    .map_err(TryFromConsensusStateError::CometNextValidatorsHash)?,
            })
        }
    }

    impl From<ConsensusState> for protos::union::ibc::lightclients::berachain::v1::ConsensusState {
        fn from(value: ConsensusState) -> Self {
            Self {
                eth_timestamp: value.eth_timestamp,
                comet_timestamp: Some(value.comet_timestamp.into()),
                eth_storage_root: value.eth_storage_root.into(),
                comet_next_validators_hash: value.comet_next_validators_hash.into(),
            }
        }
    }
}
