use macros::model;

use crate::ibc::core::{channel::packet::Packet, client::height::Height};

#[model(proto(raw(protos::ibc::core::channel::v1::MsgAcknowledgement)))]
pub struct MsgAcknowledgement {
    pub packet: Packet,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub acknowledgement: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub proof_acked: Vec<u8>,
    pub proof_height: Height,
}
