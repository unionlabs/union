use super::{get_zero_hash, Hash256, BYTES_PER_CHUNK};
use crate::tree_hash::{hash_concat, hash_fixed};

/// Merkleize `bytes` and return the root, optionally padding the tree out to `min_leaves` number of
/// leaves.
///
/// **Note**: This function is generally worse than using the `crate::merkle_root` which uses
/// `MerkleHasher`. We only keep this function around for reference testing.
///
/// First all nodes are extracted from `bytes` and then a padding node is added until the number of
/// leaf chunks is greater than or equal to `min_leaves`. Callers may set `min_leaves` to `0` if no
/// adding additional chunks should be added to the given `bytes`.
///
/// If `bytes.len() <= BYTES_PER_CHUNK`, no hashing is done and `bytes` is returned, potentially
/// padded out to `BYTES_PER_CHUNK` length with `0`.
///
/// ## CPU Performance
///
/// A cache of `MAX_TREE_DEPTH` hashes are stored to avoid re-computing the hashes of padding nodes
/// (or their parents). Therefore, adding padding nodes only incurs one more hash per additional
/// height of the tree.
///
/// ## Memory Performance
///
/// This algorithm has two interesting memory usage properties:
///
/// 1. The maximum memory footprint is roughly `O(V / 2)` memory, where `V` is the number of leaf
///    chunks with values (i.e., leaves that are not padding). The means adding padding nodes to
///    the tree does not increase the memory footprint.
/// 2. At each height of the tree half of the memory is freed until only a single chunk is stored.
/// 3. The input `bytes` are not copied into another list before processing.
///
/// _Note: there are some minor memory overheads, including a handful of usizes and a list of
/// `MAX_TREE_DEPTH` hashes as `lazy_static` constants._
#[must_use]
pub fn merkleize_padded(bytes: &[u8], min_leaves: usize) -> Hash256 {
    // If the bytes are just one chunk or less, pad to one chunk and return without hashing.
    if bytes.len() <= BYTES_PER_CHUNK && min_leaves <= 1 {
        let mut o = [0; BYTES_PER_CHUNK];
        o[0..bytes.len()].copy_from_slice(bytes);
        return o;
    }

    assert!(
        bytes.len() > BYTES_PER_CHUNK || min_leaves > 1,
        "Merkle hashing only needs to happen if there is more than one chunk"
    );

    // The number of leaves that can be made directly from `bytes`.
    let leaves_with_values = bytes.len().div_ceil(BYTES_PER_CHUNK);

    // The number of parents that have at least one non-padding leaf.
    //
    // Since there is more than one node in this tree (see prior assertion), there should always be
    // one or more initial parent nodes.
    let initial_parents_with_values = std::cmp::max(1, next_even_number(leaves_with_values) / 2);

    // The number of leaves in the full tree (including padding nodes).
    let num_leaves = std::cmp::max(leaves_with_values, min_leaves).next_power_of_two();

    // The number of levels in the tree.
    //
    // A tree with a single node has `height == 1`.
    let height = num_leaves.trailing_zeros() as usize + 1;

    assert!(height >= 2, "The tree should have two or more heights");

    // A buffer/scratch-space used for storing each round of hashes at each height.
    //
    // This buffer is kept as small as possible; it will shrink so it never stores a padding node.
    let mut chunks = ChunkStore::with_capacity(initial_parents_with_values);

    // Create a parent in the `chunks` buffer for every two chunks in `bytes`.
    //
    // I.e., do the first round of hashing, hashing from the `bytes` slice and filling the `chunks`
    // struct.
    for i in 0..initial_parents_with_values {
        let start = i * BYTES_PER_CHUNK * 2;

        // Hash two chunks, creating a parent chunk.
        let hash = match bytes.get(start..start + BYTES_PER_CHUNK * 2) {
            // All bytes are available, hash as usual.
            Some(slice) => hash_fixed(slice),
            // Unable to get all the bytes, get a small slice and pad it out.
            None => {
                let mut preimage = bytes
                    .get(start..)
                    .expect("`i` can only be larger than zero if there are bytes to read")
                    .to_vec();
                preimage.resize(BYTES_PER_CHUNK * 2, 0);
                hash_fixed(&preimage)
            }
        };

        assert_eq!(
            hash.len(),
            BYTES_PER_CHUNK,
            "Hashes should be exactly one chunk"
        );

        // Store the parent node.
        chunks
            .set(i, &hash)
            .expect("Buffer should always have capacity for parent nodes");
    }

    // Iterate through all heights above the leaf nodes and either (a) hash two children or, (b)
    // hash a left child and a right padding node.
    //
    // Skip the 0'th height because the leaves have already been processed. Skip the highest-height
    // in the tree as it is the root does not require hashing.
    //
    // The padding nodes for each height are cached via `lazy static` to simulate non-adjacent
    // padding nodes (i.e., avoid doing unnecessary hashing).
    for height in 1..height - 1 {
        let child_nodes = chunks.len();
        let parent_nodes = next_even_number(child_nodes) / 2;

        // For each pair of nodes stored in `chunks`:
        //
        // - If two nodes are available, hash them to form a parent.
        // - If one node is available, hash it and a cached padding node to form a parent.
        for i in 0..parent_nodes {
            let (left, right) = match (chunks.get(i * 2), chunks.get(i * 2 + 1)) {
                (Ok(left), Ok(right)) => (left, right),
                (Ok(left), Err(_)) => (left, get_zero_hash(height)),
                // Deriving `parent_nodes` from `chunks.len()` has ensured that we never encounter the
                // scenario where we expect two nodes but there are none.
                (Err(_), Err(_)) => unreachable!("Parent must have one child"),
                // `chunks` is a contiguous array so it is impossible for an index to be missing
                // when a higher index is present.
                (Err(_), Ok(_)) => unreachable!("Parent must have a left child"),
            };

            assert!(
                left.len() == right.len() && right.len() == BYTES_PER_CHUNK,
                "Both children should be `BYTES_PER_CHUNK` bytes."
            );

            let hash = hash_concat(left, right);

            // Store a parent node.
            chunks
                .set(i, &hash)
                .expect("Buf is adequate size for parent");
        }

        // Shrink the buffer so it neatly fits the number of new nodes created in this round.
        //
        // The number of `parent_nodes` is either decreasing or stable. It never increases.
        chunks.truncate(parent_nodes);
    }

    // There should be a single chunk left in the buffer and it is the Merkle root.
    let root = chunks.into_vec();

    assert_eq!(root.len(), BYTES_PER_CHUNK, "Only one chunk should remain");

    root.try_into().expect("length is checked above")
}

/// A helper struct for storing words of `BYTES_PER_CHUNK` size in a flat byte array.
#[derive(Debug)]
struct ChunkStore(Vec<u8>);

impl ChunkStore {
    /// Creates a new instance with `chunks` padding nodes.
    fn with_capacity(chunks: usize) -> Self {
        Self(vec![0; chunks * BYTES_PER_CHUNK])
    }

    /// Set the `i`th chunk to `value`.
    ///
    /// Returns `Err` if `value.len() != BYTES_PER_CHUNK` or `i` is out-of-bounds.
    fn set(&mut self, i: usize, value: &[u8]) -> Result<(), ()> {
        if i < self.len() && value.len() == BYTES_PER_CHUNK {
            let slice = &mut self.0[i * BYTES_PER_CHUNK..i * BYTES_PER_CHUNK + BYTES_PER_CHUNK];
            slice.copy_from_slice(value);
            Ok(())
        } else {
            Err(())
        }
    }

    /// Gets the `i`th chunk.
    ///
    /// Returns `Err` if `i` is out-of-bounds.
    fn get(&self, i: usize) -> Result<&[u8], ()> {
        if i < self.len() {
            Ok(&self.0[i * BYTES_PER_CHUNK..i * BYTES_PER_CHUNK + BYTES_PER_CHUNK])
        } else {
            Err(())
        }
    }

    /// Returns the number of chunks presently stored in `self`.
    fn len(&self) -> usize {
        self.0.len() / BYTES_PER_CHUNK
    }

    /// Truncates 'self' to `num_chunks` chunks.
    ///
    /// Functionally identical to `Vec::truncate`.
    fn truncate(&mut self, num_chunks: usize) {
        self.0.truncate(num_chunks * BYTES_PER_CHUNK);
    }

    /// Consumes `self`, returning the underlying byte array.
    fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

/// Returns the next even number following `n`. If `n` is even, `n` is returned.
fn next_even_number(n: usize) -> usize {
    n + n % 2
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tree_hash::ZERO_HASHES_MAX_INDEX;

    pub fn reference_root(bytes: &[u8]) -> Hash256 {
        crate::tree_hash::merkleize_standard(bytes)
    }

    macro_rules! common_tests {
        ($get_bytes: ident) => {
            #[test]
            fn zero_value_0_nodes() {
                test_against_reference(&$get_bytes(0 * BYTES_PER_CHUNK), 0);
            }

            #[test]
            fn zero_value_1_nodes() {
                test_against_reference(&$get_bytes(1 * BYTES_PER_CHUNK), 0);
            }

            #[test]
            fn zero_value_2_nodes() {
                test_against_reference(&$get_bytes(2 * BYTES_PER_CHUNK), 0);
            }

            #[test]
            fn zero_value_3_nodes() {
                test_against_reference(&$get_bytes(3 * BYTES_PER_CHUNK), 0);
            }

            #[test]
            fn zero_value_4_nodes() {
                test_against_reference(&$get_bytes(4 * BYTES_PER_CHUNK), 0);
            }

            #[test]
            fn zero_value_8_nodes() {
                test_against_reference(&$get_bytes(8 * BYTES_PER_CHUNK), 0);
            }

            #[test]
            fn zero_value_9_nodes() {
                test_against_reference(&$get_bytes(9 * BYTES_PER_CHUNK), 0);
            }

            #[test]
            fn zero_value_8_nodes_varying_min_length() {
                for i in 0..64 {
                    test_against_reference(&$get_bytes(8 * BYTES_PER_CHUNK), i);
                }
            }

            #[test]
            fn zero_value_range_of_nodes() {
                for i in 0..32 * BYTES_PER_CHUNK {
                    test_against_reference(&$get_bytes(i), 0);
                }
            }

            #[test]
            fn max_tree_depth_min_nodes() {
                let input = vec![0; 10 * BYTES_PER_CHUNK];
                let min_nodes = 2usize.pow(ZERO_HASHES_MAX_INDEX as u32);
                assert_eq!(
                    merkleize_padded(&input, min_nodes),
                    get_zero_hash(ZERO_HASHES_MAX_INDEX)
                );
            }
        };
    }

    mod zero_value {
        use super::*;

        fn zero_bytes(bytes: usize) -> Vec<u8> {
            vec![0; bytes]
        }

        common_tests!(zero_bytes);
    }

    // mod random_value {
    //     use rand::RngCore;

    //     use super::*;

    //     fn random_bytes(bytes: usize) -> Vec<u8> {
    //         let mut bytes = Vec::with_capacity(bytes);
    //         rand::thread_rng().fill_bytes(&mut bytes);
    //         bytes
    //     }

    //     common_tests!(random_bytes);
    // }

    fn test_against_reference(input: &[u8], min_nodes: usize) {
        let mut reference_input = input.to_vec();
        reference_input.resize(
            std::cmp::max(
                reference_input.len(),
                min_nodes.next_power_of_two() * BYTES_PER_CHUNK,
            ),
            0,
        );

        assert_eq!(
            reference_root(&reference_input),
            merkleize_padded(input, min_nodes),
            "input.len(): {:?}",
            input.len()
        );
    }
}
