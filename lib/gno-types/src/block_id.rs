use core::fmt;

use serde::{Deserialize, Serialize};
use unionlabs::primitives::{
    H256,
    encoding::{Base64, HexUnprefixed},
};

use crate::PartSetHeader;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct BlockId {
    /// Hash of the previous block. This is only None on block 1, as the genesis block does not have a hash.
    pub hash: Option<H256<Base64>>,
    #[serde(rename = "parts")]
    pub part_set_header: PartSetHeader,
}

impl BlockId {
    /// Source: <https://github.com/gnolang/gno/blob/db1e3ec26c613fd5d119c4466b32c2c0806b2e5c/tm2/pkg/bft/types/block.go#L775>
    pub fn is_zero(&self) -> bool {
        self.hash.is_none() && self.part_set_header.is_zero()
    }
}

/// Source: <https://github.com/gnolang/gno/blob/db1e3ec26c613fd5d119c4466b32c2c0806b2e5c/tm2/pkg/bft/types/block.go#L788>
impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(hash) = self.hash {
            // NOTE: The orginal Go implementation uses upper case hex (%X)
            write!(f, "{}", hash.as_encoding::<HexUnprefixed>())?;
        }

        write!(f, "{}", self.part_set_header)?;

        Ok(())
    }
}
