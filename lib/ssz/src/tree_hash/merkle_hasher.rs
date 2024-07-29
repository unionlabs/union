use std::mem;

use sha2::Digest;
use smallvec::{smallvec, SmallVec};

use crate::tree_hash::{get_zero_hash, Hash256, HASHSIZE};

type SmallVec8<T> = SmallVec<[T; 8]>;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// The maximum number of leaves defined by the initialization `depth` has been exceed.
    MaximumLeavesExceeded { max_leaves: usize },
}

/// Helper struct to store either a hash digest or a slice.
///
/// Should be used as a left or right value for some node.
enum Preimage<'a> {
    Digest([u8; 32]),
    Slice(&'a [u8]),
}

impl<'a> Preimage<'a> {
    /// Returns a 32-byte slice.
    fn as_bytes(&self) -> &[u8] {
        match self {
            Preimage::Digest(digest) => digest.as_ref(),
            Preimage::Slice(slice) => slice,
        }
    }
}

/// A node that has had a left child supplied, but not a right child.
struct HalfNode {
    /// The hasher context.
    context: sha2::Sha256,
    /// The tree id of the node. The root node has in id of `1` and ids increase moving down the
    /// tree from left to right.
    id: usize,
}

impl HalfNode {
    /// Create a new half-node from the given `left` value.
    fn new(id: usize, left: Preimage) -> Self {
        let mut context = sha2::Sha256::new();
        context.update(left.as_bytes());

        Self { context, id }
    }

    /// Complete the half-node by providing a `right` value. Returns a digest of the left and right
    /// nodes.
    fn finish(mut self, right: Preimage) -> [u8; 32] {
        self.context.update(right.as_bytes());
        self.context.finalize().into()
    }
}

/// Provides a Merkle-root hasher that allows for streaming bytes (i.e., providing any-length byte
/// slices without need to separate into leaves). Efficiently handles cases where not all leaves
/// have been provided by assuming all non-provided leaves are `[0; 32]` and pre-computing the
/// zero-value hashes at all depths of the tree.
///
/// This algorithm aims to allocate as little memory as possible and it does this by "folding" up
/// the tree as each leaf is provided. Consider this step-by-step functional diagram of hashing a
/// tree with depth three:
///
/// ## Functional Diagram
///
/// Nodes that are `-` have not been defined and do not occupy memory. Nodes that are `L` are
/// leaves that are provided but are not stored. Nodes that have integers (`1`, `2`) are stored in
/// our struct. Finally, nodes that are `X` were stored, but are now removed.
///
/// ### Start
///
/// ```plaintext
///           -
///          / \
///         -   -
///        / \ / \
///       -  - -  -
/// ```
///
/// ### Provide first leaf
///
/// ```plaintext
///           -
///          / \
///         2   -
///        / \ / \
///       L  - -  -
/// ```
///
/// ### Provide second leaf
///
/// ```plaintext
///           1
///          / \
///         X   -
///        / \ / \
///       L  L -  -
/// ```
///
/// ### Provide third leaf
///
/// ```plaintext
///           1
///          / \
///         X   3
///        / \ / \
///       L  L L  -
/// ```
///
/// ### Provide fourth and final leaf
///
/// ```plaintext
///           1
///          / \
///         X   X
///        / \ / \
///       L  L L  L
/// ```
pub struct MerkleHasher {
    /// Stores the nodes that are half-complete and awaiting a right node.
    ///
    /// A smallvec of size 8 means we can hash a tree with 256 leaves without allocating on the
    /// heap. Each half-node is 232 bytes, so this smallvec may store 1856 bytes on the stack.
    half_nodes: SmallVec8<HalfNode>,
    /// The depth of the tree that will be produced.
    ///
    /// Depth is counted top-down (i.e., the root node is at depth 0). A tree with 1 leaf has a
    /// depth of 1, a tree with 4 leaves has a depth of 3.
    depth: usize,
    /// The next leaf that we are expecting to process.
    next_leaf: usize,
    /// A buffer of bytes that are waiting to be written to a leaf.
    buffer: SmallVec<[u8; 32]>,
    /// Set to Some(root) when the root of the tree is known.
    root: Option<Hash256>,
}

/// Returns the parent of node with id `i`.
fn get_parent(i: usize) -> usize {
    i / 2
}

/// Gets the depth of a node with an id of `i`.
///
/// It is a logic error to provide `i == 0`.
///
/// E.g., if `i` is 1, depth is 0. If `i` is is 1, depth is 1.
fn get_depth(i: usize) -> usize {
    let total_bits = mem::size_of::<usize>() * 8;
    total_bits - i.leading_zeros() as usize - 1
}

impl MerkleHasher {
    /// Instantiate a hasher for a tree with a given number of leaves.
    ///
    /// `num_leaves` will be rounded to the next power of two. E.g., if `num_leaves == 6`, then the
    /// tree will _actually_ be able to accommodate 8 leaves and the resulting hasher is exactly the
    /// same as one that was instantiated with `Self::with_leaves(8)`.
    ///
    /// ## Notes
    ///
    /// If `num_leaves == 0`, a tree of depth 1 will be created. If no leaves are provided it will
    /// return a root of `[0; 32]`.
    #[must_use]
    pub fn with_leaves(num_leaves: usize) -> Self {
        let depth = get_depth(num_leaves.next_power_of_two()) + 1;
        Self::with_depth(depth)
    }

    /// Instantiates a new, empty hasher for a tree with `depth` layers which will have capacity
    /// for `1 << (depth - 1)` leaf nodes.
    ///
    /// It is not possible to grow the depth of the tree after instantiation.
    ///
    /// ## Panics
    ///
    /// Panics if `depth == 0`.
    fn with_depth(depth: usize) -> Self {
        assert!(depth > 0, "merkle tree cannot have a depth of zero");

        Self {
            half_nodes: SmallVec::with_capacity(depth - 1),
            depth,
            next_leaf: 1 << (depth - 1),
            buffer: SmallVec::with_capacity(32),
            root: None,
        }
    }

    /// Write some bytes to the hasher.
    ///
    /// ## Errors
    ///
    /// Returns an error if the given bytes would create a leaf that would exceed the maximum
    /// permissible number of leaves defined by the initialization `depth`. E.g., a tree of `depth
    /// == 2` can only accept 2 leaves. A tree of `depth == 14` can only accept 8,192 leaves.
    pub fn write(&mut self, bytes: &[u8]) -> Result<(), Error> {
        let mut ptr = 0;
        while ptr <= bytes.len() {
            let slice = &bytes[ptr..std::cmp::min(bytes.len(), ptr + HASHSIZE)];

            if self.buffer.is_empty() && slice.len() == HASHSIZE {
                self.process_leaf(slice.try_into().expect("size is checked above"))?;
                ptr += HASHSIZE;
            } else if self.buffer.len() + slice.len() < HASHSIZE {
                self.buffer.extend_from_slice(slice);
                ptr += HASHSIZE;
            } else {
                let buf_len = self.buffer.len();
                let required = HASHSIZE - buf_len;

                let mut leaf = [0; HASHSIZE];
                leaf[..buf_len].copy_from_slice(&self.buffer);
                leaf[buf_len..].copy_from_slice(&slice[0..required]);

                self.process_leaf(leaf)?;
                self.buffer = smallvec![];

                ptr += required;
            }
        }

        Ok(())
    }

    /// Process the next leaf in the tree.
    ///
    /// ## Errors
    ///
    /// Returns an error if the given leaf would exceed the maximum permissible number of leaves
    /// defined by the initialization `depth`. E.g., a tree of `depth == 2` can only accept 2
    /// leaves. A tree of `depth == 14` can only accept 8,192 leaves.
    fn process_leaf(&mut self, leaf: [u8; 32]) -> Result<(), Error> {
        assert_eq!(leaf.len(), HASHSIZE, "a leaf must be 32 bytes");

        let max_leaves = 1 << (self.depth + 1);

        if self.next_leaf > max_leaves {
            return Err(Error::MaximumLeavesExceeded { max_leaves });
        } else if self.next_leaf == 1 {
            // A tree of depth one has a root that is equal to the first given leaf.
            self.root = Some(leaf);
        } else if self.next_leaf % 2 == 0 {
            self.process_left_node(self.next_leaf, Preimage::Slice(&leaf));
        } else {
            self.process_right_node(self.next_leaf, Preimage::Slice(&leaf));
        }

        self.next_leaf += 1;

        Ok(())
    }

    /// Returns the root of the Merkle tree.
    ///
    /// If not all leaves have been provided, the tree will be efficiently completed under the
    /// assumption that all not-yet-provided leaves are equal to `[0; 32]`.
    ///
    /// ## Errors
    ///
    /// Returns an error if the bytes remaining in the buffer would create a leaf that would exceed
    /// the maximum permissible number of leaves defined by the initialization `depth`.
    pub fn finish(mut self) -> Result<Hash256, Error> {
        if !self.buffer.is_empty() {
            let mut leaf = [0; HASHSIZE];
            leaf[..self.buffer.len()].copy_from_slice(&self.buffer);
            self.process_leaf(leaf)?;
        }

        // If the tree is incomplete, we must complete it by providing zero-hashes.
        loop {
            if let Some(root) = self.root {
                break Ok(root);
            } else if let Some(node) = self.half_nodes.last() {
                let right_child = node.id * 2 + 1;
                self.process_right_node(right_child, self.zero_hash(right_child));
            } else if self.next_leaf == 1 {
                // The next_leaf can only be 1 if the tree has a depth of one. If have been no
                // leaves supplied, assume a root of zero.
                break Ok([0; 32]);
            } else {
                // The only scenario where there are (a) no half nodes and (b) a tree of depth
                // two or more is where no leaves have been supplied at all.
                //
                // Once we supply this first zero-hash leaf then all future operations will be
                // triggered via the `process_right_node` branch.
                self.process_left_node(self.next_leaf, self.zero_hash(self.next_leaf));
            }
        }
    }

    /// Process a node that will become the left-hand node of some parent. The supplied `id` is
    /// that of the node (not the parent). The `preimage` is the value of the node (i.e., if this
    /// is a leaf node it will be the value of that leaf).
    ///
    /// In this scenario, the only option is to push a new half-node.
    fn process_left_node(&mut self, id: usize, preimage: Preimage) {
        self.half_nodes
            .push(HalfNode::new(get_parent(id), preimage));
    }

    /// Process a node that will become the right-hand node of some parent. The supplied `id` is
    /// that of the node (not the parent). The `preimage` is the value of the node (i.e., if this
    /// is a leaf node it will be the value of that leaf).
    ///
    /// This operation will always complete one node, then it will attempt to crawl up the tree and
    /// collapse all other completed nodes. For example, consider a tree of depth 3 (see diagram
    /// below). When providing the node with id `7`, the node with id `3` will be completed which
    /// will also provide the right-node for the `1` node. This function will complete both of
    /// those nodes and ultimately find the root of the tree.
    ///
    /// ```plaintext
    ///           1 <-- completed
    ///          / \
    ///         2   3  <-- completed
    ///        / \ / \
    ///       4  5 6  7 <-- supplied right node
    /// ```
    fn process_right_node(&mut self, id: usize, mut preimage: Preimage) {
        let mut parent = get_parent(id);

        loop {
            match self.half_nodes.last() {
                Some(node) if node.id == parent => {
                    let digest = self
                        .half_nodes
                        .pop()
                        .expect("if .last() is Some then .pop() must succeed")
                        .finish(preimage);
                    preimage = Preimage::Digest(digest);
                    if parent == 1 {
                        self.root = Some(digest);
                        break;
                    } else {
                        parent = get_parent(parent);
                    }
                }
                _ => {
                    self.half_nodes.push(HalfNode::new(parent, preimage));
                    break;
                }
            }
        }
    }

    /// Returns a "zero hash" from a pre-computed set for the given node.
    ///
    /// Note: this node is not always zero, instead it is the result of hashing up a tree where the
    /// leaves are all zeros.  E.g., in a tree of depth 2, the `zero_hash` of a node at depth 1
    /// will be `[0; 32]`.  However, the `zero_hash` for a node at depth 0 will be
    /// `hash(concat([0; 32], [0; 32])))`.
    fn zero_hash(&self, id: usize) -> Preimage<'static> {
        Preimage::Slice(get_zero_hash(self.depth - (get_depth(id) + 1)))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tree_hash::merkleize_padded;

    /// This test is just to ensure that the stack size of the `Context` remains the same. We choose
    /// our smallvec size based upon this, so it's good to know if it suddenly changes in size.
    #[test]
    fn context_size() {
        assert_eq!(
            mem::size_of::<HalfNode>(),
            120,
            "Halfnode size should be as expected"
        );
    }

    fn compare_with_reference(leaves: &[Hash256], depth: usize) {
        let reference_bytes = leaves.iter().flatten().copied().collect::<Vec<_>>();

        let reference_root = merkleize_padded(&reference_bytes, 1 << (depth - 1));

        let merklizer_root_32_bytes = {
            let mut m = MerkleHasher::with_depth(depth);
            for leaf in leaves.iter() {
                m.write(leaf).expect("should process leaf");
            }
            m.finish().expect("should finish")
        };

        assert_eq!(
            reference_root, merklizer_root_32_bytes,
            "32 bytes should match reference root"
        );

        let merklizer_root_individual_3_bytes = {
            let mut m = MerkleHasher::with_depth(depth);
            for bytes in reference_bytes.chunks(3) {
                m.write(bytes).expect("should process byte");
            }
            m.finish().expect("should finish")
        };

        assert_eq!(
            reference_root, merklizer_root_individual_3_bytes,
            "3 bytes should match reference root"
        );

        let merklizer_root_individual_single_bytes = {
            let mut m = MerkleHasher::with_depth(depth);
            for byte in reference_bytes.iter() {
                m.write(&[*byte]).expect("should process byte");
            }
            m.finish().expect("should finish")
        };

        assert_eq!(
            reference_root, merklizer_root_individual_single_bytes,
            "single bytes should match reference root"
        );
    }

    /// A simple wrapper to compare MerkleHasher to the reference function by just giving a number
    /// of leaves and a depth.
    fn compare_reference_with_len(leaves: u64, depth: usize) {
        let leaves = (0..leaves)
            .map(|n| {
                let n = n.to_be_bytes();
                let mut out = [0; 32];
                out[24..32].copy_from_slice(&n);
                out
            })
            .collect::<Vec<_>>();
        compare_with_reference(&leaves, depth);
    }

    /// Compares the `MerkleHasher::with_depth` and `MerkleHasher::with_leaves` generate consistent
    /// results.
    fn compare_new_with_leaf_count(num_leaves: u64, depth: usize) {
        let leaves = (0..num_leaves)
            .map(|n| {
                let n = n.to_be_bytes();
                let mut out = [0; 32];
                out[24..32].copy_from_slice(&n);
                out
            })
            .collect::<Vec<_>>();

        let from_depth = {
            let mut m = MerkleHasher::with_depth(depth);
            for leaf in leaves.iter() {
                m.write(leaf).expect("should process leaf");
            }
            m.finish()
        };

        let from_num_leaves = {
            let mut m = MerkleHasher::with_leaves(num_leaves as usize);
            for leaf in leaves.iter() {
                m.process_leaf(*leaf).expect("should process leaf");
            }
            m.finish()
        };

        assert_eq!(
            from_depth, from_num_leaves,
            "hash generated by depth should match that from num leaves"
        );
    }

    #[test]
    fn with_leaves() {
        compare_new_with_leaf_count(1, 1);
        compare_new_with_leaf_count(2, 2);
        compare_new_with_leaf_count(3, 3);
        compare_new_with_leaf_count(4, 3);
        compare_new_with_leaf_count(5, 4);
        compare_new_with_leaf_count(6, 4);
        compare_new_with_leaf_count(7, 4);
        compare_new_with_leaf_count(8, 4);
        compare_new_with_leaf_count(9, 5);
        compare_new_with_leaf_count(10, 5);
        compare_new_with_leaf_count(11, 5);
        compare_new_with_leaf_count(12, 5);
        compare_new_with_leaf_count(13, 5);
        compare_new_with_leaf_count(14, 5);
        compare_new_with_leaf_count(15, 5);
    }

    #[test]
    fn depth() {
        assert_eq!(get_depth(1), 0);
        assert_eq!(get_depth(2), 1);
        assert_eq!(get_depth(3), 1);
        assert_eq!(get_depth(4), 2);
        assert_eq!(get_depth(5), 2);
        assert_eq!(get_depth(6), 2);
        assert_eq!(get_depth(7), 2);
        assert_eq!(get_depth(8), 3);
    }

    #[test]
    fn with_0_leaves() {
        let hasher = MerkleHasher::with_leaves(0);
        assert_eq!(hasher.finish().unwrap(), [0; 32]);
    }

    #[test]
    #[should_panic]
    fn too_many_leaves() {
        compare_reference_with_len(2, 1);
    }

    #[test]
    fn full_trees() {
        compare_reference_with_len(1, 1);
        compare_reference_with_len(2, 2);
        compare_reference_with_len(4, 3);
        compare_reference_with_len(8, 4);
        compare_reference_with_len(16, 5);
        compare_reference_with_len(32, 6);
        compare_reference_with_len(64, 7);
        compare_reference_with_len(128, 8);
        compare_reference_with_len(256, 9);
        compare_reference_with_len(256, 9);
        compare_reference_with_len(8192, 14);
    }

    #[test]
    fn incomplete_trees() {
        compare_reference_with_len(0, 1);

        compare_reference_with_len(0, 2);
        compare_reference_with_len(1, 2);

        for i in 0..=4 {
            compare_reference_with_len(i, 3);
        }

        for i in 0..=7 {
            compare_reference_with_len(i, 4);
        }

        for i in 0..=15 {
            compare_reference_with_len(i, 5);
        }

        for i in 0..=32 {
            compare_reference_with_len(i, 6);
        }

        for i in 0..=64 {
            compare_reference_with_len(i, 7);
        }

        compare_reference_with_len(0, 14);
        compare_reference_with_len(13, 14);
        compare_reference_with_len(8191, 14);
    }

    #[test]
    fn remaining_buffer() {
        let a = {
            let mut m = MerkleHasher::with_leaves(2);
            m.write(&[1]).expect("should write");
            m.finish().expect("should finish")
        };

        let b = {
            let mut m = MerkleHasher::with_leaves(2);
            let mut leaf = vec![1];
            leaf.extend_from_slice(&[0; 31]);
            m.write(&leaf).expect("should write");
            m.write(&[0; 32]).expect("should write");
            m.finish().expect("should finish")
        };

        assert_eq!(a, b, "should complete buffer");
    }
}
