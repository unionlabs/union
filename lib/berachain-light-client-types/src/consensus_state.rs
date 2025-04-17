use ibc_union_spec::Timestamp;
use unionlabs::{
    ibc::core::commitment::merkle_root::MerkleRoot,
    primitives::{encoding::HexUnprefixed, H256},
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConsensusState {
    /// Timestamp of the execution layer.
    pub evm_timestamp: Timestamp,
    /// State root of the execution layer.
    pub evm_state_root: H256,
    /// Storage root of the ibc contract extracted from the state root.
    pub evm_storage_root: H256,

    /// Timestamp of the consensus layer.
    pub comet_timestamp: Timestamp,
    /// State root of the consensus layer.
    pub comet_root: MerkleRoot,
    /// Next validators hash of the consensus layer.
    pub comet_next_validators_hash: H256<HexUnprefixed>,
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use unionlabs::impl_ethabi_via_try_from_into;

    use super::*;

    impl_ethabi_via_try_from_into!(ConsensusState => SolConsensusState);

    alloy::sol! {
        struct SolConsensusState {
            uint64 evmTimestamp;
            bytes32 evmStateRoot;
            bytes32 evmStorageRoot;

            uint64 cometTimestamp;
            bytes32 cometRoot;
            bytes32 cometNextValidatorsHash;
        }
    }

    impl From<ConsensusState> for SolConsensusState {
        fn from(value: ConsensusState) -> Self {
            Self {
                evmTimestamp: value.evm_timestamp.as_nanos(),
                evmStateRoot: value.evm_state_root.get().into(),
                evmStorageRoot: value.evm_storage_root.get().into(),

                cometTimestamp: value.comet_timestamp.as_nanos(),
                cometRoot: value.comet_root.hash.get().into(),
                cometNextValidatorsHash: value.comet_next_validators_hash.get().into(),
            }
        }
    }

    impl From<SolConsensusState> for ConsensusState {
        fn from(value: SolConsensusState) -> Self {
            Self {
                evm_timestamp: Timestamp::from_nanos(value.evmTimestamp),
                evm_state_root: H256::new(value.evmStateRoot.0),
                evm_storage_root: H256::new(value.evmStorageRoot.0),

                comet_timestamp: Timestamp::from_nanos(value.cometTimestamp),
                comet_root: H256::new(value.cometRoot.0).into(),
                comet_next_validators_hash: H256::new(value.cometNextValidatorsHash.0),
            }
        }
    }
}
