use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;

#[cw_serde]
pub enum Status {
    Active,
    Expired,
    Frozen,
}

#[cw_serde]
pub struct VerifyClientMessageUpdate {
    pub height: u64,
    pub consensus_state: Binary,
    pub client_state: Binary,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(u64)]
    GetTimestampAtHeight { client_id: u32, height: u64 },
    #[returns(u64)]
    GetLatestHeight { client_id: u32 },
    #[returns(Binary)]
    GetClientState { client_id: u32 },
    #[returns(Binary)]
    GetConsensusState { client_id: u32, height: u64 },
    #[returns(Status)]
    GetStatus { client_id: u32 },
    #[returns(u64)]
    VerifyCreation {
        client_id: u32,
        client_state: Binary,
        consensus_state: Binary,
    },
    #[returns(())]
    VerifyMembership {
        client_id: u32,
        height: u64,
        proof: Binary,
        path: Binary,
        value: Binary,
    },
    #[returns(())]
    VerifyNonMembership {
        client_id: u32,
        height: u64,
        proof: Binary,
        path: Binary,
    },
    #[returns(VerifyClientMessageUpdate)]
    VerifyClientMessage { client_id: u32, message: Binary },
    #[returns(bool)]
    CheckForMisbehavior { client_id: u32, message: Binary },
}
