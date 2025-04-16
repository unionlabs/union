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
    #[cfg_attr(feature = "cw-orch-interface", returns(Timestamp))]
    GetTimestampAtHeight { client_id: ClientId, height: u64 },
    #[cfg_attr(feature = "cw-orch-interface", returns(u64))]
    GetLatestHeight { client_id: ClientId },
    #[cfg_attr(feature = "cw-orch-interface", returns(cosmwasm_std::Binary))]
    GetClientState { client_id: ClientId },
    #[cfg_attr(feature = "cw-orch-interface", returns(cosmwasm_std::Binary))]
    GetConsensusState { client_id: ClientId, height: u64 },
    #[cfg_attr(feature = "cw-orch-interface", returns(crate::lightclient::Status))]
    GetStatus { client_id: ClientId },
    #[cfg_attr(feature = "cw-orch-interface", returns(u64))]
    GetClientType { client_id: ClientId },
    #[cfg_attr(
        feature = "cw-orch-interface",
        returns(ibc_union_spec::types::Connection)
    )]
    GetConnection { connection_id: ConnectionId },
    #[cfg_attr(feature = "cw-orch-interface", returns(ibc_union_spec::types::Channel))]
    GetChannel { channel_id: ChannelId },
    #[cfg_attr(feature = "cw-orch-interface", returns(std::collections::BTreeSet<u32>))]
    GetChannels { contract: String },
    #[cfg_attr(feature = "cw-orch-interface", returns(Option<Vec<u8>>))]
    GetBatchPackets { batch_hash: H256 },
    #[cfg_attr(feature = "cw-orch-interface", returns(Option<Vec<u8>>))]
    GetBatchReceipts { batch_hash: H256 },
    #[cfg_attr(feature = "cw-orch-interface", returns(cosmwasm_std::Addr))]
    GetClientImpl { client_id: ClientId },
    #[cfg_attr(feature = "cw-orch-interface", returns(cosmwasm_std::Addr))]
    GetRegisteredClientType { client_type: String },
}
