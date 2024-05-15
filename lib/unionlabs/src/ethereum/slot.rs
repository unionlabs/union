use alloc::borrow::Cow;

use ethers::utils::keccak256;
use sha2::Digest;
use sha3::Keccak256;

use crate::{hash::H256, uint::U256};

pub enum Slot<'data, 'slot> {
    Array(U256),
    Mapping(MappingKey<'data>, &'slot Slot<'data, 'slot>),
    Offset(U256),
}

impl<'data, 'slot> Slot<'data, 'slot> {
    // https://docs.soliditylang.org/en/latest/internals/layout_in_storage.html#mappings-and-dynamic-arrays
    #[must_use]
    pub fn slot(&self) -> U256 {
        match self {
            // keccak256(p)
            Slot::Array(p) => U256::from_be_bytes(keccak256(p.to_be_bytes())),
            // keccak256(h(k) . p)
            Slot::Mapping(k, p) => U256::from_be_bytes(
                Keccak256::new()
                    .chain_update(k.encode().to_be_bytes())
                    .chain_update(p.slot().to_be_bytes())
                    .finalize()
                    .into(),
            ),
            Slot::Offset(p) => *p,
        }
    }

    #[must_use]
    pub fn slot_with_offset(&self, offset: U256) -> U256 {
        self.slot() + offset
    }
}

pub enum MappingKey<'data> {
    String(&'data str),
    Uint256(U256),
    Bytes32(H256),
}

impl<'data> MappingKey<'data> {
    #[must_use]
    fn encode(&self) -> U256 {
        match self {
            Self::String(string) => U256::from_be_bytes(keccak256(string)),
            Self::Uint256(k) => *k,
            Self::Bytes32(k) => U256::from_be_bytes(k.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_calculation() {
        let slot = Slot::Mapping(MappingKey::Uint256(1.into()), &Slot::Offset(0.into()));
    }
}
