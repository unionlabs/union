use macros::model;

use super::sparse_merkle_proof::SparseMerkleProof;

#[model]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct StorageProof {
    pub state_value: Option<StateValue>,
    pub proof: SparseMerkleProof,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum StateValue {
    V0(Vec<u8>),
    WithMetadata {
        data: Vec<u8>,
        metadata: StateValueMetadata,
    },
}

impl StateValue {
    #[must_use]
    pub fn data(&self) -> &[u8] {
        match self {
            StateValue::V0(data) | StateValue::WithMetadata { data, .. } => data,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum StateValueMetadata {
    V0 {
        deposit: u64,
        creation_time_usecs: u64,
    },
    V1 {
        slot_deposit: u64,
        bytes_deposit: u64,
        creation_time_usecs: u64,
    },
}
