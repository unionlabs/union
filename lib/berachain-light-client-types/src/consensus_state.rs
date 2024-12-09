use unionlabs::hash::H256;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConsensusState {
    pub comet: tendermint_light_client_types::ConsensusState,
    /// Timestamp of the execution layer.
    pub timestamp: u64,
    /// Storage root of the execution layer.
    pub storage_root: H256,
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use alloy::sol_types::SolValue;
    use unionlabs::{google::protobuf::timestamp::Timestamp, impl_ethabi_via_try_from_into};

    use super::*;

    impl_ethabi_via_try_from_into!(ConsensusState => SolConsensusState);

    // TODO: if we ever support importing, deduplicate the comet consensus state here.
    alloy::sol! {
        struct SolCometConsensusState {
            uint64 timestamp;
            bytes32 root;
            bytes32 nextValidatorsHash;
        }

        struct SolConsensusState {
            SolCometConsensusState comet;
            uint64 timestamp;
            bytes32 storageRoot;
        }
    }

    impl From<ConsensusState> for SolConsensusState {
        fn from(value: ConsensusState) -> Self {
            Self {
                comet: SolCometConsensusState {
                    timestamp: value.comet.timestamp.as_unix_nanos(),
                    nextValidatorsHash: value.comet.next_validators_hash.get().into(),
                    root: value.comet.root.hash.get().into(),
                },
                timestamp: value.timestamp,
                storageRoot: value.storage_root.get().into(),
            }
        }
    }

    impl From<SolConsensusState> for ConsensusState {
        fn from(value: SolConsensusState) -> Self {
            Self {
                comet: tendermint_light_client_types::ConsensusState {
                    timestamp: Timestamp::try_from_unix_nanos(value.comet.timestamp.into())
                        .expect("impossible"),
                    next_validators_hash: H256::new(value.comet.nextValidatorsHash.0),
                    root: H256::new(value.comet.root.0).into(),
                },
                timestamp: value.timestamp,
                storage_root: H256::new(value.storageRoot.0),
            }
        }
    }
}
