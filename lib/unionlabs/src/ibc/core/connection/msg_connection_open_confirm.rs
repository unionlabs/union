use macros::model;
use serde::{Deserialize, Serialize};

use crate::{ibc::core::client::height::IsHeight, id::ConnectionId};

#[model(proto(raw(protos::ibc::core::connection::v1::MsgConnectionOpenConfirm)))]
#[serde(bound(
    serialize = "ProofAck: Serialize",
    deserialize = "ProofAck: for<'d> Deserialize<'d>"
))]
pub struct MsgConnectionOpenConfirm<ProofHeight: IsHeight, ProofAck> {
    pub connection_id: ConnectionId,
    pub proof_ack: ProofAck,
    pub proof_height: ProofHeight,
}
