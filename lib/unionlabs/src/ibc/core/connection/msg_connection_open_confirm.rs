use serde::{Deserialize, Serialize};

use crate::{ibc::core::client::height::IsHeight, id::ConnectionId, TypeUrl};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgConnectionOpenConfirm<ProofHeight: IsHeight> {
    pub connection_id: ConnectionId,
    #[serde(with = "::serde_utils::hex_string")]
    pub proof_ack: Vec<u8>,
    pub proof_height: ProofHeight,
}

impl TypeUrl for protos::ibc::core::connection::v1::MsgConnectionOpenConfirm {
    const TYPE_URL: &'static str = "/ibc.core.connection.v1.MsgConnectionOpenConfirm";
}
