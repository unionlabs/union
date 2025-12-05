use ibc_union_spec::Timestamp;
use starknet_types::Felt;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConsensusState {
    pub contracts_trie_root: Felt,
    pub classes_trie_root: Felt,
    pub ibc_storage_root: Felt,
    pub timestamp: Timestamp,
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use unionlabs_encoding::impl_ethabi_via_try_from_into;

    use super::*;

    impl_ethabi_via_try_from_into!(ConsensusState => SolConsensusState);

    alloy_sol_types::sol! {
        struct SolConsensusState {
            bytes32 contracts_trie_root;
            bytes32 classes_trie_root;
            bytes32 ibc_storage_root;
            uint64 timestamp;
        }
    }

    impl From<ConsensusState> for SolConsensusState {
        fn from(value: ConsensusState) -> Self {
            Self {
                contracts_trie_root: value.contracts_trie_root.to_be_bytes().into(),
                classes_trie_root: value.classes_trie_root.to_be_bytes().into(),
                ibc_storage_root: value.ibc_storage_root.to_be_bytes().into(),
                timestamp: value.timestamp.as_nanos(),
            }
        }
    }

    impl From<SolConsensusState> for ConsensusState {
        fn from(value: SolConsensusState) -> Self {
            Self {
                contracts_trie_root: Felt::from_be_bytes(value.contracts_trie_root.0),
                classes_trie_root: Felt::from_be_bytes(value.classes_trie_root.0),
                ibc_storage_root: Felt::from_be_bytes(value.ibc_storage_root.0),
                timestamp: Timestamp::from_nanos(value.timestamp),
            }
        }
    }
}
