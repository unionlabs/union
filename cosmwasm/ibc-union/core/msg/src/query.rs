use unionlabs::hash::H256;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum QueryMsg {
    GetTimestampAtHeight { client_id: u32, height: u64 },
    GetLatestHeight { client_id: u32 },
    GetClientState { client_id: u32 },
    GetConsensusState { client_id: u32, height: u64 },
    GetStatus { client_id: u32 },
    GetClientType { client_id: u32 },
    GetConnection { connection_id: u32 },
    GetChannel { channel_id: u32 },
    GetChannels { contract: String },
    GetBatchPackets { channel_id: u32, batch_hash: H256 },
    GetBatchReceipts { channel_id: u32, batch_hash: H256 },
}
