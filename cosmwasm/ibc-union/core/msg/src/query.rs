use unionlabs_primitives::H256;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[derive(cosmwasm_schema::QueryResponses, cw_orch::QueryFns)]
pub enum QueryMsg {
    #[returns(u64)]
    GetTimestampAtHeight { client_id: u32, height: u64 },
    #[returns(u64)]
    GetLatestHeight { client_id: u32 },
    #[returns(cosmwasm_std::Binary)]
    GetClientState { client_id: u32 },
    #[returns(cosmwasm_std::Binary)]
    GetConsensusState { client_id: u32, height: u64 },
    #[returns(crate::lightclient::Status)]
    GetStatus { client_id: u32 },
    #[returns(u64)]
    GetClientType { client_id: u32 },
    #[returns(ibc_solidity::Connection)]
    GetConnection { connection_id: u32 },
    #[returns(ibc_solidity::Channel)]
    GetChannel { channel_id: u32 },
    #[returns(BTreeSet<u32>)]
    GetChannels { contract: String },
    #[returns(Option<H256>)]
    GetBatchPackets { channel_id: u32, batch_hash: H256 },
    #[returns(Option<H256>)]
    GetBatchReceipts { channel_id: u32, batch_hash: H256 },
}
