use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::client::height::Height,
    id::{ChannelId, PortId},
    CosmosAccountId, MsgIntoProto, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgChannelOpenAck {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub counterparty_channel_id: ChannelId,
    // yes, this is actually just an unbounded string
    pub counterparty_version: String,
    #[serde(with = "::serde_utils::hex_string")]
    pub proof_try: Vec<u8>,
    pub proof_height: Height,
}

impl TypeUrl for protos::ibc::core::channel::v1::MsgChannelOpenAck {
    const TYPE_URL: &'static str = "/ibc.core.channel.v1.MsgChannelOpenAck";
}

impl MsgIntoProto for MsgChannelOpenAck {
    type Proto = protos::ibc::core::channel::v1::MsgChannelOpenAck;

    fn into_proto_with_signer(self, signer: &CosmosAccountId) -> Self::Proto {
        Self::Proto {
            port_id: self.port_id.to_string(),
            channel_id: self.channel_id.to_string(),
            counterparty_version: self.counterparty_version,
            counterparty_channel_id: self.counterparty_channel_id.to_string(),
            proof_try: self.proof_try,
            proof_height: Some(self.proof_height.into()),
            signer: signer.to_string(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<MsgChannelOpenAck> for contracts::ibc_handler::MsgChannelOpenAck {
    fn from(msg: MsgChannelOpenAck) -> Self {
        Self {
            port_id: msg.port_id.to_string(),
            channel_id: msg.channel_id.to_string(),
            counterparty_version: msg.counterparty_version,
            counterparty_channel_id: msg.counterparty_channel_id.to_string(),
            proof_try: msg.proof_try.into(),
            proof_height: msg.proof_height.into(),
        }
    }
}
