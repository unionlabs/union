use crate::{committee::Committee, ObjectID};

#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "version", content = "data", rename_all = "snake_case")
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum ClientState {
    V1(ClientStateV1),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ClientStateV1 {
    pub chain_id: String,
    pub latest_checkpoint: u64,
    pub frozen_height: u64,
    pub ibc_commitments_object_id: ObjectID,
    #[cfg_attr(
        feature = "serde",
        serde(default),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub initial_committee: Option<Committee>,
}
