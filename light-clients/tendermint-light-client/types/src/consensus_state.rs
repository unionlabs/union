use serde::{Deserialize, Serialize};
use unionlabs::{
    google::protobuf::timestamp::Timestamp, hash::H256,
    ibc::core::commitment::merkle_root::MerkleRoot,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsensusState {
    pub timestamp: Timestamp,
    pub root: MerkleRoot,
    pub next_validators_hash: H256,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{
        errors::{InvalidLength, MissingField},
        google::protobuf::timestamp::TryFromTimestampError,
        ibc::core::commitment::merkle_root::TryFromMerkleRootError,
        impl_proto_via_try_from_into, required,
    };

    use crate::ConsensusState;

    impl_proto_via_try_from_into!(ConsensusState => protos::ibc::lightclients::tendermint::v1::ConsensusState);

    impl TryFrom<protos::ibc::lightclients::tendermint::v1::ConsensusState> for ConsensusState {
        type Error = Error;

        fn try_from(
            value: protos::ibc::lightclients::tendermint::v1::ConsensusState,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                timestamp: required!(value.timestamp)?.try_into()?,
                root: required!(value.root)?.try_into()?,
                next_validators_hash: value.next_validators_hash.try_into()?,
            })
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid root")]
        Root(#[from] TryFromMerkleRootError),
        #[error("invalid next validators hash")]
        NextValidatorsHash(#[from] InvalidLength),
        #[error("invalid timestamp")]
        Timestamp(#[from] TryFromTimestampError),
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
