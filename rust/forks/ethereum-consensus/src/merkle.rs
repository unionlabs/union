use crate::{beacon::Root, errors::MerkleError, internal_prelude::*, types::H256};
use sha2::{Digest, Sha256};

/// Check if ``leaf`` at ``index`` verifies against the Merkle ``root`` and ``branch``.
/// https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#is_valid_merkle_branch
pub fn is_valid_merkle_branch(
    leaf: H256,
    branch: &[H256],
    index: u64,
    root: Root,
) -> Result<(), MerkleError> {
    let mut value = leaf.clone();
    for (i, b) in branch.iter().enumerate() {
        if let Some(v) = 2u64.checked_pow(i as u32) {
            if index / v % 2 == 1 {
                value = hash([b.as_bytes(), value.as_bytes()].concat());
            } else {
                value = hash([value.as_bytes(), b.as_bytes()].concat());
            }
        } else {
            return Err(MerkleError::TooLongMerkleBranch(
                leaf,
                branch.to_vec(),
                index,
                root,
            ));
        }
    }
    if value == root {
        Ok(())
    } else {
        Err(MerkleError::InvalidMerkleBranch(
            leaf,
            branch.to_vec(),
            index,
            root,
        ))
    }
}

fn hash(bz: Vec<u8>) -> H256 {
    let mut output = H256::default();
    output.0.copy_from_slice(Sha256::digest(bz).as_slice());
    output
}
