# ADR ADR-001: Node Key Refactoring

## Changelog

- 2022-10-31: First draft

## Status

Proposed

## Context

The original key format of IAVL nodes is a hash of the node. It does not take advantage of data locality on LSM-Tree. Nodes are stored with the random hash value, so it increases the number of compactions and makes it difficult to find the node. The new key format will take advantage of data locality in the LSM tree and reduce the number of compactions.

The `orphans` are used to manage node removal in the current design and allow the deletion of removed nodes for the specific version from the disk through the `DeleteVersion` API. It needs to track every time when updating the tree and also requires extra storage to store `orphans`. But there are only 2 use cases for `DeleteVersion`:

1. Rollback of the tree to a previous version
2. Remove unnecessary old nodes

## Decision

- Use the version and the local nonce as a node key like `bigendian(version) | bigendian(nonce)` format. Here the `nonce` is a local sequence id for the same version.
	- Store the children node keys (`leftNodeKey` and `rightNodeKey`) in the node body.
	- Remove the `version` field from node body writes.
	- Remove the `leftHash` and `rightHash` fields, and instead store `hash` field in the node body.
- Remove the `orphans` completely from both tree and storage.

New node structure

```go
type NodeKey struct {
	version int64
	nonce   int32
}

type Node struct {
	key           []byte
	value         []byte
	hash          []byte     // keep it in the storage instead of leftHash and rightHash
	nodeKey       *NodeKey   // new field, the key in the storage
	leftNodeKey   *NodeKey   // new field, need to store in the storage
	rightNodeKey  *NodeKey   // new field, need to store in the storage
	leftNode      *Node
	rightNode     *Node
	size          int64
	leftNode      *Node
	rightNode     *Node
	subtreeHeight int8
}
```

New tree structure

```go
type MutableTree struct {
	*ImmutableTree                                     // The current, working tree.
	lastSaved                *ImmutableTree            // The most recently saved tree.
	unsavedFastNodeAdditions map[string]*fastnode.Node // FastNodes that have not yet been saved to disk
	unsavedFastNodeRemovals  map[string]interface{}    // FastNodes that have not yet been removed from disk
	ndb                      *nodeDB
	skipFastStorageUpgrade   bool // If true, the tree will work like no fast storage and always not upgrade fast storage
	
	mtx sync.Mutex
}
```

We will assign the `nodeKey` when saving the current version in `SaveVersion`. It will reduce unnecessary checks in CRUD operations of the tree and keep sorted the order of insertion in the LSM tree.

### Migration

We can migrate nodes through the following steps:

- Export the snapshot of the tree from the original version.
- Import the snapshot to the new version.
	- Track the nonce for the same version using int32 array of the version length.
	- Assign the `nodeKey` when saving the node.

### Pruning

The current pruning strategies allows for intermediate versions to exist. With the adoption of this ADR we are migrating to allowing only versions to exist between a range (50-100 instead of 1,25,50-100).

Here we are introducing a new way how to get orphaned nodes which remove in the `n+1`th version updates without storing orphanes in the storage.

When we want to remove the `n+1`th version

- Traverse the tree in-order way based on the root of `n+1`th version.
- If we visit the lower version node, pick the node and don't visit further deeply. Pay attention to the order of these nodes.
- Traverse the tree in-order way based on the root of `n`th version.
- Iterate the tree until meet the first node among the above nodes(stack) and delete all visited nodes so far from `n`th tree.
- Pop the first node from the stack and iterate again.

If we assume `1 to (n-1)` versions already been removed, when we want to remove the `n`th version, we can just remove the above orphaned nodes.

### Rollback

When we want to rollback to the specific version `n`

- Iterate the version from `n+1`.
- Traverse key-value through `traversePrefix` with `prefix=bigendian(version)`.
- Remove all iterated nodes.

## Consequences

### Positive

* Using the version and a local nonce, we take advantage of data locality in the LSM tree. Since we commit the sorted data, it can reduce compactions and makes it easy to find the key. Also, it can reduce the key and node size in the storage.

	```
	# node body

	add `hash`:							+32 byte
	add `leftNodeKey`, `rightNodeKey`:	max (8 + 4) * 2 = 	+24 byte
	remove `leftHash`, `rightHash`:			    		-64 byte
	remove `version`: 					max	 -8 byte
	------------------------------------------------------------
						total save	 	 16 byte

	# node key

	remove `hash`:			-32 byte
	add `version|nonce`:		+12 byte
	------------------------------------
			total save 	 20 byte
	```

* Removing orphans also provides performance improvements including memory and storage saving.

### Negative

* `Update` operations will require extra DB access because we need to take children to calculate the hash of updated nodes.
	* It doesn't require more access in other cases including `Set`, `Remove`, and `Proof`.

* It is impossible to remove the individual version. The new design requires more restrict pruning strategies.

* When importing the tree, it may require more memory because of int32 array of the version length. We will introduce the new importing strategy to reduce the memory usage.

## References

- https://github.com/cosmos/iavl/issues/548
- https://github.com/cosmos/iavl/issues/137
- https://github.com/cosmos/iavl/issues/571
- https://github.com/cosmos/cosmos-sdk/issues/12989
