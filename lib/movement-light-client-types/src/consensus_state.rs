use consensus_primitives::Timestamp;
use unionlabs::primitives::H256;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConsensusState {
    pub state_root: H256,
    pub timestamp: Timestamp,
    /// This is the hash of the `StateProof` which is committed to l1
    pub state_proof_hash: H256,
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
            uint64 timestamp;
            /// This is the hash of the `StateProof` which is committed to l1
            bytes32 state_proof_hash;
        }
    }

    impl From<ConsensusState> for SolConsensusState {
        fn from(value: ConsensusState) -> Self {
            Self {
                state_root: value.state_root.get().into(),
                timestamp: value.timestamp.as_nanos(),
                state_proof_hash: value.state_proof_hash.get().into(),
            }
        }
    }

    impl From<SolConsensusState> for ConsensusState {
        fn from(value: SolConsensusState) -> Self {
            Self {
                state_root: H256::new(value.state_root.0),
                timestamp: Timestamp::from_nanos(value.timestamp),
                state_proof_hash: H256::new(value.state_proof_hash.0),
            }
        }
    }
}
