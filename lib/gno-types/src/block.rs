use serde::{Deserialize, Serialize};

use crate::{Commit, Data, Header};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Block {
    pub header: Header,
    pub data: Data,
    pub last_commit: Commit,
}
