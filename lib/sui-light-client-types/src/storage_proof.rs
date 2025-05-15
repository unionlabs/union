use crate::{
    checkpoint_summary::CheckpointContents, object::ObjectInner,
    transaction_effects::TransactionEffects,
};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct StorageProof {
    pub checkpoint_contents: CheckpointContents,
    pub transaction_effects: TransactionEffects,
    pub object: ObjectInner,
}
