use serde::{Deserialize, Serialize};

use crate::{ethereum::H256, tendermint::types::canonical_block_header::CanonicalPartSetHeader};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalBlockId {
    pub hash: H256,
    pub part_set_header: CanonicalPartSetHeader,
}
