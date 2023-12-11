use serde::{Deserialize, Serialize};

use crate::{ibc::core::client::height::IsHeight, id::ConnectionId, TypeUrl};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "
        ProofAck: Serialize,
    ",
    deserialize = "
        ProofAck: for<'d> Deserialize<'d>,
    "
))]
pub struct MsgConnectionOpenConfirm<ProofHeight: IsHeight, ProofAck> {
    pub connection_id: ConnectionId,
    pub proof_ack: ProofAck,
    pub proof_height: ProofHeight,
}

impl TypeUrl for protos::ibc::core::connection::v1::MsgConnectionOpenConfirm {
    const TYPE_URL: &'static str = "/ibc.core.connection.v1.MsgConnectionOpenConfirm";
}
