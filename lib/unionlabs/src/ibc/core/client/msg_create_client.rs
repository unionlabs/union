use macros::model;

#[model(proto(raw(protos::ibc::core::client::v1::MsgCreateClient)))]
pub struct MsgCreateClient {
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub client_state: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub consensus_state: Vec<u8>,
}
