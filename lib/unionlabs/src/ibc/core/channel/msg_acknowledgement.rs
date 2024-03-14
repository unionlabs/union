use macros::model;
use serde::{Deserialize, Serialize};

use crate::ibc::core::{channel::packet::Packet, client::height::IsHeight};

#[model(proto(raw(protos::ibc::core::channel::v1::MsgAcknowledgement)))]
#[serde(bound(
    serialize = "ProofAcked: Serialize",
    deserialize = "ProofAcked: for<'d> Deserialize<'d>",
))]
pub struct MsgAcknowledgement<ProofAcked, ProofHeight: IsHeight> {
    pub packet: Packet,
    #[serde(with = "::serde_utils::hex_string")]
    pub acknowledgement: Vec<u8>,
    pub proof_acked: ProofAcked,
    pub proof_height: ProofHeight,
}
