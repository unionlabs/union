use unionlabs::bytes::Bytes;

#[derive(serde::Serialize, serde::Deserialize)]
pub enum Status {
    Active,
    Expired,
    Frozen,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct VerifyClientMessageUpdate {
    pub height: u64,
    pub consensus_state: Bytes,
    pub client_state: Bytes,
}

#[derive(serde::Serialize, serde::Deserialize)]
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
    VerifyClientMessage {
        client_id: u32,
        message: Bytes,
    },
    Misbehaviour {
        client_id: u32,
        message: Bytes,
    },
}
