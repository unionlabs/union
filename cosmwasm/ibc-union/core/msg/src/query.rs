use ibc_union_spec::{ChannelId, ClientId, ConnectionId};
use unionlabs_primitives::H256;
#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "cw-orch-interface",
    derive(cosmwasm_schema::QueryResponses, cw_orch::QueryFns)
)]
pub enum QueryMsg {
    #[cfg_attr(feature = "cw-orch-interface", returns(ibc_union_spec::Timestamp))]
    GetTimestampAtHeight { client_id: ClientId, height: u64 },
    #[cfg_attr(feature = "cw-orch-interface", returns(u64))]
    GetLatestHeight { client_id: ClientId },
    #[cfg_attr(feature = "cw-orch-interface", returns(unionlabs_primitives::Bytes))]
    GetClientState { client_id: ClientId },
    #[cfg_attr(feature = "cw-orch-interface", returns(unionlabs_primitives::Bytes))]
    GetConsensusState { client_id: ClientId, height: u64 },
    #[cfg_attr(feature = "cw-orch-interface", returns(crate::lightclient::Status))]
    GetStatus { client_id: ClientId },
    #[cfg_attr(feature = "cw-orch-interface", returns(String))]
    GetClientType { client_id: ClientId },
    #[cfg_attr(feature = "cw-orch-interface", returns(ibc_union_spec::Connection))]
    GetConnection { connection_id: ConnectionId },
    #[cfg_attr(feature = "cw-orch-interface", returns(ibc_union_spec::Channel))]
    GetChannel { channel_id: ChannelId },
    #[cfg_attr(feature = "cw-orch-interface", returns(std::collections::BTreeSet<u32>))]
    GetChannels { contract: String },
    #[cfg_attr(feature = "cw-orch-interface", returns(Option<H256>))]
    GetBatchPackets { batch_hash: H256 },
    #[cfg_attr(feature = "cw-orch-interface", returns(Option<H256>))]
    GetBatchReceipts { batch_hash: H256 },
    #[cfg_attr(feature = "cw-orch-interface", returns(cosmwasm_std::Addr))]
    GetClientImpl { client_id: ClientId },
    #[cfg_attr(feature = "cw-orch-interface", returns(String))]
    GetRegisteredClientType { client_type: String },
}
