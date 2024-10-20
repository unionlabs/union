use serde::{Deserialize, Serialize};
use unionlabs::{hash::H256, ibc::core::commitment::merkle_root::MerkleRoot};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusState {
    pub timestamp: u64,
    pub app_hash: MerkleRoot,
    pub next_validators_hash: H256,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{
        errors::{required, InvalidLength, MissingField},
        ibc::core::commitment::merkle_root::TryFromMerkleRootError,
        impl_proto_via_try_from_into,
    };

    use crate::consensus_state::ConsensusState;

    impl_proto_via_try_from_into!(ConsensusState => protos::union::ibc::lightclients::cometbls::v1::ConsensusState);

    impl TryFrom<protos::union::ibc::lightclients::cometbls::v1::ConsensusState> for ConsensusState {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::cometbls::v1::ConsensusState,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                timestamp: value.timestamp,
                app_hash: required!(value.root)?.try_into().map_err(Error::Root)?,
                next_validators_hash: value
                    .next_validators_hash
                    .try_into()
                    .map_err(Error::NextValidatorsHash)?,
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
pub mod ethabi {
    use std::string::FromUtf8Error;

    use alloy::sol_types::SolValue;
    use unionlabs::{
        encoding::{Decode, Encode, EthAbi},
        ibc::core::commitment::merkle_root::MerkleRoot,
        TryFromEthAbiBytesErrorAlloy,
    };

    use crate::ConsensusState;

    alloy::sol! {
        struct SolConsensusState {
            uint64 timestamp;
            bytes32 appHash;
            bytes32 nextValidatorsHash;
        }
    }

    impl Encode<EthAbi> for ConsensusState {
        fn encode(self) -> Vec<u8> {
            SolConsensusState {
                timestamp: self.timestamp,
                appHash: self.app_hash.hash.into(),
                nextValidatorsHash: self.next_validators_hash.into(),
            }
            .abi_encode()
        }
    }

    impl Decode<EthAbi> for ConsensusState {
        type Error = TryFromEthAbiBytesErrorAlloy<Error>;

        fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
            let consensus_state = SolConsensusState::abi_decode(bytes, true)?;

            Ok(Self {
                timestamp: consensus_state.timestamp,
                app_hash: MerkleRoot {
                    hash: consensus_state.appHash.into(),
                },
                next_validators_hash: consensus_state.nextValidatorsHash.into(),
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid chain_id")]
        ChainId(#[from] FromUtf8Error),
    }
}
