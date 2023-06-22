use crate::{core::client::height::Height, CosmosAccountId, MsgIntoProto};

pub struct MsgConnectionOpenConfirm {
    pub connection_id: String,
    pub proof_ack: Vec<u8>,
    pub proof_height: Height,
}

impl MsgIntoProto for MsgConnectionOpenConfirm {
    type Proto = protos::ibc::core::connection::v1::MsgConnectionOpenConfirm;

    fn into_proto_with_signer(self, signer: &CosmosAccountId) -> Self::Proto {
        Self::Proto {
            connection_id: self.connection_id,
            proof_ack: self.proof_ack,
            proof_height: Some(self.proof_height.into()),
            signer: signer.to_string(),
        }
    }
}

impl From<MsgConnectionOpenConfirm> for contracts::ibc_handler::MsgConnectionOpenConfirm {
    fn from(msg: MsgConnectionOpenConfirm) -> Self {
        Self {
            connection_id: msg.connection_id,
            proof_ack: msg.proof_ack.into(),
            proof_height: msg.proof_height.into(),
        }
    }
}
