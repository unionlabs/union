use core::fmt;

use serde::{Deserialize, Serialize};
use unionlabs::primitives::{
    Bytes, H256,
    encoding::{Base64, HexUnprefixed},
};

use crate::{
    Amino, PartSetHeader,
    part_set_header::{CanonicalPartSetHeader, canonicalize_part_set_header},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct BlockId {
    /// Hash of the previous block. This is only None on block 1, as the genesis block does not have a hash.
    pub hash: Option<H256<Base64>>,
    #[serde(rename = "parts")]
    pub parts_header: PartSetHeader,
}

impl BlockId {
    /// Source: <https://github.com/gnolang/gno/blob/db1e3ec26c613fd5d119c4466b32c2c0806b2e5c/tm2/pkg/bft/types/block.go#L775>
    pub fn is_zero(&self) -> bool {
        self.hash.is_none() && self.parts_header.is_zero()
    }
}

/// Source: <https://github.com/gnolang/gno/blob/db1e3ec26c613fd5d119c4466b32c2c0806b2e5c/tm2/pkg/bft/types/block.go#L788>
impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(hash) = self.hash {
            // NOTE: The original Go implementation uses upper case hex (%X)
            write!(f, "{}", hash.as_encoding::<HexUnprefixed>())?;
        }

        write!(f, "{}", self.parts_header)?;

        Ok(())
    }
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct CanonicalBlockId {
    #[prost(bytes, tag = "1")]
    pub hash: Vec<u8>,
    /// canonicalization requires fixed size encoding here
    #[prost(message, optional, tag = "2")]
    pub parts_header: Option<CanonicalPartSetHeader>,
}

impl Amino for CanonicalBlockId {
    fn marshal_sized(&self) -> Bytes {
        let mut out = vec![];
        prost::Message::encode(self, &mut out).expect("infallible");
        out.into()
    }
}

pub fn canonicalize_block_id(block_id: &BlockId) -> CanonicalBlockId {
    CanonicalBlockId {
        hash: block_id.hash.map_or(Vec::new(), |hash| hash.into()),
        parts_header: if block_id.parts_header == PartSetHeader::default() {
            None
        } else {
            Some(canonicalize_part_set_header(&block_id.parts_header))
        },
    }
}
