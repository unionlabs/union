// Original Copyright (c) 2015 Nebulous
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

// Package merkletree provides Merkle tree and proof following RFC 6962.
//
// From https://gitlab.com/NebulousLabs/merkletree
package merkletree

import (
	"errors"
	"fmt"
	"hash"
)

// A Tree takes data as leaves and returns the Merkle root. Each call to 'Push'
// adds one leaf to the Merkle tree. Calling 'Root' returns the Merkle root.
// The Tree also constructs proof that a single leaf is a part of the tree. The
// leaf can be chosen with 'SetIndex'. The memory footprint of Tree grows in
// O(log(n)) in the number of leaves.
type Tree struct {
	// The Tree is stored as a stack of subtrees. Each subtree has a height,
	// and is the Merkle root of 2^height leaves. A Tree with 11 nodes is
	// represented as a subtree of height 3 (8 nodes), a subtree of height 1 (2
	// nodes), and a subtree of height 0 (1 node). Head points to the smallest
	// tree. When a new leaf is inserted, it is inserted as a subtree of height
	// 0. If there is another subtree of the same height, both can be removed,
	// combined, and then inserted as a subtree of height n + 1.
	head *subTree
	hash hash.Hash

	// Helper variables used to construct proofs that the data at 'proofIndex'
	// is in the Merkle tree. The proofSet is constructed as elements are being
	// added to the tree. The first element of the proof set is the original
	// data used to create the leaf at index 'proofIndex'. proofTree indicates
	// if the tree will be used to create a merkle proof.
	currentIndex uint64
	proofIndex   uint64
	proofSet     [][]byte
	proofTree    bool

	// The cachedTree flag indicates that the tree is cached, meaning that
	// different code is used in 'Push' for creating a new head subtree. Adding
	// this flag is somewhat gross, but eliminates needing to duplicate the
	// entire 'Push' function when writing the cached tree.
	cachedTree bool
}

// A subTree contains the Merkle root of a complete (2^height leaves) subTree
// of the Tree. 'sum' is the Merkle root of the subTree. If 'next' is not nil,
// it will be a tree with a higher height.
type subTree struct {
	next   *subTree
	height int // Int is okay because a height over 300 is physically unachievable.
	sum    []byte
}

// sum returns the hash of the input data using the specified algorithm.
func sum(h hash.Hash, data ...[]byte) []byte {

	h.Reset()

	for _, d := range data {
		// the Hash interface specifies that Write never returns an error
		_, err := h.Write(d)
		if err != nil {
			panic(err)
		}
	}
	return h.Sum(nil)
}

// leafSum returns the hash created from data inserted to form a leaf. Leaf
// sums are calculated using:
//
//	Hash(0x00 || data)
func leafSum(h hash.Hash, data []byte) []byte {

	//return sum(h, leafHashPrefix, data)
	return sum(h, data)
}

// nodeSum returns the hash created from two sibling nodes being combined into
// a parent node. Node sums are calculated using:
//
//	Hash(0x01 || left sibling sum || right sibling sum)
func nodeSum(h hash.Hash, a, b []byte) []byte {
	//return sum(h, nodeHashPrefix, a, b)
	return sum(h, a, b)
}

// joinSubTrees combines two equal sized subTrees into a larger subTree.
func joinSubTrees(h hash.Hash, a, b *subTree) *subTree {
	// if DEBUG {
	// 	if b.next != a {
	// 		panic("invalid subtree join - 'a' is not paired with 'b'")
	// 	}
	// 	if a.height < b.height {
	// 		panic("invalid subtree presented - height mismatch")
	// 	}
	// }

	return &subTree{
		next:   a.next,
		height: a.height + 1,
		sum:    nodeSum(h, a.sum, b.sum),
	}
}

// New creates a new Tree. The provided hash will be used for all hashing
// operations within the Tree.
func New(h hash.Hash) *Tree {
	return &Tree{
		hash: h,
	}
}

// Prove creates a proof that the leaf at the established index (established by
// SetIndex) is an element of the Merkle tree. Prove will return a nil proof
// set if used incorrectly. Prove does not modify the Tree. Prove can only be
// called if SetIndex has been called previously.
func (t *Tree) Prove() (merkleRoot []byte, proofSet [][]byte, proofIndex uint64, numLeaves uint64) {
	if !t.proofTree {
		panic("wrong usage: can't call prove on a tree if SetIndex wasn't called")
	}

	// Return nil if the Tree is empty, or if the proofIndex hasn't yet been
	// reached.
	if t.head == nil || len(t.proofSet) == 0 {
		return t.Root(), nil, t.proofIndex, t.currentIndex
	}
	proofSet = t.proofSet

	// The set of subtrees must now be collapsed into a single root. The proof
	// set already contains all of the elements that are members of a complete
	// subtree. Of what remains, there will be at most 1 element provided from
	// a sibling on the right, and all of the other proofs will be provided
	// from a sibling on the left. This results from the way orphans are
	// treated. All subtrees smaller than the subtree containing the proofIndex
	// will be combined into a single subtree that gets combined with the
	// proofIndex subtree as a single right sibling. All subtrees larger than
	// the subtree containing the proofIndex will be combined with the subtree
	// containing the proof index as left siblings.

	// Start at the smallest subtree and combine it with larger subtrees until
	// it would be combining with the subtree that contains the proof index. We
	// can recognize the subtree containing the proof index because the height
	// of that subtree will be one less than the current length of the proof
	// set.
	current := t.head
	for current.next != nil && current.next.height < len(proofSet)-1 {
		current = joinSubTrees(t.hash, current.next, current)
	}

	// Sanity check - check that either 'current' or 'current.next' is the
	// subtree containing the proof index.
	// if DEBUG {
	// 	if current.height != len(t.proofSet)-1 && (current.next != nil && current.next.height != len(t.proofSet)-1) {
	// 		panic("could not find the subtree containing the proof index")
	// 	}
	// }

	// If the current subtree is not the subtree containing the proof index,
	// then it must be an aggregate subtree that is to the right of the subtree
	// containing the proof index, and the next subtree is the subtree
	// containing the proof index.
	if current.next != nil && current.next.height == len(proofSet)-1 {
		proofSet = append(proofSet, current.sum)
		current = current.next
	}

	// The current subtree must be the subtree containing the proof index. This
	// subtree does not need an entry, as the entry was created during the
	// construction of the Tree. Instead, skip to the next subtree.
	current = current.next

	// All remaining subtrees will be added to the proof set as a left sibling,
	// completing the proof set.
	for current != nil {
		proofSet = append(proofSet, current.sum)
		current = current.next
	}
	return t.Root(), proofSet, t.proofIndex, t.currentIndex
}

// Push will add data to the set, building out the Merkle tree and Root. The
// tree does not remember all elements that are added, instead only keeping the
// log(n) elements that are necessary to build the Merkle root and keeping the
// log(n) elements necessary to build a proof that a piece of data is in the
// Merkle tree.
func (t *Tree) Push(data []byte) {
	// The first element of a proof is the data at the proof index. If this
	// data is being inserted at the proof index, it is added to the proof set.
	if t.currentIndex == t.proofIndex {
		t.proofSet = append(t.proofSet, data)
	}

	// Hash the data to create a subtree of height 0. The sum of the new node
	// is going to be the data for cached trees, and is going to be the result
	// of calling leafSum() on the data for standard trees. Doing a check here
	// prevents needing to duplicate the entire 'Push' function for the trees.
	t.head = &subTree{
		next:   t.head,
		height: 0,
	}
	if t.cachedTree {
		t.head.sum = data
	} else {
		t.head.sum = leafSum(t.hash, data)
	}

	// Join subTrees if possible.
	t.joinAllSubTrees()

	// Update the index.
	t.currentIndex++

	// Sanity check - From head to tail of the stack, the height should be
	// strictly increasing.
	// if DEBUG {
	// 	current := t.head
	// 	height := current.height
	// 	for current.next != nil {
	// 		current = current.next
	// 		if current.height <= height {
	// 			panic("subtrees are out of order")
	// 		}
	// 		height = current.height
	// 	}
	// }
}

// PushSubTree pushes a cached subtree into the merkle tree. The subtree has to
// be smaller than the smallest subtree in the merkle tree, it has to be
// balanced and it can't contain the element that needs to be proven.  Since we
// can't tell if a subTree is balanced, we can't sanity check for unbalanced
// trees. Therefore an unbalanced tree will cause silent errors, pain and
// misery for the person who wants to debug the resulting error.
func (t *Tree) PushSubTree(height int, sum []byte) error {
	// Check if the cached tree that is pushed contains the element at
	// proofIndex. This is not allowed.
	newIndex := t.currentIndex + 1<<uint64(height)
	if t.proofTree && (t.currentIndex == t.proofIndex ||
		(t.currentIndex < t.proofIndex && t.proofIndex < newIndex)) {
		return errors.New("the cached tree shouldn't contain the element to prove")
	}

	// We can only add the cached tree if its depth is <= the depth of the
	// current subtree.
	if t.head != nil && height > t.head.height {
		return fmt.Errorf("can't add a subtree that is larger than the smallest subtree %v > %v", height, t.head.height)
	}

	// Insert the cached tree as the new head.
	t.head = &subTree{
		height: height,
		next:   t.head,
		sum:    sum,
	}

	// Join subTrees if possible.
	t.joinAllSubTrees()

	// Update the index.
	t.currentIndex = newIndex

	// Sanity check - From head to tail of the stack, the height should be
	// strictly increasing.
	// if DEBUG {
	// 	current := t.head
	// 	height := current.height
	// 	for current.next != nil {
	// 		current = current.next
	// 		if current.height <= height {
	// 			panic("subtrees are out of order")
	// 		}
	// 		height = current.height
	// 	}
	// }
	return nil
}

// Root returns the Merkle root of the data that has been pushed.
func (t *Tree) Root() []byte {
	// If the Tree is empty, return nil.
	if t.head == nil {
		return nil
	}

	// The root is formed by hashing together subTrees in order from least in
	// height to greatest in height. The taller subtree is the first subtree in
	// the join.
	current := t.head
	for current.next != nil {
		current = joinSubTrees(t.hash, current.next, current)
	}
	// Return a copy to prevent leaking a pointer to internal data.
	return append(current.sum[:0:0], current.sum...)
}

// SetIndex will tell the Tree to create a storage proof for the leaf at the
// input index. SetIndex must be called on an empty tree.
func (t *Tree) SetIndex(i uint64) error {
	if t.head != nil {
		return errors.New("cannot call SetIndex on Tree if Tree has not been reset")
	}
	t.proofTree = true
	t.proofIndex = i
	return nil
}

// joinAllSubTrees inserts the subTree at t.head into the Tree. As long as the
// height of the next subTree is the same as the height of the current subTree,
// the two will be combined into a single subTree of height n+1.
func (t *Tree) joinAllSubTrees() {
	for t.head.next != nil && t.head.height == t.head.next.height {
		// Before combining subtrees, check whether one of the subtree hashes
		// needs to be added to the proof set. This is going to be true IFF the
		// subtrees being combined are one height higher than the previous
		// subtree added to the proof set. The height of the previous subtree
		// added to the proof set is equal to len(t.proofSet) - 1.
		if t.head.height == len(t.proofSet)-1 {
			// One of the subtrees needs to be added to the proof set. The
			// subtree that needs to be added is the subtree that does not
			// contain the proofIndex. Because the subtrees being compared are
			// the smallest and rightmost trees in the Tree, this can be
			// determined by rounding the currentIndex down to the number of
			// nodes in the subtree and comparing that index to the proofIndex.
			leaves := uint64(1 << uint(t.head.height))
			mid := (t.currentIndex / leaves) * leaves
			if t.proofIndex < mid {
				t.proofSet = append(t.proofSet, t.head.sum)
			} else {
				t.proofSet = append(t.proofSet, t.head.next.sum)
			}

			// Sanity check - the proofIndex should never be less than the
			// midpoint minus the number of leaves in each subtree.
			// if DEBUG {
			// 	if t.proofIndex < mid-leaves {
			// 		panic("proof being added with weird values")
			// 	}
			// }
		}

		// Join the two subTrees into one subTree with a greater height. Then
		// compare the new subTree to the next subTree.
		t.head = joinSubTrees(t.hash, t.head.next, t.head)
	}
}
