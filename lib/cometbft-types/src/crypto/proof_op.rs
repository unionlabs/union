use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProofOp {
    #[serde(rename = "type")]
    pub ty: String,
    #[serde(with = "::serde_utils::hex_string")]
    pub key: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    pub data: Vec<u8>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::crypto::proof_op::ProofOp;

    impl From<ProofOp> for protos::cometbft::crypto::v1::ProofOp {
        fn from(value: ProofOp) -> Self {
            Self {
                r#type: value.ty,
                key: value.key,
                data: value.data,
            }
        }
    }

    impl From<protos::cometbft::crypto::v1::ProofOp> for ProofOp {
        fn from(value: protos::cometbft::crypto::v1::ProofOp) -> Self {
            Self {
                ty: value.r#type,
                key: value.key,
                data: value.data,
            }
        }
    }
}
