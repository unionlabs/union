use crate::{ibc::core::channel::channel::Channel, CosmosAccountId, MsgIntoProto};

#[derive(Debug, Clone)]
pub struct MsgChannelOpenInit {
    pub port_id: String,
    pub channel: Channel,
}

#[derive(Debug, Clone)]
pub struct MsgChannelOpenInitResponse {
    pub channel_id: String,
    pub version: String,
}

impl MsgIntoProto for MsgChannelOpenInit {
    type Proto = protos::ibc::core::channel::v1::MsgChannelOpenInit;

    fn into_proto_with_signer(self, signer: &CosmosAccountId) -> Self::Proto {
        Self::Proto {
            port_id: self.port_id,
            channel: Some(self.channel.into()),
            signer: signer.to_string(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<MsgChannelOpenInit> for contracts::ibc_handler::MsgChannelOpenInit {
    fn from(msg: MsgChannelOpenInit) -> Self {
        Self {
            port_id: msg.port_id,
            channel: msg.channel.into(),
        }
    }
}
