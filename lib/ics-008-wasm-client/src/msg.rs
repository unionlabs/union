use std::fmt::Display;

use cosmwasm_std::Binary;
use serde::{Deserialize, Serialize};
use unionlabs::ibc::core::{
    client::{genesis_metadata::GenesisMetadata, height::Height},
    commitment::merkle_path::MerklePath,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct InstantiateMsg {
    pub client_state: Binary,
    pub consensus_state: Binary,
    pub checksum: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClientMessage {
    pub data: Binary,
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
        proof: Binary,
        path: MerklePath,
        value: Binary,
    },

    VerifyNonMembership {
        height: Height,
        delay_time_period: u64,
        delay_block_period: u64,
        proof: Binary,
        path: MerklePath,
    },
    UpdateState {
        client_message: Binary,
    },

    UpdateStateOnMisbehaviour {
        client_message: Binary,
    },

    VerifyUpgradeAndUpdateState {
        upgrade_client_state: Binary,
        upgrade_consensus_state: Binary,
        proof_upgrade_client: Binary,
        proof_upgrade_consensus_state: Binary,
    },

    MigrateClientStore {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum QueryMsg {
    VerifyClientMessage { client_message: Binary },

    CheckForMisbehaviour { client_message: Binary },

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
