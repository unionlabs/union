use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::client::height::Height,
    id::{ChannelId, PortId},
    TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgChannelOpenConfirm<ProofAck> {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub proof_ack: ProofAck,
    pub proof_height: Height,
}

impl TypeUrl for protos::ibc::core::channel::v1::MsgChannelOpenConfirm {
    const TYPE_URL: &'static str = "/ibc.core.channel.v1.MsgChannelOpenConfirm";
}
