use consensus_primitives::Timestamp;
use unionlabs::{
    ibc::core::commitment::merkle_root::MerkleRoot,
    primitives::{encoding::HexUnprefixed, H256},
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub struct ConsensusState {
    pub timestamp: Timestamp,
    pub app_hash: MerkleRoot,
    pub next_validators_hash: H256<HexUnprefixed>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use consensus_primitives::Timestamp;
    use unionlabs::{
        errors::{required, MissingField},
        ibc::core::commitment::merkle_root::TryFromMerkleRootError,
        impl_proto_via_try_from_into,
        primitives::FixedBytesError,
    };

    use crate::consensus_state::ConsensusState;

    impl_proto_via_try_from_into!(ConsensusState => protos::union::ibc::lightclients::cometbls::v1::ConsensusState);

    impl TryFrom<protos::union::ibc::lightclients::cometbls::v1::ConsensusState> for ConsensusState {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::cometbls::v1::ConsensusState,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                timestamp: Timestamp::from_nanos(value.timestamp),
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
        NextValidatorsHash(#[from] FixedBytesError),
    }

    impl From<ConsensusState> for protos::union::ibc::lightclients::cometbls::v1::ConsensusState {
        fn from(value: ConsensusState) -> Self {
            Self {
                timestamp: value.timestamp.as_nanos(),
                root: Some(value.app_hash.into()),
                next_validators_hash: value.next_validators_hash.into(),
            }
        }
    }
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use alloy::sol_types::SolValue;
    use unionlabs::impl_ethabi_via_try_from_into;

    use super::*;

    impl_ethabi_via_try_from_into!(ConsensusState => SolConsensusState);

    alloy::sol! {
        struct SolConsensusState {
            uint64 timestamp;
            bytes32 app_hash;
            bytes32 next_validators_hash;
        }
    }

    impl From<ConsensusState> for SolConsensusState {
        fn from(value: ConsensusState) -> Self {
            Self {
                timestamp: value.timestamp.as_nanos(),
                app_hash: value.app_hash.hash.get().into(),
                next_validators_hash: value.next_validators_hash.get().into(),
            }
        }
    }

    impl From<SolConsensusState> for ConsensusState {
        fn from(value: SolConsensusState) -> Self {
            Self {
                timestamp: Timestamp::from_nanos(value.timestamp),
                app_hash: MerkleRoot {
                    hash: H256::new(value.app_hash.0),
                },
                next_validators_hash: H256::new(value.next_validators_hash.0),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use unionlabs::{
        encoding::{EthAbi, Json, Proto},
        test_utils::assert_codec_iso,
    };

    use super::*;

    fn mk_consensus_state() -> ConsensusState {
        ConsensusState {
            timestamp: Timestamp::from_nanos(123_456_789),
            app_hash: MerkleRoot {
                hash: H256::from([0xAA; 32]),
            },
            next_validators_hash: H256::from([0xAA; 32]),
        }
    }

    #[test]
    fn ethabi_iso() {
        assert_codec_iso::<_, EthAbi>(&mk_consensus_state());
    }

    #[test]
    fn json_iso() {
        assert_codec_iso::<_, Json>(&mk_consensus_state());
    }

    #[test]
    fn proto_iso() {
        assert_codec_iso::<_, Proto>(&mk_consensus_state());
    }
}
