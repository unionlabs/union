use macros::model;

use crate::tendermint::crypto::proof_op::ProofOp;

#[model(proto(raw(protos::tendermint::crypto::ProofOps), into, from))]
pub struct ProofOps {
    pub ops: Vec<ProofOp>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::tendermint::crypto::proof_ops::ProofOps;

    impl From<ProofOps> for protos::tendermint::crypto::ProofOps {
        fn from(value: ProofOps) -> Self {
            Self {
                ops: value.ops.into_iter().map(Into::into).collect(),
            }
        }
    }

    impl From<protos::tendermint::crypto::ProofOps> for ProofOps {
        fn from(value: protos::tendermint::crypto::ProofOps) -> Self {
            Self {
                ops: value.ops.into_iter().map(Into::into).collect(),
            }
        }
    }
}
