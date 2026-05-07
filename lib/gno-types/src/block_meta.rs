use serde::{Deserialize, Serialize};

use crate::{BlockId, Header};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockMeta {
    pub block_id: BlockId,
    pub header: Header,
}
