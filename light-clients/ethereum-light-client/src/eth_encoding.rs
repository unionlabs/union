use sha3::Digest;
use unionlabs::{hash::H256, uint::U256};

/// Calculates the slot for a `path` at saved in the commitment map in `slot`
///
/// key: keccak256(keccak256(abi.encode_packed(path)) || slot)
pub fn generate_commitment_key(path: String, slot: U256) -> H256 {
    sha3::Keccak256::new()
        .chain_update(
            sha3::Keccak256::new()
                .chain_update(path.as_bytes())
                .finalize(),
        )
        .chain_update(slot.to_big_endian())
        .finalize()
        .into()
}
