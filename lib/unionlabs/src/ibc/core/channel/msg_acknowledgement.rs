use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::{channel::packet::Packet, client::height::Height},
    TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgAcknowledgement {
    pub packet: Packet,
    #[serde(with = "::serde_utils::hex_string")]
    pub acknowledgement: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    pub proof_acked: Vec<u8>,
    pub proof_height: Height,
}

impl TypeUrl for protos::ibc::core::channel::v1::MsgAcknowledgement {
    const TYPE_URL: &'static str = "/ibc.core.channel.v1.MsgAcknowledgement";
}
