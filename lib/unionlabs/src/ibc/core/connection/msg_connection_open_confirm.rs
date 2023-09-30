use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::client::height::IsHeight, id::ConnectionId, CosmosAccountId, MsgIntoProto, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MsgConnectionOpenConfirm<ProofHeight: IsHeight> {
    pub connection_id: ConnectionId,
    pub proof_ack: Vec<u8>,
    pub proof_height: ProofHeight,
}

impl TypeUrl for protos::ibc::core::connection::v1::MsgConnectionOpenConfirm {
    const TYPE_URL: &'static str = "/ibc.core.connection.v1.MsgConnectionOpenConfirm";
}

impl<ProofHeight: IsHeight> MsgIntoProto for MsgConnectionOpenConfirm<ProofHeight> {
    type Proto = protos::ibc::core::connection::v1::MsgConnectionOpenConfirm;

    fn into_proto_with_signer(self, signer: &CosmosAccountId) -> Self::Proto {
        Self::Proto {
            connection_id: self.connection_id.to_string(),
            proof_ack: self.proof_ack,
            proof_height: Some(self.proof_height.into_height().into()),
            signer: signer.to_string(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl<ProofHeight: IsHeight> From<MsgConnectionOpenConfirm<ProofHeight>>
    for contracts::ibc_handler::MsgConnectionOpenConfirm
{
    fn from(msg: MsgConnectionOpenConfirm<ProofHeight>) -> Self {
        Self {
            connection_id: msg.connection_id.to_string(),
            proof_ack: msg.proof_ack.into(),
            proof_height: msg.proof_height.into_height().into(),
        }
    }
}
