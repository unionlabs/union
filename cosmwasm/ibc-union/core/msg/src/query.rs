use unionlabs_primitives::H256;
#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "cw-orch-interface",
    derive(cosmwasm_schema::QueryResponses, cw_orch::QueryFns)
)]
pub enum QueryMsg {
    #[cfg_attr(feature = "cw-orch-interface", returns(u64))]
    GetTimestampAtHeight { client_id: u32, height: u64 },
    #[cfg_attr(feature = "cw-orch-interface", returns(u64))]
    GetLatestHeight { client_id: u32 },
    #[cfg_attr(feature = "cw-orch-interface", returns(cosmwasm_std::Binary))]
    GetClientState { client_id: u32 },
    #[cfg_attr(feature = "cw-orch-interface", returns(cosmwasm_std::Binary))]
    GetConsensusState { client_id: u32, height: u64 },
    #[cfg_attr(feature = "cw-orch-interface", returns(crate::lightclient::Status))]
    GetStatus { client_id: u32 },
    #[cfg_attr(feature = "cw-orch-interface", returns(u64))]
    GetClientType { client_id: u32 },
    #[cfg_attr(
        feature = "cw-orch-interface",
        returns(ibc_union_spec::types::Connection)
    )]
    GetConnection { connection_id: u32 },
    #[cfg_attr(feature = "cw-orch-interface", returns(ibc_union_spec::types::Channel))]
    GetChannel { channel_id: u32 },
    #[cfg_attr(feature = "cw-orch-interface", returns(std::collections::BTreeSet<u32>))]
    GetChannels { contract: String },
    #[cfg_attr(feature = "cw-orch-interface", returns(Option<Vec<u8>>))]
    GetBatchPackets { channel_id: u32, batch_hash: H256 },
    #[cfg_attr(feature = "cw-orch-interface", returns(Option<Vec<u8>>))]
    GetBatchReceipts { channel_id: u32, batch_hash: H256 },
    #[cfg_attr(feature = "cw-orch-interface", returns(cosmwasm_std::Addr))]
    GetClientImpl { client_id: u32 },
    #[cfg_attr(feature = "cw-orch-interface", returns(cosmwasm_std::Addr))]
    GetRegisteredClientType { client_type: String },
}
