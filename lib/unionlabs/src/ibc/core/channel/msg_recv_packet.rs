use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::{channel::packet::Packet, client::height::Height},
    TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgRecvPacket<ProofCommitment> {
    pub packet: Packet,
    pub proof_commitment: ProofCommitment,
    pub proof_height: Height,
}

impl TypeUrl for protos::ibc::core::channel::v1::MsgRecvPacket {
    const TYPE_URL: &'static str = "/ibc.core.channel.v1.MsgRecvPacket";
}
