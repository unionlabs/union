use serde::{Deserialize, Serialize};

use crate::ethereum::H256;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalPartSetHeader {
    pub total: u32,
    pub hash: H256,
}
