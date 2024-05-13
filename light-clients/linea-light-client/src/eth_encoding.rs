use sha3::Digest;
use unionlabs::{hash::H256, uint::U256};

// TODO: move to unionlabs as it can be reused by any chain hosting our EVM IBC

/// Calculates the slot for a `path` at saved in the commitment map in `slot`
///
/// key: keccak256(keccak256(abi.encode_packed(path)) || slot)
pub fn generate_commitment_key(path: &str, slot: U256) -> H256 {
    sha3::Keccak256::new()
        .chain_update(sha3::Keccak256::new().chain_update(path).finalize())
        .chain_update(slot.to_be_bytes())
        .finalize()
        .into()
}
