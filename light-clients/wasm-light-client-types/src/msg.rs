use cosmwasm_std::{to_binary, Binary, StdResult};
use protos::ibc::{
    core::client::v1::GenesisMetadata,
    lightclients::wasm::v1::{ClientState, ConsensusState, Header, Misbehaviour},
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Height {
    /// Previously known as "epoch"
    #[serde(default)]
    pub revision_number: u64,

    /// The height of a block
    #[serde(default)]
    pub revision_height: u64,
}

impl TryInto<ibc::Height> for Height {
    type Error = ();

    fn try_into(self) -> Result<ibc::Height, Self::Error> {
        ibc::Height::new(self.revision_number, self.revision_height).map_err(|_| ())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MerklePath {
    pub key_path: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClientMessage {
    pub header: Option<Header>,
    pub misbehaviour: Option<Misbehaviour>,
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
        value: Binary,
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

    ExportMetadata {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum QueryMsg {
    Status {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StatusResponse {
    pub status: String,
    pub genesis_metadata: Vec<GenesisMetadata>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Status {
    Active,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Active => write!(f, "Active"),
        }
    }
}
