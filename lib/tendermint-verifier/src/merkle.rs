use sha2::{Digest, Sha256};
use unionlabs::hash::H256;

const LEAF_PREFIX: &[u8] = &[0];
const INNER_PREFIX: &[u8] = &[1];

pub fn calculate_merkle_root<Inner: AsRef<[u8]>>(leaves: &[Inner]) -> H256 {
    match leaves.len() {
        0 => empty_hash(),
        1 => leaf_hash_opt(leaves[0].as_ref()),
        len => {
            // TODO(aeryz): make this iterative, we don't know how expensive
            // would this be in wasm context
            let largest_power_of_2 = len.next_power_of_two() / 2;
            let left = calculate_merkle_root(&leaves[..largest_power_of_2]);
            let right = calculate_merkle_root(&leaves[largest_power_of_2..]);
            inner_hash_opt(left.as_ref(), right.as_ref())
        }
    }
}

fn inner_hash_opt(left: &[u8], right: &[u8]) -> H256 {
    H256(
        Sha256::new()
            .chain_update(INNER_PREFIX)
            .chain_update(left)
            .chain_update(right)
            .finalize()
            .into(),
    )
}

// TODO(aeryz): Make this const
fn empty_hash() -> H256 {
    H256(Sha256::new().chain_update(&[]).finalize().into())
}

// returns tmhash(0x00 || leaf)
fn leaf_hash_opt(leaf: &[u8]) -> H256 {
    H256(
        Sha256::new()
            .chain_update(LEAF_PREFIX)
            .chain_update(leaf)
            .finalize()
            .into(),
    )
}

#[test]
fn test_hash() {
    use hex_literal::hex;
    let leaves = [[1, 2], [3, 4], [5, 6], [7, 8], [9, 10]];

    assert_eq!(
        calculate_merkle_root(&leaves),
        H256(hex!(
            "f326493eceab4f2d9ffbc78c59432a0a005d6ea98392045c74df5d14a113be18"
        ))
    );
}
