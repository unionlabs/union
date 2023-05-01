use cosmwasm_std::Binary;
use ibc::Height;
use ibc_proto::ibc::{
    core::client::v1::GenesisMetadata,
    lightclients::wasm::v1::{ClientState, ConsensusState, Header, Misbehaviour},
};
use serde::{Deserialize, Serialize};

// TODO(aeryz): Normally, this type should be an enum. Need more
// research on that. For now, this is fine though.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MerklePath {
    key_path: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClientMessage {
    pub header: Option<Header>,
    pub misbehaviour: Option<Misbehaviour>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
pub enum QueryMsg {
    Status {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StatusResponse {
    pub status: String,
    pub genesis_metadata: Vec<GenesisMetadata>,
}
