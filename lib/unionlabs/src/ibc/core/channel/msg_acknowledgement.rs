use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::{channel::packet::Packet, client::height::IsHeight},
    TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    bound(
        serialize = "ProofAcked: Serialize",
        deserialize = "ProofAcked: for<'d> Deserialize<'d>",
    ),
    deny_unknown_fields
)]
pub struct MsgAcknowledgement<ProofAcked, ProofHeight: IsHeight> {
    pub packet: Packet,
    #[serde(with = "::serde_utils::hex_string")]
    pub acknowledgement: Vec<u8>,
    pub proof_acked: ProofAcked,
    pub proof_height: ProofHeight,
}

impl TypeUrl for protos::ibc::core::channel::v1::MsgAcknowledgement {
    const TYPE_URL: &'static str = "/ibc.core.channel.v1.MsgAcknowledgement";
}
