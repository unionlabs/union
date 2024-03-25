use sha2::{Digest, Sha256};
use unionlabs::hash::H256;

const LEAF_PREFIX: &[u8] = &[0];
const INNER_PREFIX: &[u8] = &[1];
const EMPTY_SHA256: H256 = H256(hex_literal::hex!(
    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
));

pub fn calculate_merkle_root<Inner: AsRef<[u8]>>(leaves: &[Inner]) -> H256 {
    if leaves.is_empty() {
        return EMPTY_SHA256;
    }
    // Initial Case:
    // [ LH(1), LH(2), LH(3), LH(4), LH(5) ]
    // First loop:
    // [ IH( LH(1), LH(2) ), IH( LH(3), LH(4) ), LH(5) ]
    // Second loop:
    // [ IH( IH(LH(1), LH(2)) ), IH( IH(LH(3), LH(4)), LH(5) )]
    // ..
    let mut leaves: Vec<H256> = leaves.iter().map(|item| leaf_hash(item.as_ref())).collect();
    let mut leaves_size = leaves.len();
    while leaves_size != 1 {
        let mut i = 0;
        while i < leaves_size {
            if i + 1 < leaves_size {
                leaves[i / 2] = inner_hash(leaves[i].as_ref(), leaves[i + 1].as_ref());
            } else {
                leaves.swap(i / 2, i);
            }
            i += 2;
        }
        leaves_size = i / 2;
    }

    leaves[0]
}

fn inner_hash(left: &[u8], right: &[u8]) -> H256 {
    Sha256::new()
        .chain_update(INNER_PREFIX)
        .chain_update(left)
        .chain_update(right)
        .finalize()
        .into()
}

// returns tm_hash(0x00 || leaf)
fn leaf_hash(leaf: &[u8]) -> H256 {
    Sha256::new()
        .chain_update(LEAF_PREFIX)
        .chain_update(leaf)
        .finalize()
        .into()
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

#[cfg(test)]
mod tests {
    use super::*;

    pub fn calculate_merkle_root_recursive<Inner: AsRef<[u8]>>(leaves: &[Inner]) -> H256 {
        match leaves.len() {
            0 => EMPTY_SHA256,
            1 => leaf_hash(leaves[0].as_ref()),
            len => {
                let largest_power_of_2 = len.next_power_of_two() / 2;
                let left = calculate_merkle_root(&leaves[..largest_power_of_2]);
                let right = calculate_merkle_root(&leaves[largest_power_of_2..]);
                inner_hash(left.as_ref(), right.as_ref())
            }
        }
    }

    #[test]
    fn check_iterative_impl() {
        // odd len
        let leaves = [[1, 2], [3, 4], [5, 6], [7, 8], [9, 10]];
        assert_eq!(
            calculate_merkle_root_recursive(&leaves),
            calculate_merkle_root(&leaves)
        );
        // even len
        let leaves = [[1, 2], [3, 4], [5, 6], [7, 8]];
        assert_eq!(
            calculate_merkle_root_recursive(&leaves),
            calculate_merkle_root(&leaves)
        );
        // 1 elem
        let leaves = [[1, 2]];
        assert_eq!(
            calculate_merkle_root_recursive(&leaves),
            calculate_merkle_root(&leaves)
        );
        // zero elem
        let leaves: &[&[u8]] = &[];
        assert_eq!(
            calculate_merkle_root_recursive(leaves),
            calculate_merkle_root(leaves)
        );
    }
}
