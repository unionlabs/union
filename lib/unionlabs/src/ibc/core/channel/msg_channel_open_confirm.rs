use macros::model;
use unionlabs_bytes::Bytes;

use crate::{
    ibc::core::client::height::Height,
    id::{ChannelId, PortId},
};

#[model(proto(raw(protos::ibc::core::channel::v1::MsgChannelOpenConfirm)))]
pub struct MsgChannelOpenConfirm {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub proof_ack: Bytes,
    pub proof_height: Height,
}
