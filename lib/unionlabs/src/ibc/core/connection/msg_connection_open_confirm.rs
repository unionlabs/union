use macros::model;

use crate::{bytes::Bytes, ibc::core::client::height::Height, id::ConnectionId};

#[model(proto(raw(protos::ibc::core::connection::v1::MsgConnectionOpenConfirm)))]
pub struct MsgConnectionOpenConfirm {
    pub connection_id: ConnectionId,
    pub proof_ack: Bytes,
    pub proof_height: Height,
}
