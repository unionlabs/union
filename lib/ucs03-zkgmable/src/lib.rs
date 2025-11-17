#![no_std]

extern crate alloc;

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
    /// Caller is the account that submitted the recv_packet message to this chain.
    ///
    /// Can also be thought of as "tx.origin".
    pub caller: Addr,
    pub path: U256,
    pub source_channel_id: ChannelId,
    pub destination_channel_id: ChannelId,
    pub sender: Bytes,
    pub message: Bytes,
    /// An arbitrary address provided by `caller` on message submission.
    ///
    /// This can be used for arbitrary rewards depending on the protocol, and is intentionally separate from `caller` to allow for this address to be a cold wallet.
    pub relayer: Addr,
    pub relayer_msg: Bytes,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct OnIntentZkgm {
    /// Caller is the account that submitted the intent_recv_packet message to this chain.
    ///
    /// Can also be thought of as "tx.origin".
    pub caller: Addr,
    pub path: U256,
    pub source_channel_id: ChannelId,
    pub destination_channel_id: ChannelId,
    pub sender: Bytes,
    pub message: Bytes,
    pub market_maker: Addr,
    pub market_maker_msg: Bytes,
}
