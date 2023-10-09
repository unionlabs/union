use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::{channel::packet::Packet, client::height::Height},
    CosmosAccountId, MsgIntoProto, TypeUrl,
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

impl MsgIntoProto for MsgAcknowledgement {
    type Proto = protos::ibc::core::channel::v1::MsgAcknowledgement;

    fn into_proto_with_signer(self, signer: &CosmosAccountId) -> Self::Proto {
        protos::ibc::core::channel::v1::MsgAcknowledgement {
            packet: Some(self.packet.into()),
            acknowledgement: self.acknowledgement,
            proof_acked: self.proof_acked,
            proof_height: Some(self.proof_height.into()),
            signer: signer.to_string(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<MsgAcknowledgement> for contracts::ibc_handler::MsgPacketAcknowledgement {
    fn from(value: MsgAcknowledgement) -> Self {
        Self {
            packet: value.packet.into(),
            acknowledgement: value.acknowledgement.into(),
            proof: value.proof_acked.into(),
            proof_height: value.proof_height.into(),
        }
    }
}
