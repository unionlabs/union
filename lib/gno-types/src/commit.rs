use serde::{Deserialize, Serialize};

use crate::{BlockId, Vote};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Commit {
    pub block_id: BlockId,
    pub precommits: Vec<Option<Vote>>,
}
