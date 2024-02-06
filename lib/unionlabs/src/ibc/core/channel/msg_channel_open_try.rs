use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::{channel::channel::Channel, client::height::Height},
    id::PortId,
    TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct MsgChannelOpenTry<ProofInit> {
    pub port_id: PortId,
    pub channel: Channel,
    pub counterparty_version: String,
    pub proof_init: ProofInit,
    pub proof_height: Height,
}

impl TypeUrl for protos::ibc::core::channel::v1::MsgChannelOpenTry {
    const TYPE_URL: &'static str = "/ibc.core.channel.v1.MsgChannelOpenTry";
}
