use std::fmt::Display;

use cosmwasm_std::Binary;
use protos::ibc::{
    core::client::v1::GenesisMetadata,
    lightclients::wasm::v1::{ClientState, ConsensusState},
};
use serde::{Deserialize, Serialize};
use unionlabs::ibc::core::client::height::Height;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MerklePath {
    pub key_path: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum ClientMessage<Header, Misbehaviour> {
    Header(Header),
    Misbehaviour(Misbehaviour),
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
    genesis_metadata: GenesisMetadata,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum SudoMsg<Header, Misbehaviour> {
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
        client_message: ClientMessage<Header, Misbehaviour>,
    },

    UpdateStateOnMisbehaviour {
        client_message: ClientMessage<Header, Misbehaviour>,
    },

    VerifyUpgradeAndUpdateState {
        upgrade_client_state: ClientState,
        upgrade_consensus_state: ConsensusState,
        proof_upgrade_client: Binary,
        proof_upgrade_consensus_state: Binary,
    },

    MigrateClientStore {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum QueryMsg<Header, Misbehaviour> {
    VerifyClientMessage {
        client_message: ClientMessage<Header, Misbehaviour>,
    },

    CheckForMisbehaviour {
        client_message: ClientMessage<Header, Misbehaviour>,
    },

    TimestampAtHeight {
        height: Height,
    },

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

// #[cfg(test)]
// mod tests {
//     use crate::{ClientMessage, ExecuteMsg};

//     #[test]
//     fn execute_msg_snake_case_encoded() {
//         let msg = ExecuteMsg::CheckSubstituteAndUpdateState {};
//         assert_eq!(
//             serde_json::to_string(&msg).unwrap(),
//             r#"{"check_substitute_and_update_state":{}}"#
//         )
//     }
// }
