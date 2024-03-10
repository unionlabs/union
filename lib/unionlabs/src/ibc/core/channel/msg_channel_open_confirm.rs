use macros::model;
use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::client::height::Height,
    id::{ChannelId, PortId},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[model(proto(raw(protos::ibc::core::channel::v1::MsgChannelOpenConfirm)))]
pub struct MsgChannelOpenConfirm<ProofAck> {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub proof_ack: ProofAck,
    pub proof_height: Height,
}
