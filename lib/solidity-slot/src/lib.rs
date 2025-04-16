//! This crate provides an implementation of the [Solidity storage layout][layout].
//!
//! [layout]: https://docs.soliditylang.org/en/latest/internals/layout_in_storage.html

#![warn(clippy::pedantic)] // TODO: missing_docs
#![no_std]

use sha3::{Digest, Keccak256};
pub use unionlabs_primitives::{H256, U256};

/// Solidity storage slot calculations. Note that this currently does not handle dynamic arrays with packed values; the index passed to [`Slot::Array`] will need to be calculated manually in this case.
pub enum Slot<'a> {
    /// (base slot, index)
    Array(&'a Slot<'a>, U256),
    /// (base slot, size, index)
    StructArray(&'a Slot<'a>, u32, U256),
    /// (base slot, mapping key)
    Mapping(&'a Slot<'a>, MappingKey<'a>),
    Offset(U256),
    /// (base slot, offset)
    StructOffset(&'a Slot<'a>, U256),
}

impl Slot<'_> {
    // https://docs.soliditylang.org/en/latest/internals/layout_in_storage.html#mappings-and-dynamic-arrays
    #[inline]
    #[must_use = "calculating the slot has no effect"]
    // REVIEW: Make const? <https://crates.io/crates/keccak-const/0.2.0>
    pub fn slot(&self) -> U256 {
        match self {
            // keccak256(p)
            Slot::StructArray(p, size, idx) => {
                U256::from_be_bytes(*keccak256(p.slot().to_be_bytes()).get())
                    + U256::from(*size) * *idx
            }
            Slot::Array(p, idx) => Slot::StructArray(p, 1, *idx).slot(),
            // keccak256(h(k) . p)
            Slot::Mapping(p, k) => {
                let mut hasher = Keccak256::new();

                match &k {
                    MappingKey::String(string) => hasher.update(string.as_bytes()),
                    MappingKey::Uint256(k) => hasher.update(k.to_be_bytes()),
                    MappingKey::Uint64(k) => hasher.update(U256::from(*k).to_be_bytes()),
                    MappingKey::Bytes32(k) => hasher.update(k.get()),
                }

                U256::from_be_bytes(
                    hasher
                        .chain_update(p.slot().to_be_bytes())
                        .finalize()
                        .into(),
                )
            }
            Slot::Offset(p) => *p,
            Slot::StructOffset(base, offset) => base.slot() + *offset,
        }
    }
}

pub enum MappingKey<'a> {
    String(&'a str),
    Uint256(U256),
    Uint64(u64),
    Bytes32(H256),
}

#[inline]
#[must_use]
pub fn keccak256(bytes: impl AsRef<[u8]>) -> H256 {
    Keccak256::new().chain_update(bytes).finalize().into()
}

#[test]
fn test() {
    // Test contract uploaded here: https://sepolia.etherscan.io/address/0x6845dbaa9513d3d07737ea9f6e350011dcfeb9bd

    // mapping(uint256 => mapping(uint256 => uint256)[])
    let slot = Slot::Mapping(
        &Slot::Array(
            &Slot::Mapping(
                &Slot::Offset(0u32.into()),
                MappingKey::Uint256(123u32.into()),
            ),
            1u32.into(),
        ),
        MappingKey::Uint256(100u32.into()),
    )
    .slot();

    assert_eq!(
        <H256>::new(slot.to_be_bytes()),
        <H256>::new(hex_literal::hex!(
            "00a9b48fe93e5d10ebc2d9021d1477088c6292bf047876944343f57fdf3f0467"
        ))
    );
}
