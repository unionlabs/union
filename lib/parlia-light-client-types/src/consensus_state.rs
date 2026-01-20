use ibc_union_spec::Timestamp;
use unionlabs::primitives::H256;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConsensusState {
    pub valset_epoch_block_number: u64,
    pub state_root: H256,
    pub timestamp: Timestamp,
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use unionlabs::impl_ethabi_via_try_from_into;

    use super::*;

    impl_ethabi_via_try_from_into!(ConsensusState => SolConsensusState);

    alloy::sol! {
        struct SolConsensusState {
            uint64 valset_epoch_block_number;
            bytes32 state_root;
            uint64 timestamp;
        }
    }

    impl From<ConsensusState> for SolConsensusState {
        fn from(value: ConsensusState) -> Self {
            Self {
                valset_epoch_block_number: value.valset_epoch_block_number,
                state_root: value.state_root.get().into(),
                timestamp: value.timestamp.as_nanos(),
            }
        }
    }

    impl From<SolConsensusState> for ConsensusState {
        fn from(value: SolConsensusState) -> Self {
            Self {
                valset_epoch_block_number: value.valset_epoch_block_number,
                state_root: H256::new(value.state_root.0),
                timestamp: Timestamp::from_nanos(value.timestamp),
            }
        }
    }
}
