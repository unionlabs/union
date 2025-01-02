use serde::{Deserialize, Serialize};
use unionlabs::primitives::{encoding::Base64, Bytes};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProofOp {
    #[serde(rename = "type")]
    pub ty: String,
    pub key: Bytes<Base64>,
    pub data: Bytes<Base64>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::crypto::proof_op::ProofOp;

    impl From<ProofOp> for protos::cometbft::crypto::v1::ProofOp {
        fn from(value: ProofOp) -> Self {
            Self {
                r#type: value.ty,
                key: value.key.to_vec(),
                data: value.data.to_vec(),
            }
        }
    }

    impl From<protos::cometbft::crypto::v1::ProofOp> for ProofOp {
        fn from(value: protos::cometbft::crypto::v1::ProofOp) -> Self {
            Self {
                ty: value.r#type,
                key: value.key.into(),
                data: value.data.into(),
            }
        }
    }
}
