mod merkle_hasher;
mod merkleize_padded;
mod merkleize_standard;

pub use merkle_hasher::{Error, MerkleHasher};
pub use merkleize_padded::merkleize_padded;
pub use merkleize_standard::merkleize_standard;
use sha2::{digest::FixedOutput, Digest, Sha256};
use smallvec::SmallVec;
use unionlabs_primitives::H256;

pub const BYTES_PER_CHUNK: usize = 32;
pub const HASHSIZE: usize = 32;
pub const MERKLE_HASH_CHUNK: usize = 2 * BYTES_PER_CHUNK;
pub const MAX_UNION_SELECTOR: u8 = 127;
pub const SMALLVEC_SIZE: usize = 32;

pub type PackedEncoding = SmallVec<[u8; SMALLVEC_SIZE]>;

#[must_use]
pub fn hash_fixed(input: &[u8]) -> H256 {
    Sha256::new().chain_update(input).finalize_fixed().into()
}

#[must_use]
pub fn hash_concat(h1: &[u8], h2: &[u8]) -> H256 {
    sha2::Sha256::new()
        .chain_update(h1)
        .chain_update(h2)
        .finalize()
        .into()
}

/// Convenience method for `MerkleHasher` which also provides some fast-paths for small trees.
///
/// `minimum_leaf_count` will only be used if it is greater than or equal to the minimum number of leaves that can be created from `bytes`.
#[must_use]
pub fn merkle_root(bytes: &[u8], minimum_leaf_count: usize) -> H256 {
    let leaves = std::cmp::max(bytes.len().div_ceil(HASHSIZE), minimum_leaf_count);

    if leaves == 0 {
        // If there are no bytes then the hash is always zero.
        H256::default()
    } else if leaves == 1 {
        // If there is only one leaf, the hash is always those leaf bytes padded out to 32-bytes.
        let mut hash = [0; HASHSIZE];
        hash[0..bytes.len()].copy_from_slice(bytes);
        H256::new(hash)
    } else if leaves == 2 {
        // If there are only two leaves (this is common with BLS pubkeys), we can avoid some
        // overhead with `MerkleHasher` and just do a simple 3-node tree here.
        let mut leaves = [0; HASHSIZE * 2];
        leaves[0..bytes.len()].copy_from_slice(bytes);

        hash_fixed(&leaves)
    } else {
        // If there are 3 or more leaves, use `MerkleHasher`.
        let mut hasher = MerkleHasher::with_leaves(leaves);
        hasher
            .write(bytes)
            .expect("the number of leaves is adequate for the number of bytes");
        hasher
            .finish()
            .expect("the number of leaves is adequate for the number of bytes")
    }
}

/// Returns the node created by hashing `root` and `length`.
///
/// Used in `TreeHash` for inserting the length of a list above it's root.
// TODO: NonZeroUsize for length
#[must_use]
pub fn mix_in_length(root: &H256, length: usize) -> H256 {
    let usize_len = std::mem::size_of::<usize>();

    let mut length_bytes = [0; BYTES_PER_CHUNK];
    length_bytes[0..usize_len].copy_from_slice(&length.to_le_bytes());

    hash_concat(root.get(), &length_bytes)
}

/// Returns the node created by hashing `root` and `length`.
///
/// Used in `TreeHash` for inserting the length of a list above it's root.
// TODO: NonZeroUsize for length
#[must_use]
pub fn mix_in_type(root: &H256, type_index: usize) -> H256 {
    let usize_len = std::mem::size_of::<usize>();

    let mut length_bytes = [0; BYTES_PER_CHUNK];
    length_bytes[0..usize_len].copy_from_slice(&type_index.to_le_bytes());

    hash_concat(root.get(), &length_bytes)
}

/// Returns `Some(root)` created by hashing `root` and `selector`, if `selector <=
/// MAX_UNION_SELECTOR`. Otherwise, returns `None`.
///
/// Used in `TreeHash` for the "union" type.
///
/// ## Specification
///
/// ```plaintext
/// mix_in_selector: Given a Merkle root and a type selector selector ("uint256" little-endian
/// serialization) return hash(root + selector).
/// ```
///
/// <https://github.com/ethereum/consensus-specs/blob/v1.1.0-beta.3/ssz/simple-serialize.md#union>
#[must_use]
pub fn mix_in_selector(root: &H256, selector: u8) -> Option<H256> {
    if selector > MAX_UNION_SELECTOR {
        return None;
    }

    let mut chunk = [0; BYTES_PER_CHUNK];
    chunk[0] = selector;

    let root = hash_concat(root.get(), &chunk);
    Some(root)
}

/// The max index that can be used with `ZERO_HASHES`.
pub const ZERO_HASHES_MAX_INDEX: usize = 48;

lazy_static::lazy_static! {
    /// Cached zero hashes where `ZERO_HASHES[i]` is the hash of a Merkle tree with 2^i zero leaves.
    pub static ref ZERO_HASHES: Vec<H256> = {
        let mut hashes = vec![H256::default(); ZERO_HASHES_MAX_INDEX + 1];

        for i in 0..ZERO_HASHES_MAX_INDEX {
            hashes[i + 1] = hash_concat(hashes[i].as_ref(), hashes[i].as_ref());
        }

        hashes
    };
}

/// Returns a cached padding node for a given height.
fn get_zero_hash(height: usize) -> &'static H256 {
    if height <= ZERO_HASHES_MAX_INDEX {
        &ZERO_HASHES[height]
    } else {
        panic!("Tree exceeds MAX_TREE_DEPTH of {}", ZERO_HASHES_MAX_INDEX)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TreeHashType {
    Basic {
        /// Corresponds to `size_of(B)` from [the spec](https://github.com/ethereum/consensus-specs/blob/dev/ssz/simple-serialize.md#merkleization)
        // NOTE: Technically, this can only be one of `1, 2, 4, 8, 16, 32`, encode in types somehow?
        size: u8,
    },
    Vector,
    List,
    Container,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Ssz;

    #[test]
    fn mix_length() {
        let hash = {
            let mut preimage = vec![42; BYTES_PER_CHUNK];
            preimage.append(&mut vec![42]);
            preimage.append(&mut vec![0; BYTES_PER_CHUNK - 1]);
            hash_fixed(&preimage)
        };

        assert_eq!(mix_in_length(&H256::new([42; BYTES_PER_CHUNK]), 42), hash);
    }

    #[test]
    fn bool() {
        let mut true_bytes = <H256>::default();
        true_bytes[0] = 1;

        let false_bytes = <H256>::default();

        assert_eq!(true.tree_hash_root(), true_bytes);
        assert_eq!(false.tree_hash_root(), false_bytes);
    }
}
