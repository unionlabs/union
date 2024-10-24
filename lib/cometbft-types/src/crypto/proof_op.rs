use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProofOp {
    pub ty: String,
    #[serde(with = "::serde_utils::hex_string")]
    pub key: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    pub data: Vec<u8>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::crypto::proof_op::ProofOp;

    impl From<ProofOp> for protos::tendermint::crypto::ProofOp {
        fn from(value: ProofOp) -> Self {
            Self {
                r#type: value.ty,
                key: value.key,
                data: value.data,
            }
        }
    }

    impl From<protos::tendermint::crypto::ProofOp> for ProofOp {
        fn from(value: protos::tendermint::crypto::ProofOp) -> Self {
            Self {
                ty: value.r#type,
                key: value.key,
                data: value.data,
            }
        }
    }
}
