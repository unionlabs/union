use macros::model;

use crate::{id::ClientId, primitives::Bytes};

#[model(proto(raw(protos::ibc::core::client::v1::MsgUpdateClient)))]
pub struct MsgUpdateClient {
    pub client_id: ClientId,
    pub client_message: Bytes,
}
