use macros::model;

#[model(proto(raw(protos::ibc::core::client::v1::MsgUpdateClient)))]
pub struct MsgUpdateClient<ClientId, Header> {
    pub client_id: ClientId,
    pub client_message: Header,
}
