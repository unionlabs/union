use macros::model;

use crate::{
    bytes::Bytes,
    ibc::core::client::height::Height,
    id::{ChannelId, PortId},
};

#[model(proto(raw(protos::ibc::core::channel::v1::MsgChannelOpenAck)))]
pub struct MsgChannelOpenAck {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub counterparty_channel_id: ChannelId,
    // yes, this is actually just an unbounded string
    pub counterparty_version: String,
    pub proof_try: Bytes,
    pub proof_height: Height,
}
