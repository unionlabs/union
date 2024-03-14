use macros::model;

use crate::{ibc::core::channel::channel::Channel, id::PortId};

#[model(proto(raw(protos::ibc::core::channel::v1::MsgChannelOpenInit)))]
pub struct MsgChannelOpenInit {
    pub port_id: PortId,
    pub channel: Channel,
}
