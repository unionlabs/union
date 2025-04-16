use typenum::Unsigned;
use unionlabs_primitives::H256;

use crate::{
    tree_hash::{MerkleHasher, TreeHashType, BYTES_PER_CHUNK},
    Ssz,
};

/// A helper function providing common functionality between the `TreeHash` implementations for
/// `Vector` and `List`.
pub fn vec_tree_hash_root<T, N>(vec: &[T]) -> H256
where
    T: Ssz,
    N: Unsigned,
{
    match T::TREE_HASH_TYPE {
        TreeHashType::Basic { size } => {
            let mut hasher = MerkleHasher::with_leaves(chunk_count_basic_list_or_vector::<N>(size));

            for item in vec {
                hasher
                    .write(&item.tree_hash_root()[..(size as usize)])
                    .expect("ssz::types variable vec should not contain more elements than max");
            }

            hasher
                .finish()
                .expect("ssz::types variable vec should not have a remaining buffer")
        }
        TreeHashType::Container | TreeHashType::List | TreeHashType::Vector => {
            let mut hasher = MerkleHasher::with_leaves(N::USIZE);

            for item in vec {
                hasher
                    .write(item.tree_hash_root().get())
                    .expect("ssz::types vec should not contain more elements than max");
            }

            hasher
                .finish()
                .expect("ssz::types vec should not have a remaining buffer")
        }
    }
}

/// Corresponds to `chunk_count(type)` definition for `List[B, N]` and `Vector[B, N]` from [the spec](https://github.com/ethereum/consensus-specs/blob/dev/ssz/simple-serialize.md#merkleization).
#[inline]
fn chunk_count_basic_list_or_vector<N>(size: u8) -> usize
where
    N: Unsigned,
{
    (N::USIZE * (size as usize)).div_ceil(32_usize)
}

/// A helper function providing common functionality for finding the Merkle root of some bytes that
/// represent a bitfield.
#[must_use]
pub fn bitfield_bytes_tree_hash_root<N: Unsigned>(bytes: &[u8]) -> H256 {
    let byte_size = N::USIZE.div_ceil(8);
    let leaf_count = byte_size.div_ceil(BYTES_PER_CHUNK);

    let mut hasher = MerkleHasher::with_leaves(leaf_count);

    hasher
        .write(bytes)
        .expect("bitfield should not exceed tree hash leaf limit");

    hasher
        .finish()
        .expect("bitfield tree hash buffer should not exceed leaf limit")
}
