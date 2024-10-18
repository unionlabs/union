use macros::model;

use crate::id::ClientId;

#[model(proto(raw(protos::ibc::core::client::v1::MsgUpdateClient)))]
#[cfg_attr(feature = "valuable", derive(valuable::Valuable))]
pub struct MsgUpdateClient {
    pub client_id: ClientId,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub client_message: Vec<u8>,
}
