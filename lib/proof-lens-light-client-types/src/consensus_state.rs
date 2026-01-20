use ibc_union_spec::Timestamp;
use unionlabs_primitives::Bytes;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ConsensusState {
    pub timestamp: Timestamp,
    pub l1_height: u64,
    pub raw_l2_consensus_state: Bytes,
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use unionlabs_encoding::impl_ethabi_via_try_from_into;

    use super::*;

    impl_ethabi_via_try_from_into!(ConsensusState => SolConsensusState);

    alloy_sol_types::sol! {
        struct SolConsensusState {
            uint64 timestamp;
            uint64 l1Height;
            bytes rawL2ConsensusState;
        }
    }

    impl From<ConsensusState> for SolConsensusState {
        fn from(value: ConsensusState) -> Self {
            Self {
                timestamp: value.timestamp.as_nanos(),
                l1Height: value.l1_height,
                rawL2ConsensusState: value.raw_l2_consensus_state.into(),
            }
        }
    }

    impl From<SolConsensusState> for ConsensusState {
        fn from(value: SolConsensusState) -> Self {
            Self {
                timestamp: Timestamp::from_nanos(value.timestamp),
                l1_height: value.l1Height,
                raw_l2_consensus_state: value.rawL2ConsensusState.into(),
            }
        }
    }
}
