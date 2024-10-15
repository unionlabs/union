use core::num::NonZeroU64;

use macros::model;

use crate::ibc::core::{channel::packet::Packet, client::height::Height};

#[model(proto(raw(protos::ibc::core::channel::v1::MsgRecvPacket)))]
pub struct MsgTimeout {
    pub packet: Packet,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub proof_unreceived: Vec<u8>,
    pub proof_height: Height,
    pub next_sequence_recv: NonZeroU64,
}
