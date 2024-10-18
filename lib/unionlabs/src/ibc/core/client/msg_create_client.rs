use macros::model;

#[model(proto(raw(protos::ibc::core::client::v1::MsgCreateClient)))]
#[cfg_attr(feature = "valuable", derive(valuable::Valuable))]
pub struct MsgCreateClient {
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub client_state: Vec<u8>,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub consensus_state: Vec<u8>,
}
