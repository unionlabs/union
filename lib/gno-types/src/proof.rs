use serde::{Deserialize, Serialize};

use crate::ProofOp;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Proof {
    pub ops: Vec<ProofOp>,
}
