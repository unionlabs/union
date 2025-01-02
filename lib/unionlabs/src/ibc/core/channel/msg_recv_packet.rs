use macros::model;
use unionlabs_bytes::Bytes;

use crate::ibc::core::{channel::packet::Packet, client::height::Height};

#[model(proto(raw(protos::ibc::core::channel::v1::MsgRecvPacket)))]
pub struct MsgRecvPacket {
    pub packet: Packet,
    pub proof_commitment: Bytes,
    pub proof_height: Height,
}
