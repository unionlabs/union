use core::num::NonZeroU64;

use macros::model;

use crate::ibc::core::{channel::packet::Packet, client::height::IsHeight};

#[model(proto(raw(protos::ibc::core::channel::v1::MsgRecvPacket)))]
#[serde(bound(
    serialize = "ProofUnreceived: serde::Serialize",
    deserialize = "ProofUnreceived: for<'d> serde::Deserialize<'d>",
))]
pub struct MsgTimeout<ProofUnreceived, ProofHeight: IsHeight> {
    pub packet: Packet,
    pub proof_unreceived: ProofUnreceived,
    pub proof_height: ProofHeight,
    pub next_sequence_recv: NonZeroU64,
}
