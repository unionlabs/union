use macros::model;
use unionlabs_bytes::Bytes;

use crate::id::ClientId;

#[model(proto(raw(protos::ibc::core::client::v1::MsgUpdateClient)))]
pub struct MsgUpdateClient {
    pub client_id: ClientId,
    pub client_message: Bytes,
}
