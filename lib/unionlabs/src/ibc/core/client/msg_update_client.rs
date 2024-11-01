use macros::model;

use crate::{bytes::Bytes, id::ClientId};

#[model(proto(raw(protos::ibc::core::client::v1::MsgUpdateClient)))]
pub struct MsgUpdateClient {
    pub client_id: ClientId,
    pub client_message: Bytes,
}
