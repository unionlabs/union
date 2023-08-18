use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::{channel::channel::Channel, client::height::Height},
    CosmosAccountId, MsgIntoProto, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgChannelOpenTry {
    pub port_id: String,
    pub channel: Channel,
    pub counterparty_version: String,
    pub proof_init: Vec<u8>,
    pub proof_height: Height,
}

impl TypeUrl for protos::ibc::core::channel::v1::MsgChannelOpenTry {
    const TYPE_URL: &'static str = "/ibc.core.channel.v1.MsgChannelOpenTry";
}

#[derive(Debug, Clone)]
pub struct MsgChannelOpenTryResponse {
    pub version: String,
    pub channel_id: String,
}

impl MsgIntoProto for MsgChannelOpenTry {
    type Proto = protos::ibc::core::channel::v1::MsgChannelOpenTry;

    fn into_proto_with_signer(self, signer: &CosmosAccountId) -> Self::Proto {
        #[allow(deprecated)]
        protos::ibc::core::channel::v1::MsgChannelOpenTry {
            port_id: self.port_id,
            channel: Some(self.channel.into()),
            counterparty_version: self.counterparty_version,
            proof_init: self.proof_init,
            proof_height: Some(self.proof_height.into()),
            previous_channel_id: String::new(),
            signer: signer.to_string(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<MsgChannelOpenTry> for contracts::ibc_handler::MsgChannelOpenTry {
    fn from(msg: MsgChannelOpenTry) -> Self {
        Self {
            port_id: msg.port_id,
            channel: msg.channel.into(),
            counterparty_version: msg.counterparty_version,
            proof_init: msg.proof_init.into(),
            proof_height: msg.proof_height.into(),
        }
    }
}
