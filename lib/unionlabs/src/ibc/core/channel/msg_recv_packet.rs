use macros::model;
use serde::{Deserialize, Serialize};

use crate::ibc::core::{channel::packet::Packet, client::height::IsHeight};

#[model(proto(raw(protos::ibc::core::channel::v1::MsgRecvPacket)))]
#[serde(bound(
    serialize = "ProofCommitment: Serialize",
    deserialize = "ProofCommitment: for<'d> Deserialize<'d>",
))]
pub struct MsgRecvPacket<ProofCommitment, ProofHeight: IsHeight> {
    pub packet: Packet,
    pub proof_commitment: ProofCommitment,
    pub proof_height: ProofHeight,
}
