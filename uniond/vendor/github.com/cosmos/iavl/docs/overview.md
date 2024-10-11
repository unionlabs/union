# IAVL Spec

The IAVL tree is a versioned, snapshottable (immutable) AVL+ tree for persistent data.

The purpose of this data structure is to provide persistent storage for key-value pairs (say to store account balances) such that a deterministic merkle root hash can be computed.  The tree is balanced using a variant of the [AVL algorithm](http://en.wikipedia.org/wiki/AVL_tree) so all operations are O(log(n)).

Nodes of this tree are immutable and indexed by their hash.  Thus any node serves as an immutable snapshot which lets us stage uncommitted transactions from the mempool cheaply, and we can instantly roll back to the last committed state to process transactions of a newly committed block (which may not be the same set of transactions as those from the mempool).

In an AVL tree, the heights of the two child subtrees of any node differ by at most one.  Whenever this condition is violated upon an update, the tree is rebalanced by creating O(log(n)) new nodes that point to unmodified nodes of the old tree.  In the original AVL algorithm, inner nodes can also hold key-value pairs.  The AVL+ algorithm (note the plus) modifies the AVL algorithm to keep all values on leaf nodes, while only using branch-nodes to store keys.  This simplifies the algorithm while keeping the merkle hash trail short.

The IAVL tree will typically be wrapped by a `MutableTree` to enable updates to the tree. Any changes between versions get persisted to disk while nodes that exist in both the old version and new version are simply pointed to by the respective tree without duplicated the node data.

When a node is no longer part of the latest IAVL tree, it is called an orphan. The orphaned node will exist in the nodeDB so long as there are versioned IAVL trees that are persisted in nodeDB that contain the orphaned node. Once all trees that referred to orphaned node have been deleted from database, the orphaned node will also get deleted.

In Ethereum, the analog is [Patricia tries](http://en.wikipedia.org/wiki/Radix_tree).  There are tradeoffs.  Keys do not need to be hashed prior to insertion in IAVL+ trees, so this provides faster iteration in the key space which may benefit some applications.  The logic is simpler to implement, requiring only two types of nodes -- inner nodes and leaf nodes.  On the other hand, while IAVL+ trees provide a deterministic merkle root hash, it depends on the order of transactions.  In practice this shouldn't be a problem, since you can efficiently encode the tree structure when serializing the tree contents.

### Suggested Order for Understanding IAVL

1. [Node docs](./node/node.md)
    - Explains node structure
    - Explains how node gets marshalled and hashed
2. [KeyFormat docs](./node/key_format.md)
    - Explains keyformats for how nodes, orphans, and roots are stored under formatted keys in database
3. [NodeDB docs](./node/nodedb.md): 
    - Explains how nodes, orphans, roots get saved in database
    - Explains saving and deleting tree logic.
4. [ImmutableTree docs](./tree/immutable_tree.md)
    - Explains ImmutableTree structure
    - Explains ImmutableTree Iteration functions
5. [MutableTree docs](./tree/mutable_tree.md)
    - Explains MutableTree structure
    - Explains how to make updates (set/delete) to current working tree of IAVL
    - Explains how automatic rebalancing of IAVL works
    - Explains how Saving and Deleting versions of IAVL works
6. [Proof docs](./proof/proof.md)
    - Explains what Merkle proofs are
    - Explains how IAVL supports presence, absence, and range proofs
    - Explains the IAVL proof data structures
7. [Export/import docs](./tree/export_import.md)
    - Explains the overall export/import functionality
    - Explains the `ExportNode` format for exported nodes
    - Explains the algorithms for exporting and importing nodes