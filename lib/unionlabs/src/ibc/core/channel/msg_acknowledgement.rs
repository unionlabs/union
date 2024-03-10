use macros::model;
use serde::{Deserialize, Serialize};

use crate::ibc::core::{channel::packet::Packet, client::height::IsHeight};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    bound(
        serialize = "ProofAcked: Serialize",
        deserialize = "ProofAcked: for<'d> Deserialize<'d>",
    ),
    deny_unknown_fields
)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[model(proto(raw(protos::ibc::core::channel::v1::MsgAcknowledgement)))]
pub struct MsgAcknowledgement<ProofAcked, ProofHeight: IsHeight> {
    pub packet: Packet,
    #[serde(with = "::serde_utils::hex_string")]
    pub acknowledgement: Vec<u8>,
    pub proof_acked: ProofAcked,
    pub proof_height: ProofHeight,
}
