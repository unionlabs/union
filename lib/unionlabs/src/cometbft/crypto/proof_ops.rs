use macros::model;

use crate::cometbft::crypto::proof_op::ProofOp;

#[model(proto(raw(protos::cometbft::crypto::v1::ProofOps), into, from))]
pub struct ProofOps {
    pub ops: Vec<ProofOp>,
}

impl From<ProofOps> for protos::cometbft::crypto::v1::ProofOps {
    fn from(value: ProofOps) -> Self {
        Self {
            ops: value.ops.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<protos::cometbft::crypto::v1::ProofOps> for ProofOps {
    fn from(value: protos::cometbft::crypto::v1::ProofOps) -> Self {
        Self {
            ops: value.ops.into_iter().map(Into::into).collect(),
        }
    }
}
