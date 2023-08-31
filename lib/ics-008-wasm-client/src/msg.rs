use std::fmt::Display;

use cosmwasm_std::{to_binary, Binary, StdResult};
use protos::ibc::{
    core::client::v1::GenesisMetadata,
    lightclients::wasm::v1::{ClientState, ConsensusState, Header, Misbehaviour},
};
use serde::{Deserialize, Serialize};
use unionlabs::ibc::core::client::height::Height;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MerklePath {
    pub key_path: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ClientMessage {
    Header(Header),
    Misbehaviour(Misbehaviour),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ContractResult {
    pub is_valid: bool,
    pub error_msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<u8>>,
    pub found_misbehaviour: bool,
}

impl ContractResult {
    pub fn valid(data: Option<Vec<u8>>) -> Self {
        Self {
            is_valid: true,
            error_msg: Default::default(),
            data,
            found_misbehaviour: false,
        }
    }

    pub fn invalid(error_msg: String) -> Self {
        Self {
            is_valid: false,
            error_msg,
            data: None,
            found_misbehaviour: true,
        }
    }

    pub fn encode(self) -> StdResult<Binary> {
        to_binary(&self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum ExecuteMsg {
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

    VerifyClientMessage {
        client_message: ClientMessage,
    },

    UpdateState {
        client_message: ClientMessage,
    },

    UpdateStateOnMisbehaviour {
        client_message: ClientMessage,
    },

    CheckForMisbehaviour {
        client_message: ClientMessage,
    },

    VerifyUpgradeAndUpdateState {
        upgrade_client_state: ClientState,
        upgrade_consensus_state: ConsensusState,
        proof_upgrade_client: Binary,
        proof_upgrade_consensus_state: Binary,
    },

    CheckSubstituteAndUpdateState {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum QueryMsg {
    Status {},
    ExportMetadata {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct QueryResponse {
    pub status: String,
    pub genesis_metadata: Vec<GenesisMetadata>,
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

impl From<Status> for QueryResponse {
    fn from(value: Status) -> Self {
        QueryResponse {
            status: value.to_string(),
            genesis_metadata: Vec::new(),
        }
    }
}
