use unionlabs::{google::protobuf::timestamp::Timestamp, hash::H256};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConsensusState {
    /// Timestamp of the execution layer.
    pub eth_timestamp: u64,
    /// Timestamp of the cometbft beacon node (consensus layer).
    pub comet_timestamp: Timestamp,

    /// Storage root of the execution layer.
    pub eth_storage_root: H256,
    /// Next validators hash of the cometbft beacon node (consensus layer).
    pub comet_next_validators_hash: H256,
}
