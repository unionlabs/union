use macros::model;

use crate::{
    ibc::core::{channel::channel::Channel, client::height::Height},
    id::PortId,
};

#[model(proto(raw(protos::ibc::core::channel::v1::MsgChannelOpenTry)))]
pub struct MsgChannelOpenTry {
    pub port_id: PortId,
    pub channel: Channel,
    pub counterparty_version: String,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub proof_init: Vec<u8>,
    pub proof_height: Height,
}
