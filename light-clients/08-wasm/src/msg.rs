use cosmwasm_schema::{cw_serde, QueryResponses};

// TODO(aeryz): Move this to its own project
#[cw_serde]
pub struct Height {
    #[serde(default)]
    pub revision_number: u64,
    #[serde(default)]
    pub revision_height: u64,
}

// // TODO(aeryz): Move this to its own project
// // TODO(aeryz): Also, the only path I see is used is MerklePath
// // check if this can be MerklePath or enum.
// #[cw_serde]
// #[serde(untagged)]
// pub enum Path {
//     Merkle { key_path: Vec<String> },
// }

// TODO(aeryz): Normally, the above Path was being used but serde untagged
// produces floating point code which makes the code unusable. Will figure out.
#[cw_serde]
pub struct MerklePath {
    key_path: Vec<String>,
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
        // TODO(aeryz): This is base64, check serde to convert from base64 to bytes
        proof: String,
        // TODO(aeryz): This is a very long type so check that out
        path: MerklePath,
        // TODO(aeryz): This is base64, check serde to convert from base64 to bytes
        value: String,
    },

    VerifyNonMembership {
        height: Height,
        delay_time_period: u64,
        delay_block_period: u64,
        // TODO(aeryz): This is base64, check serde to convert from base64 to bytes
        proof: String,
        // TODO(aeryz): This is a very long type so check that out
        path: MerklePath,
        // TODO(aeryz): This is base64, check serde to convert from base64 to bytes
        value: String,
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
