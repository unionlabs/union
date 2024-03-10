use macros::model;
use serde::{Deserialize, Serialize};

use crate::{ibc::core::channel::channel::Channel, id::PortId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[model(proto(raw(protos::ibc::core::channel::v1::MsgChannelOpenInit)))]
pub struct MsgChannelOpenInit {
    pub port_id: PortId,
    pub channel: Channel,
}
