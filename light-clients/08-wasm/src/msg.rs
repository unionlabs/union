use cosmwasm_schema::{cw_serde, QueryResponses};

// TODO(aeryz): Move this to its own project
#[cw_serde]
pub struct Height {
    pub revision_number: u64,
    pub revision_height: u64,
}

// TODO(aeryz): This is probably not belong to here
#[cw_serde]
pub struct Header {
    // TODO(aeryz): this might be base64
    pub data: Vec<u8>,
    pub height: Height,
}

// TODO(aeryz): This is probably not belong to here
#[cw_serde]
pub struct Misbehaviour {
    // TODO(aeryz): this might be base64
    pub data: Vec<u8>,
}

#[cw_serde]
pub struct ClientMessage {
    pub header: Option<Header>,
    pub misbehaviour: Option<Misbehaviour>,
}

// TODO(aeryz): not here
#[cw_serde]
pub struct ClientState {
    // TODO(aeryz): Check base64
    pub data: Vec<u8>,
    // TODO(aeryz): This is provided as base64 byte array. Check how to do that in serde.
    pub code_id: String,
    pub latest_height: Height,
}

// TODO(aeryz): not here
#[cw_serde]
pub struct ConsensusState {
    // TODO(aeryz): Check base64
    pub data: Vec<u8>,
    pub timestamp: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    VerifyMembership {
        height: Height,
        delay_time_period: u64,
        delay_block_period: u64,
        // TODO(aeryz): This might be base64
        proof: Vec<u8>,
        // TODO(aeryz): This is a very long type so check that out
        path: String,
        // TODO(aeryz): This might be base64
        value: Vec<u8>,
    },

    VerifyNonMembership {
        height: Height,
        delay_time_period: u64,
        delay_block_period: u64,
        // TODO(aeryz): This might be base64
        proof: Vec<u8>,
        // TODO(aeryz): This is a very long type so check that out
        path: String,
        // TODO(aeryz): This might be base64
        value: Vec<u8>,
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
        // TODO(aeryz): check base64
        proof_upgrade_client: Vec<u8>,
        // TODO(aeryz): check base64
        proof_upgrade_consensus_state: Vec<u8>,
    },

    CheckSubstituteAndUpdateState {},

    ExportMetadata {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(StatusResponse)]
    Status {},
}

// TODO(aeryz): This belongs to 02-client
#[cw_serde]
pub struct GenesisMetadata {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[cw_serde]
pub struct StatusResponse {
    pub status: String,
    pub genesis_metadata: Vec<GenesisMetadata>,
}
