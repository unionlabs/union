// Copyright 2023 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>

package trienode

import (
	"fmt"
	"sort"
	"strings"

	"github.com/ethereum/go-ethereum/common"
)

// Node is a wrapper which contains the encoded blob of the trie node and its
// node hash. It is general enough that can be used to represent trie node
// corresponding to different trie implementations.
type Node struct {
	Hash common.Hash // Node hash, empty for deleted node
	Blob []byte      // Encoded node blob, nil for the deleted node
}

// Size returns the total memory size used by this node.
func (n *Node) Size() int {
	return len(n.Blob) + common.HashLength
}

// IsDeleted returns the indicator if the node is marked as deleted.
func (n *Node) IsDeleted() bool {
	return n.Hash == (common.Hash{})
}

// New constructs a node with provided node information.
func New(hash common.Hash, blob []byte) *Node {
	return &Node{Hash: hash, Blob: blob}
}

// NewDeleted constructs a node which is deleted.
func NewDeleted() *Node { return New(common.Hash{}, nil) }

// leaf represents a trie leaf node
type leaf struct {
	Blob   []byte      // raw blob of leaf
	Parent common.Hash // the hash of parent node
}

// NodeSet contains a set of nodes collected during the commit operation.
// Each node is keyed by path. It's not thread-safe to use.
type NodeSet struct {
	Owner   common.Hash
	Leaves  []*leaf
	Nodes   map[string]*Node
	updates int // the count of updated and inserted nodes
	deletes int // the count of deleted nodes
}

// NewNodeSet initializes a node set. The owner is zero for the account trie and
// the owning account address hash for storage tries.
func NewNodeSet(owner common.Hash) *NodeSet {
	return &NodeSet{
		Owner: owner,
		Nodes: make(map[string]*Node),
	}
}

// ForEachWithOrder iterates the nodes with the order from bottom to top,
// right to left, nodes with the longest path will be iterated first.
func (set *NodeSet) ForEachWithOrder(callback func(path string, n *Node)) {
	var paths []string
	for path := range set.Nodes {
		paths = append(paths, path)
	}
	// Bottom-up, the longest path first
	sort.Sort(sort.Reverse(sort.StringSlice(paths)))
	for _, path := range paths {
		callback(path, set.Nodes[path])
	}
}

// AddNode adds the provided node into set.
func (set *NodeSet) AddNode(path []byte, n *Node) {
	if n.IsDeleted() {
		set.deletes += 1
	} else {
		set.updates += 1
	}
	set.Nodes[string(path)] = n
}

// Merge adds a set of nodes into the set.
func (set *NodeSet) Merge(owner common.Hash, nodes map[string]*Node) error {
	if set.Owner != owner {
		return fmt.Errorf("nodesets belong to different owner are not mergeable %x-%x", set.Owner, owner)
	}
	for path, node := range nodes {
		prev, ok := set.Nodes[path]
		if ok {
			// overwrite happens, revoke the counter
			if prev.IsDeleted() {
				set.deletes -= 1
			} else {
				set.updates -= 1
			}
		}
		set.AddNode([]byte(path), node)
	}
	return nil
}

// AddLeaf adds the provided leaf node into set. TODO(rjl493456442) how can
// we get rid of it?
func (set *NodeSet) AddLeaf(parent common.Hash, blob []byte) {
	set.Leaves = append(set.Leaves, &leaf{Blob: blob, Parent: parent})
}

// Size returns the number of dirty nodes in set.
func (set *NodeSet) Size() (int, int) {
	return set.updates, set.deletes
}

// Hashes returns the hashes of all updated nodes. TODO(rjl493456442) how can
// we get rid of it?
func (set *NodeSet) Hashes() []common.Hash {
	var ret []common.Hash
	for _, node := range set.Nodes {
		ret = append(ret, node.Hash)
	}
	return ret
}

// Summary returns a string-representation of the NodeSet.
func (set *NodeSet) Summary() string {
	var out = new(strings.Builder)
	fmt.Fprintf(out, "nodeset owner: %v\n", set.Owner)
	if set.Nodes != nil {
		for path, n := range set.Nodes {
			// Deletion
			if n.IsDeleted() {
				fmt.Fprintf(out, "  [-]: %x\n", path)
				continue
			}
			// Insertion or update
			fmt.Fprintf(out, "  [+/*]: %x -> %v \n", path, n.Hash)
		}
	}
	for _, n := range set.Leaves {
		fmt.Fprintf(out, "[leaf]: %v\n", n)
	}
	return out.String()
}

// MergedNodeSet represents a merged node set for a group of tries.
type MergedNodeSet struct {
	Sets map[common.Hash]*NodeSet
}

// NewMergedNodeSet initializes an empty merged set.
func NewMergedNodeSet() *MergedNodeSet {
	return &MergedNodeSet{Sets: make(map[common.Hash]*NodeSet)}
}

// NewWithNodeSet constructs a merged nodeset with the provided single set.
func NewWithNodeSet(set *NodeSet) *MergedNodeSet {
	merged := NewMergedNodeSet()
	merged.Merge(set)
	return merged
}

// Merge merges the provided dirty nodes of a trie into the set. The assumption
// is held that no duplicated set belonging to the same trie will be merged twice.
func (set *MergedNodeSet) Merge(other *NodeSet) error {
	subset, present := set.Sets[other.Owner]
	if present {
		return subset.Merge(other.Owner, other.Nodes)
	}
	set.Sets[other.Owner] = other
	return nil
}

// Flatten returns a two-dimensional map for internal nodes.
func (set *MergedNodeSet) Flatten() map[common.Hash]map[string]*Node {
	nodes := make(map[common.Hash]map[string]*Node)
	for owner, set := range set.Sets {
		nodes[owner] = set.Nodes
	}
	return nodes
}
