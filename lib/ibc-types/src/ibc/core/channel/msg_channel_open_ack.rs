use crate::{ibc::core::client::height::Height, CosmosAccountId, MsgIntoProto};

#[derive(Debug, Clone)]
pub struct MsgChannelOpenAck {
    pub port_id: String,
    pub channel_id: String,
    pub counterparty_channel_id: String,
    pub counterparty_version: String,
    pub proof_try: Vec<u8>,
    pub proof_height: Height,
}

impl MsgIntoProto for MsgChannelOpenAck {
    type Proto = protos::ibc::core::channel::v1::MsgChannelOpenAck;

    fn into_proto_with_signer(self, signer: &CosmosAccountId) -> Self::Proto {
        Self::Proto {
            port_id: self.port_id,
            channel_id: self.channel_id,
            counterparty_version: self.counterparty_version,
            counterparty_channel_id: self.counterparty_channel_id,
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
            port_id: msg.port_id,
            channel_id: msg.channel_id,
            counterparty_version: msg.counterparty_version,
            counterparty_channel_id: msg.counterparty_channel_id,
            proof_try: msg.proof_try.into(),
            proof_height: msg.proof_height.into(),
        }
    }
}
