use macros::model;
use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::client::height::IsHeight,
    id::{ChannelId, PortId},
};

#[model(proto(raw(protos::ibc::core::channel::v1::MsgChannelOpenAck)))]
#[serde(bound(
    serialize = "ProofTry: Serialize",
    deserialize = "ProofTry: for<'d> Deserialize<'d>",
))]
pub struct MsgChannelOpenAck<ProofTry, ProofHeight: IsHeight> {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub counterparty_channel_id: ChannelId,
    // yes, this is actually just an unbounded string
    pub counterparty_version: String,
    pub proof_try: ProofTry,
    pub proof_height: ProofHeight,
}
