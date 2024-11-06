#[derive(serde::Serialize, serde::Deserialize)]
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
}
