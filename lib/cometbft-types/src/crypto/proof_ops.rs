use serde::{Deserialize, Serialize};

use crate::crypto::proof_op::ProofOp;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProofOps {
    pub ops: Vec<ProofOp>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::crypto::proof_ops::ProofOps;

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
}
