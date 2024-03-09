use macros::proto;
use serde::{Deserialize, Serialize};

use crate::{ibc::core::client::height::IsHeight, id::ConnectionId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    bound(
        serialize = "ProofAck: Serialize",
        deserialize = "ProofAck: for<'d> Deserialize<'d>"
    ),
    deny_unknown_fields
)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[proto(raw = protos::ibc::core::connection::v1::MsgConnectionOpenConfirm)]
pub struct MsgConnectionOpenConfirm<ProofHeight: IsHeight, ProofAck> {
    pub connection_id: ConnectionId,
    pub proof_ack: ProofAck,
    pub proof_height: ProofHeight,
}
