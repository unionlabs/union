use macros::model;

use crate::{
    bytes::Bytes,
    ibc::core::{channel::channel::Channel, client::height::Height},
    id::PortId,
};

#[model(proto(raw(protos::ibc::core::channel::v1::MsgChannelOpenTry)))]
pub struct MsgChannelOpenTry {
    pub port_id: PortId,
    pub channel: Channel,
    pub counterparty_version: String,
    pub proof_init: Bytes,
    pub proof_height: Height,
}
