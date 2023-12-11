use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::client::height::Height,
    id::{ChannelId, PortId},
    TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgChannelOpenAck {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub counterparty_channel_id: ChannelId,
    // yes, this is actually just an unbounded string
    pub counterparty_version: String,
    #[serde(with = "::serde_utils::hex_string")]
    pub proof_try: Vec<u8>,
    pub proof_height: Height,
}

impl TypeUrl for protos::ibc::core::channel::v1::MsgChannelOpenAck {
    const TYPE_URL: &'static str = "/ibc.core.channel.v1.MsgChannelOpenAck";
}
