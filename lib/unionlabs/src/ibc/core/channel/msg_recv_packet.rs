use macros::proto;
use serde::{Deserialize, Serialize};

use crate::ibc::core::{channel::packet::Packet, client::height::IsHeight};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    bound(
        serialize = "ProofCommitment: Serialize",
        deserialize = "ProofCommitment: for<'d> Deserialize<'d>",
    ),
    deny_unknown_fields
)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[proto(raw = protos::ibc::core::channel::v1::MsgRecvPacket)]
pub struct MsgRecvPacket<ProofCommitment, ProofHeight: IsHeight> {
    pub packet: Packet,
    pub proof_commitment: ProofCommitment,
    pub proof_height: ProofHeight,
}
