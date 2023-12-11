use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::{channel::channel::Channel, client::height::Height},
    id::PortId,
    TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgChannelOpenTry {
    pub port_id: PortId,
    pub channel: Channel,
    pub counterparty_version: String,
    #[serde(with = "::serde_utils::hex_string")]
    pub proof_init: Vec<u8>,
    pub proof_height: Height,
}

impl TypeUrl for protos::ibc::core::channel::v1::MsgChannelOpenTry {
    const TYPE_URL: &'static str = "/ibc.core.channel.v1.MsgChannelOpenTry";
}
