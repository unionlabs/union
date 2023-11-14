use serde::{Deserialize, Serialize};

use crate::{ibc::core::channel::channel::Channel, id::PortId, TypeUrl};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgChannelOpenInit {
    pub port_id: PortId,
    pub channel: Channel,
}

impl TypeUrl for protos::ibc::core::channel::v1::MsgChannelOpenInit {
    const TYPE_URL: &'static str = "/ibc.core.channel.v1.MsgChannelOpenInit";
}
