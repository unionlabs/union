use macros::model;
use unionlabs_bytes::Bytes;

use crate::ibc::core::{channel::packet::Packet, client::height::Height};

#[model(proto(raw(protos::ibc::core::channel::v1::MsgAcknowledgement)))]
pub struct MsgAcknowledgement {
    pub packet: Packet,
    pub acknowledgement: Bytes,
    pub proof_acked: Bytes,
    pub proof_height: Height,
}
