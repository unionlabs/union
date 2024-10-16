use macros::model;

use crate::{ibc::core::client::height::Height, id::ConnectionId};

#[model(proto(raw(protos::ibc::core::connection::v1::MsgConnectionOpenConfirm)))]
#[cfg_attr(feature = "valuable", derive(valuable::Valuable))]
pub struct MsgConnectionOpenConfirm {
    pub connection_id: ConnectionId,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub proof_ack: Vec<u8>,
    pub proof_height: Height,
}
