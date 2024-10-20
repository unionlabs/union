use serde::{Deserialize, Serialize};
use unionlabs::{google::protobuf::timestamp::Timestamp, hash::H256};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    use unionlabs::{
        errors::{InvalidLength, MissingField},
        google::protobuf::timestamp::TryFromTimestampError,
        impl_proto_via_try_from_into, required,
    };

    use crate::ConsensusState;

    impl_proto_via_try_from_into!(ConsensusState => protos::union::ibc::lightclients::berachain::v1::ConsensusState);

    impl TryFrom<protos::union::ibc::lightclients::berachain::v1::ConsensusState> for ConsensusState {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::berachain::v1::ConsensusState,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                eth_timestamp: value.eth_timestamp,
                comet_timestamp: required!(value.comet_timestamp)?
                    .try_into()
                    .map_err(Error::CometTimestamp)?,
                eth_storage_root: value
                    .eth_storage_root
                    .try_into()
                    .map_err(Error::EthStorageRoot)?,
                comet_next_validators_hash: value
                    .comet_next_validators_hash
                    .try_into()
                    .map_err(Error::CometNextValidatorsHash)?,
            })
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid comet timestamp")]
        CometTimestamp(#[source] TryFromTimestampError),
        #[error("invalid comet next validators hash")]
        CometNextValidatorsHash(#[source] InvalidLength),
        #[error("invalid max clock drift")]
        EthStorageRoot(#[source] InvalidLength),
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
