use macros::model;

use crate::{ibc::core::client::height::Height, id::ConnectionId, primitives::Bytes};

#[model(proto(raw(protos::ibc::core::connection::v1::MsgConnectionOpenConfirm)))]
pub struct MsgConnectionOpenConfirm {
    pub connection_id: ConnectionId,
    pub proof_ack: Bytes,
    pub proof_height: Height,
}
