use unionlabs::{
    google::protobuf::timestamp::Timestamp,
    ibc::core::commitment::merkle_root::MerkleRoot,
    primitives::{encoding::HexUnprefixed, H256},
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ConsensusState {
    pub timestamp: Timestamp,
    pub root: MerkleRoot,
    pub next_validators_hash: H256<HexUnprefixed>,
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
            bytes32 root;
            bytes32 nextValidatorsHash;
        }
    }

    impl From<ConsensusState> for SolConsensusState {
        fn from(value: ConsensusState) -> Self {
            Self {
                timestamp: value.timestamp.as_unix_nanos(),
                root: value.root.hash.get().into(),
                nextValidatorsHash: value.next_validators_hash.get().into(),
            }
        }
    }

    impl From<SolConsensusState> for ConsensusState {
        fn from(value: SolConsensusState) -> Self {
            Self {
                timestamp: Timestamp::try_from_unix_nanos(value.timestamp.into())
                    .expect("impossible"),
                root: H256::new(value.root.0).into(),
                next_validators_hash: H256::new(value.nextValidatorsHash.0),
            }
        }
    }
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{
        errors::MissingField, google::protobuf::timestamp::TryFromTimestampError,
        ibc::core::commitment::merkle_root::TryFromMerkleRootError, impl_proto_via_try_from_into,
        primitives::FixedBytesError, required,
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
        NextValidatorsHash(#[from] FixedBytesError),
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

#[cfg(test)]
mod tests {
    use unionlabs::{
        encoding::{Bincode, EthAbi, Json, Proto},
        test_utils::assert_codec_iso,
    };

    use super::*;

    fn mk_consensus_state() -> ConsensusState {
        ConsensusState {
            timestamp: Timestamp {
                seconds: 123.try_into().unwrap(),
                nanos: 456.try_into().unwrap(),
            },
            root: MerkleRoot {
                hash: H256::new([0xAA; 32]),
            },
            next_validators_hash: H256::new([0xAA; 32]),
        }
    }

    #[test]
    fn ethabi_iso() {
        assert_codec_iso::<_, EthAbi>(&mk_consensus_state());
    }

    #[test]
    fn bincode_iso() {
        assert_codec_iso::<_, Bincode>(&mk_consensus_state());
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
