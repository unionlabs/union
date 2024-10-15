use macros::model;

use crate::{hash::H256, ibc::core::commitment::merkle_root::MerkleRoot};

#[model(
    proto(
        raw(protos::union::ibc::lightclients::cometbls::v1::ConsensusState),
        into,
        from
    ),
    ethabi(raw(ibc_solidity::cometbls::ConsensusState), into, from)
)]
pub struct ConsensusState {
    pub timestamp: u64,
    pub app_hash: MerkleRoot,
    pub next_validators_hash: H256,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        errors::{required, InvalidLength, MissingField},
        ibc::{
            core::commitment::merkle_root::proto::TryFromMerkleRootError,
            lightclients::cometbls::consensus_state::ConsensusState,
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
    }

    impl TryFrom<protos::union::ibc::lightclients::cometbls::v1::ConsensusState> for ConsensusState {
        type Error = TryFromConsensusStateError;

        fn try_from(
            value: protos::union::ibc::lightclients::cometbls::v1::ConsensusState,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                timestamp: value.timestamp,
                app_hash: required!(value.root)?
                    .try_into()
                    .map_err(TryFromConsensusStateError::Root)?,
                next_validators_hash: value
                    .next_validators_hash
                    .try_into()
                    .map_err(TryFromConsensusStateError::NextValidatorsHash)?,
            })
        }
    }

    impl From<ConsensusState> for protos::union::ibc::lightclients::cometbls::v1::ConsensusState {
        fn from(value: ConsensusState) -> Self {
            Self {
                timestamp: value.timestamp,
                root: Some(value.app_hash.into()),
                next_validators_hash: value.next_validators_hash.into(),
            }
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<ConsensusState> for ibc_solidity::cometbls::ConsensusState {
    fn from(value: ConsensusState) -> Self {
        Self {
            timestamp: value.timestamp,
            appHash: value.app_hash.hash.into(),
            nextValidatorsHash: value.next_validators_hash.into(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl TryFrom<ibc_solidity::cometbls::ConsensusState> for ConsensusState {
    type Error = proto::TryFromConsensusStateError;

    fn try_from(value: ibc_solidity::cometbls::ConsensusState) -> Result<Self, Self::Error> {
        Ok(Self {
            timestamp: value.timestamp,
            app_hash: MerkleRoot {
                hash: value.appHash.into(),
            },
            next_validators_hash: value.nextValidatorsHash.into(),
        })
    }
}
