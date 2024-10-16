use macros::model;

use crate::{ibc::core::channel::channel::Channel, id::PortId};

#[model(proto(raw(protos::ibc::core::channel::v1::MsgChannelOpenInit)))]
#[cfg_attr(feature = "valuable", derive(valuable::Valuable))]
pub struct MsgChannelOpenInit {
    pub port_id: PortId,
    pub channel: Channel,
}
