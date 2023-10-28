use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::channel::channel::Channel, id::PortId, CosmosAccountId, MsgIntoProto, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgChannelOpenInit {
    pub port_id: PortId,
    pub channel: Channel,
}

impl TypeUrl for protos::ibc::core::channel::v1::MsgChannelOpenInit {
    const TYPE_URL: &'static str = "/ibc.core.channel.v1.MsgChannelOpenInit";
}

impl MsgIntoProto for MsgChannelOpenInit {
    type Proto = protos::ibc::core::channel::v1::MsgChannelOpenInit;

    fn into_proto_with_signer(self, signer: &CosmosAccountId) -> Self::Proto {
        Self::Proto {
            port_id: self.port_id.to_string(),
            channel: Some(self.channel.into()),
            signer: signer.to_string(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<MsgChannelOpenInit> for contracts::ibc_handler::MsgChannelOpenInit {
    fn from(msg: MsgChannelOpenInit) -> Self {
        Self {
            port_id: msg.port_id.to_string(),
            channel: msg.channel.into(),
        }
    }
}
