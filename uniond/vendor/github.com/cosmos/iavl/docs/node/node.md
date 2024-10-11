# Node

The Node struct stores a node in the IAVL tree.

## Structure

```golang
// NodeKey represents a key of node in the DB.
type NodeKey struct {
	version int64	// version of the IAVL that this node was first added in
	nonce   int32	// local nonce for the same version
}

// Node represents a node in a Tree.
type Node struct {
	key           []byte	// key for the node.
	value         []byte	// value of leaf node. If inner node, value = nil
	hash          []byte	// hash of above field and left node's hash, right node's hash
	nodeKey       *NodeKey	// node key of the nodeDB
	leftNodeKey   *NodeKey	// node key of the left child
	rightNodeKey  *NodeKey	// node key of the right child
	size          int64		// number of leaves that are under the current node. Leaf nodes have size = 1
	leftNode      *Node		// pointer to left child
	rightNode     *Node		// pointer to right child
	subtreeHeight int8		// height of the node. Leaf nodes have height 0
}
```

Inner nodes have keys equal to the highest key on the subtree and have values set to nil.

The version of a node is the first version of the IAVL tree that the node gets added in. Future versions of the IAVL may point to this node if they also contain the node, however the node's version itself does not change.

Size is the number of leaves under a given node. With a full subtree, `node.size = 2^(node.height)`.

### Marshaling

Every node is persisted by encoding the key, height, and size. If the node is a leaf node, then the value is persisted as well. If the node is not a leaf node, then the hash, leftNodeKey, and rightNodeKey are persisted as well. The hash should be persisted in inner nodes to avoid recalculating the hash when the node is loaded from the disk, if not persisted, we should iterate through the entire subtree to calculate the hash.

```golang
// Writes the node as a serialized byte slice to the supplied io.Writer.
func (node *Node) writeBytes(w io.Writer) error {
	if node == nil {
		return errors.New("cannot write nil node")
	}
	err := encoding.EncodeVarint(w, int64(node.subtreeHeight))
	if err != nil {
		return fmt.Errorf("writing height, %w", err)
	}
	err = encoding.EncodeVarint(w, node.size)
	if err != nil {
		return fmt.Errorf("writing size, %w", err)
	}

	// Unlike writeHashByte, key is written for inner nodes.
	err = encoding.EncodeBytes(w, node.key)
	if err != nil {
		return fmt.Errorf("writing key, %w", err)
	}

	if node.isLeaf() {
		err = encoding.EncodeBytes(w, node.value)
		if err != nil {
			return fmt.Errorf("writing value, %w", err)
		}
	} else {
		err = encoding.EncodeBytes(w, node.hash)
		if err != nil {
			return fmt.Errorf("writing hash, %w", err)
		}
		if node.leftNodeKey == nil {
			return ErrLeftNodeKeyEmpty
		}
		err = encoding.EncodeVarint(w, node.leftNodeKey.version)
		if err != nil {
			return fmt.Errorf("writing the version of left node key, %w", err)
		}
		err = encoding.EncodeVarint(w, int64(node.leftNodeKey.nonce))
		if err != nil {
			return fmt.Errorf("writing the nonce of left node key, %w", err)
		}

		if node.rightNodeKey == nil {
			return ErrRightNodeKeyEmpty
		}
		err = encoding.EncodeVarint(w, node.rightNodeKey.version)
		if err != nil {
			return fmt.Errorf("writing the version of right node key, %w", err)
		}
		err = encoding.EncodeVarint(w, int64(node.rightNodeKey.nonce))
		if err != nil {
			return fmt.Errorf("writing the nonce of right node key, %w", err)
		}
	}
	return nil
}
```

### Hashes

A node's hash is calculated by hashing the height, size, and version of the node. If the node is a leaf node, then the key and value are also hashed. If the node is an inner node, the leftHash and rightHash are included in hash but the key is not.

```golang
// Writes the node's hash to the given io.Writer. This function expects
// child hashes to be already set.
func (node *Node) writeHashBytes(w io.Writer, version int64) error {
	err := encoding.EncodeVarint(w, int64(node.subtreeHeight))
	if err != nil {
		return fmt.Errorf("writing height, %w", err)
	}
	err = encoding.EncodeVarint(w, node.size)
	if err != nil {
		return fmt.Errorf("writing size, %w", err)
	}
	err = encoding.EncodeVarint(w, version)
	if err != nil {
		return fmt.Errorf("writing version, %w", err)
	}

	// Key is not written for inner nodes, unlike writeBytes.

	if node.isLeaf() {
		err = encoding.EncodeBytes(w, node.key)
		if err != nil {
			return fmt.Errorf("writing key, %w", err)
		}

		// Indirection needed to provide proofs without values.
		// (e.g. ProofLeafNode.ValueHash)
		valueHash := sha256.Sum256(node.value)

		err = encoding.EncodeBytes(w, valueHash[:])
		if err != nil {
			return fmt.Errorf("writing value, %w", err)
		}
	} else {
		if node.leftNode == nil || node.rightNode == nil {
			return ErrEmptyChild
		}
		err = encoding.EncodeBytes(w, node.leftNode.hash)
		if err != nil {
			return fmt.Errorf("writing left hash, %w", err)
		}
		err = encoding.EncodeBytes(w, node.rightNode.hash)
		if err != nil {
			return fmt.Errorf("writing right hash, %w", err)
		}
	}

	return nil
}
```
