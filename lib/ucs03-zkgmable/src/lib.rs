use cosmwasm_std::Addr;
use ibc_union_spec::ChannelId;
use serde::{Deserialize, Serialize};
use unionlabs_primitives::{Bytes, U256};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum Zkgmable {
    OnZkgm(OnZkgm),
    OnIntentZkgm(OnIntentZkgm),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct OnZkgm {
    pub caller: Addr,
    pub path: U256,
    pub source_channel_id: ChannelId,
    pub destination_channel_id: ChannelId,
    pub sender: Bytes,
    pub message: Bytes,
    pub relayer: Addr,
    pub relayer_msg: Bytes,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct OnIntentZkgm {
    pub caller: Addr,
    pub path: U256,
    pub source_channel_id: ChannelId,
    pub destination_channel_id: ChannelId,
    pub sender: Bytes,
    pub message: Bytes,
    pub market_maker: Addr,
    pub market_maker_msg: Bytes,
}
