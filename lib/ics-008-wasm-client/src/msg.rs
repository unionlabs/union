use std::fmt::Display;

use serde::{Deserialize, Serialize};
use unionlabs::ibc::core::{
    client::{genesis_metadata::GenesisMetadata, height::Height},
    commitment::merkle_path::MerklePath,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct InstantiateMsg {
    pub client_state: Vec<u8>,
    pub consensus_state: Vec<u8>,
    pub checksum: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClientMessage {
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EmptyResult {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StatusResult {
    pub status: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TimestampAtHeightResult {
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CheckForMisbehaviourResult {
    pub found_misbehaviour: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdateStateResult {
    pub heights: Vec<Height>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ExportMetadataResult {
    pub genesis_metadata: Vec<GenesisMetadata>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum SudoMsg {
    VerifyMembership {
        height: Height,
        delay_time_period: u64,
        delay_block_period: u64,
        proof: Vec<u8>,
        path: MerklePath,
        value: Vec<u8>,
    },

    VerifyNonMembership {
        height: Height,
        delay_time_period: u64,
        delay_block_period: u64,
        proof: Vec<u8>,
        path: MerklePath,
    },
    UpdateState {
        client_message: Vec<u8>,
    },

    UpdateStateOnMisbehaviour {
        client_message: Vec<u8>,
    },

    VerifyUpgradeAndUpdateState {
        upgrade_client_state: Vec<u8>,
        upgrade_consensus_state: Vec<u8>,
        proof_upgrade_client: Vec<u8>,
        proof_upgrade_consensus_state: Vec<u8>,
    },

    MigrateClientStore {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum QueryMsg {
    VerifyClientMessage { client_message: Vec<u8> },

    CheckForMisbehaviour { client_message: Vec<u8> },

    TimestampAtHeight { height: Height },

    Status {},

    ExportMetadata {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Status {
    Active,
    Frozen,
    Expired,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Active => write!(f, "Active"),
            Status::Frozen => write!(f, "Frozen"),
            Status::Expired => write!(f, "Expired"),
        }
    }
}

impl From<Status> for StatusResult {
    fn from(value: Status) -> Self {
        StatusResult {
            status: value.to_string(),
        }
    }
}
