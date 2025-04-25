use cosmwasm_std::{Addr, Uint256};
use ibc_union_spec::ChannelId;
use serde::{Deserialize, Serialize};
use unionlabs::primitives::Bytes;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct InitMsg {
    pub config: Config,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct Config {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum ExecuteMsg {
    Execute {
        message: Bytes,
        intent: bool,
    },
    OnZkgm {
        caller: Addr,
        path: Uint256,
        source_channel_id: ChannelId,
        destination_channel_id: ChannelId,
        sender: Bytes,
        message: Bytes,
        relayer: Addr,
        relayer_msg: Bytes,
    },
    OnIntentZkgm {
        caller: Addr,
        path: Uint256,
        source_channel_id: ChannelId,
        destination_channel_id: ChannelId,
        sender: Bytes,
        message: Bytes,
        market_maker: Addr,
        market_maker_msg: Bytes,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct MigrateMsg {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum QueryMsg {}
