use tree_hash::{Hash256, MerkleHasher, TreeHash, TreeHashType, BYTES_PER_CHUNK};
use typenum::Unsigned;

/// A helper function providing common functionality between the `TreeHash` implementations for
/// `FixedVector` and `VariableList`.
pub fn vec_tree_hash_root<T, N>(vec: &[T]) -> Hash256
where
    T: TreeHash,
    N: Unsigned,
{
    match T::tree_hash_type() {
        TreeHashType::Basic => {
            let mut hasher = MerkleHasher::with_leaves(
                (N::to_usize() + T::tree_hash_packing_factor() - 1) / T::tree_hash_packing_factor(),
            );

            for item in vec {
                hasher
                    .write(&item.tree_hash_packed_encoding())
                    .expect("ssz::types variable vec should not contain more elements than max");
            }

            hasher
                .finish()
                .expect("ssz::types variable vec should not have a remaining buffer")
        }
        TreeHashType::Container | TreeHashType::List | TreeHashType::Vector => {
            let mut hasher = MerkleHasher::with_leaves(N::to_usize());

            for item in vec {
                hasher
                    .write(&item.tree_hash_root())
                    .expect("ssz::types vec should not contain more elements than max");
            }

            hasher
                .finish()
                .expect("ssz::types vec should not have a remaining buffer")
        }
    }
}

/// A helper function providing common functionality for finding the Merkle root of some bytes that
/// represent a bitfield.
pub fn bitfield_bytes_tree_hash_root<N: Unsigned>(bytes: &[u8]) -> Hash256 {
    let byte_size = (N::to_usize() + 7) / 8;
    let leaf_count = (byte_size + BYTES_PER_CHUNK - 1) / BYTES_PER_CHUNK;

    let mut hasher = MerkleHasher::with_leaves(leaf_count);

    hasher
        .write(bytes)
        .expect("bitfield should not exceed tree hash leaf limit");

    hasher
        .finish()
        .expect("bitfield tree hash buffer should not exceed leaf limit")
}
