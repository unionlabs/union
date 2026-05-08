use core::{cmp::min, fmt};

use serde::{Deserialize, Serialize};
use unionlabs::primitives::{
    FixedBytes, H256,
    encoding::{Base64, HexUnprefixed},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
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
            // NOTE: The orginal Go implementation uses upper case hex (%X)
            write!(f, "{}", hash.as_encoding::<HexUnprefixed>())?;
        }
        Ok(())
    }
}

/// Source: <https://github.com/gnolang/gno/blob/db1e3ec26c613fd5d119c4466b32c2c0806b2e5c/tm2/pkg/bft/types/fingerprint.go#L6>
fn fingerprint(slice: impl AsRef<[u8]>) -> FixedBytes<6> {
    let mut fingerprint = FixedBytes::default();
    let end = min(6, slice.as_ref().len());
    fingerprint[0..end].copy_from_slice(slice.as_ref());
    return fingerprint;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_works() {
        for (i, o) in [
            (hex!(""), hex!("000000000000")),
            (hex!("0102"), hex!("010200000000")),
            (hex!("010203010203"), hex!("010203010203")),
            (hex!("010203010203AA"), hex!("010203010203AA")),
            (hex!("010203010203AAAAAA"), hex!("010203010203AAAAAA")),
        ] {
            assert_eq!(fingerprint(i), o);
        }
    }
}
