use unionlabs::primitives::H256;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConsensusState {
    pub state_root: H256,
    pub ibc_storage_root: H256,
    pub timestamp: u64,
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use alloy::sol_types::SolValue;
    use unionlabs::impl_ethabi_via_try_from_into;

    use super::*;

    impl_ethabi_via_try_from_into!(ConsensusState => SolConsensusState);

    alloy::sol! {
        struct SolConsensusState {
            bytes32 state_root;
            bytes32 ibc_storage_root;
            uint64 timestamp;
        }
    }

    impl From<ConsensusState> for SolConsensusState {
        fn from(value: ConsensusState) -> Self {
            Self {
                state_root: value.state_root.get().into(),
                ibc_storage_root: value.ibc_storage_root.get().into(),
                timestamp: value.timestamp,
            }
        }
    }

    impl From<SolConsensusState> for ConsensusState {
        fn from(value: SolConsensusState) -> Self {
            Self {
                state_root: H256::new(value.state_root.0),
                ibc_storage_root: H256::new(value.ibc_storage_root.0),
                timestamp: value.timestamp,
            }
        }
    }
}
