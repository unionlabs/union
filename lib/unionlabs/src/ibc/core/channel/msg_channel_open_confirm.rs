use macros::model;

use crate::{
    ibc::core::client::height::Height,
    id::{ChannelId, PortId},
};

#[model(proto(raw(protos::ibc::core::channel::v1::MsgChannelOpenConfirm)))]
#[cfg_attr(feature = "valuable", derive(valuable::Valuable))]
pub struct MsgChannelOpenConfirm {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub proof_ack: Vec<u8>,
    pub proof_height: Height,
}
