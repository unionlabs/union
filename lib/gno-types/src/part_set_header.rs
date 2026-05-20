use core::fmt;

use serde::{Deserialize, Serialize};
use unionlabs::primitives::{
    Bytes, H256,
    encoding::{Base64, HexUnprefixed},
};

use crate::Amino;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct PartSetHeader {
    #[serde(with = "::serde_utils::string")]
    pub total: u32,
    /// Hash of the previous block. This is only None on block 1, as the genesis block does not have a hash.
    pub hash: Option<H256<Base64>>,
}

impl PartSetHeader {
    /// Source: <https://github.com/gnolang/gno/blob/db1e3ec26c613fd5d119c4466b32c2c0806b2e5c/tm2/pkg/bft/types/part_set.go#L67>
    pub fn is_zero(&self) -> bool {
        self.total == 0 && self.hash.is_none()
    }
}

/// Source: <https://github.com/gnolang/gno/blob/db1e3ec26c613fd5d119c4466b32c2c0806b2e5c/tm2/pkg/bft/types/part_set.go#L63>
impl fmt::Display for PartSetHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:", self.total)?;
        if let Some(hash) = self.hash {
            // NOTE: The original Go implementation uses upper case hex (%X)
            write!(f, "{}", hash.as_encoding::<HexUnprefixed>())?;
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct CanonicalPartSetHeader {
    #[prost(bytes, tag = "1")]
    pub hash: Vec<u8>,
    #[prost(sint32, tag = "2")]
    pub total: i32,
}

impl Amino for CanonicalPartSetHeader {
    fn marshal_sized(&self) -> Bytes {
        let mut out = vec![];
        prost::Message::encode(self, &mut out).expect("infallible");
        out.into()
    }
}

pub fn canonicalize_part_set_header(parts_header: &PartSetHeader) -> CanonicalPartSetHeader {
    CanonicalPartSetHeader {
        hash: parts_header.hash.map_or(Vec::new(), |hash| hash.into()),
        total: parts_header.total as i32,
    }
}
