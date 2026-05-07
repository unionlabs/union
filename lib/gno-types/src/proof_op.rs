use serde::{Deserialize, Serialize};
use unionlabs::primitives::{Bytes, encoding::Base64};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProofOp {
    #[serde(rename = "type")]
    pub ty: String,
    pub key: Bytes<Base64>,
    pub data: Bytes<Base64>,
}
