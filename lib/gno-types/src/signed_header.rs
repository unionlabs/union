use serde::{Deserialize, Serialize};

use crate::{Commit, Header};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct SignedHeader {
    pub header: Header,
    pub commit: Commit,
}
