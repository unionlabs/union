use macros::model;
use crate::primitives::Bytes;

#[model(proto(raw(protos::ibc::core::client::v1::MsgCreateClient)))]
pub struct MsgCreateClient {
    pub client_state: Bytes,
    pub consensus_state: Bytes,
}
