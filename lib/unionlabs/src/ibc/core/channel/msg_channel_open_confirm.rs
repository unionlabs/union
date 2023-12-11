use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::client::height::Height,
    id::{ChannelId, PortId},
    TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgChannelOpenConfirm {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    #[serde(with = "::serde_utils::hex_string")]
    pub proof_ack: Vec<u8>,
    pub proof_height: Height,
}

impl TypeUrl for protos::ibc::core::channel::v1::MsgChannelOpenConfirm {
    const TYPE_URL: &'static str = "/ibc.core.channel.v1.MsgChannelOpenConfirm";
}
