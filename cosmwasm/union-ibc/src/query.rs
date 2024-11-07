use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;

use crate::lightclient::query::Status;

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
}
