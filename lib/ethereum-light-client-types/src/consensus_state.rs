use beacon_api_types::custom_types::Slot;
use consensus_primitives::Timestamp;
use unionlabs::primitives::H256;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConsensusState {
    pub slot: Slot,
    /// The state root for this chain, used for L2s to verify against this contract.
    pub state_root: H256,
    pub storage_root: H256,
    /// Timestamp of the block, *normalized to nanoseconds* in order to be compatible with ibc-go.
    pub timestamp: Timestamp,
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use alloy::sol_types::SolValue;
    use unionlabs::{impl_ethabi_via_try_from_into, primitives::FixedBytesError};

    use super::*;

    impl_ethabi_via_try_from_into!(ConsensusState => SolConsensusState);

    alloy::sol! {
        struct SolConsensusState {
            uint64 slot;
            bytes32 state_root;
            bytes32 storage_root;
            uint64 timestamp;
        }
    }

    impl From<ConsensusState> for SolConsensusState {
        fn from(value: ConsensusState) -> Self {
            Self {
                slot: value.slot.get(),
                state_root: value.state_root.get().into(),
                storage_root: value.storage_root.get().into(),
                timestamp: value.timestamp.as_nanos(),
            }
        }
    }

    #[derive(Debug, thiserror::Error)]
    pub enum TryFromEthAbiError {
        #[error("invalid current sync committee")]
        CurrentSyncCommittee(#[source] FixedBytesError),
        #[error("invalid next sync committee")]
        NextSyncCommittee(#[source] FixedBytesError),
    }

    impl TryFrom<SolConsensusState> for ConsensusState {
        type Error = TryFromEthAbiError;

        fn try_from(value: SolConsensusState) -> Result<Self, Self::Error> {
            Ok(Self {
                slot: Slot::new(value.slot),
                state_root: H256::new(value.state_root.0),
                storage_root: H256::new(value.storage_root.0),
                timestamp: Timestamp::from_nanos(value.timestamp),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use unionlabs::{
        encoding::{EthAbi, Json},
        primitives::H256,
        test_utils::assert_codec_iso,
    };

    use super::*;

    fn mk_consensus_state() -> ConsensusState {
        ConsensusState {
            slot: Slot::new(42),
            state_root: H256::new([0xAA; 32]),
            storage_root: H256::new([0xAA; 32]),
            timestamp: Timestamp::from_nanos(123_456_789),
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
}
