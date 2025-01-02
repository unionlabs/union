use macros::model;
use unionlabs_bytes::Bytes;

#[model(proto(raw(protos::ibc::core::client::v1::MsgCreateClient)))]
pub struct MsgCreateClient {
    pub client_state: Bytes,
    pub consensus_state: Bytes,
}
