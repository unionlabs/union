use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::client::height::IsHeight,
    id::{ChannelId, PortId},
    TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    bound(
        serialize = "ProofTry: Serialize",
        deserialize = "ProofTry: for<'d> Deserialize<'d>",
    ),
    deny_unknown_fields
)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct MsgChannelOpenAck<ProofTry, ProofHeight: IsHeight> {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub counterparty_channel_id: ChannelId,
    // yes, this is actually just an unbounded string
    pub counterparty_version: String,
    pub proof_try: ProofTry,
    pub proof_height: ProofHeight,
}

impl TypeUrl for protos::ibc::core::channel::v1::MsgChannelOpenAck {
    const TYPE_URL: &'static str = "/ibc.core.channel.v1.MsgChannelOpenAck";
}
