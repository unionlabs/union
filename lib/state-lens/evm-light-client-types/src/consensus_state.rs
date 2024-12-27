use unionlabs::hash::H256;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConsensusState {
    /// Timestamp of the execution layer.
    pub timestamp: u64,
    /// State root of the execution layer.
    pub state_root: H256,
    /// Storage root of the ibc contract extracted from the state root.
    pub storage_root: H256,
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
            bytes32 stateRoot;
            bytes32 storageRoot;
        }
    }

    impl From<ConsensusState> for SolConsensusState {
        fn from(value: ConsensusState) -> Self {
            Self {
                timestamp: value.timestamp,
                stateRoot: value.state_root.get().into(),
                storageRoot: value.storage_root.get().into(),
            }
        }
    }

    impl From<SolConsensusState> for ConsensusState {
        fn from(value: SolConsensusState) -> Self {
            Self {
                timestamp: value.timestamp,
                state_root: H256::new(value.stateRoot.0),
                storage_root: H256::new(value.storageRoot.0),
            }
        }
    }
}
