use ibc_union_spec::{ChannelId, ClientId, ConnectionId};
use unionlabs_primitives::H256;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum QueryMsg {
    GetTimestampAtHeight { client_id: ClientId, height: u64 },
    GetLatestHeight { client_id: ClientId },
    GetClientState { client_id: ClientId },
    GetConsensusState { client_id: ClientId, height: u64 },
    GetStatus { client_id: ClientId },
    GetClientType { client_id: ClientId },
    GetConnection { connection_id: ConnectionId },
    GetChannel { channel_id: ChannelId },
    GetChannels { contract: String },
    GetBatchPackets { batch_hash: H256 },
    GetBatchReceipts { batch_hash: H256 },
    GetClientImpl { client_id: ClientId },
    GetRegisteredClientType { client_type: String },
}
