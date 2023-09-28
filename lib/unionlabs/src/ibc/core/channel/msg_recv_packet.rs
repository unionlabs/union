use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::{channel::packet::Packet, client::height::Height},
    CosmosAccountId, MsgIntoProto, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgRecvPacket {
    pub packet: Packet,
    #[serde(with = "::serde_utils::hex_string")]
    pub proof_commitment: Vec<u8>,
    pub proof_height: Height,
}

impl TypeUrl for protos::ibc::core::channel::v1::MsgRecvPacket {
    const TYPE_URL: &'static str = "/ibc.core.channel.v1.MsgRecvPacket";
}

impl MsgIntoProto for MsgRecvPacket {
    type Proto = protos::ibc::core::channel::v1::MsgRecvPacket;

    fn into_proto_with_signer(self, signer: &CosmosAccountId) -> Self::Proto {
        protos::ibc::core::channel::v1::MsgRecvPacket {
            packet: Some(self.packet.into()),
            proof_commitment: self.proof_commitment,
            proof_height: Some(self.proof_height.into()),
            signer: signer.to_string(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<MsgRecvPacket> for contracts::ibc_handler::MsgPacketRecv {
    fn from(value: MsgRecvPacket) -> Self {
        Self {
            packet: value.packet.into(),
            proof: value.proof_commitment.into(),
            proof_height: value.proof_height.into(),
        }
    }
}
