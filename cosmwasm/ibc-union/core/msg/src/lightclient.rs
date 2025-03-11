use std::collections::BTreeMap;

use unionlabs_primitives::Bytes;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum Status {
    Active,
    Expired,
    Frozen,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct MisbehaviourResponse {
    pub client_state: Bytes,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum VerifyCreationResponseEvent {
    CreateLensClient {
        l1_client_id: u32,
        l2_client_id: u32,
        l2_chain_id: String,
    },
}

pub type StorageWrites = BTreeMap<Bytes, Bytes>;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct VerifyCreationResponse {
    pub latest_height: u64,
    pub counterparty_chain_id: String,
    pub client_state_bytes: Option<Bytes>,
    pub storage_writes: StorageWrites,
    pub events: Vec<VerifyCreationResponseEvent>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct UpdateStateResponse {
    /// The height to save the consensus state at
    pub height: u64,
    /// The client state to overwrite the current one with if provided
    pub client_state_bytes: Option<Bytes>,
    /// The consensus state to save at the `update_height`
    pub consensus_state_bytes: Bytes,
    /// The storage writes which will be written under the client's storage in the core module
    pub storage_writes: StorageWrites,
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum QueryMsg {
    GetTimestamp {
        client_id: u32,
        height: u64,
    },
    GetLatestHeight {
        client_id: u32,
    },
    GetStatus {
        client_id: u32,
    },
    VerifyCreation {
        client_id: u32,
        client_state: Bytes,
        consensus_state: Bytes,
    },
    VerifyMembership {
        client_id: u32,
        height: u64,
        proof: Bytes,
        path: Bytes,
        value: Bytes,
    },
    VerifyNonMembership {
        client_id: u32,
        height: u64,
        proof: Bytes,
        path: Bytes,
    },
    UpdateState {
        client_id: u32,
        caller: String,
    },
    Misbehaviour {
        client_id: u32,
        message: Bytes,
    },
}
