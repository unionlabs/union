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

package merkletree

import (
	"bytes"
	"hash"
)

// VerifyProof takes a Merkle root, a proofSet, and a proofIndex and returns
// true if the first element of the proof set is a leaf of data in the Merkle
// root. False is returned if the proof set or Merkle root is nil, and if
// 'numLeaves' equals 0.
func VerifyProof(h hash.Hash, merkleRoot []byte, proofSet [][]byte, proofIndex uint64, numLeaves uint64) bool {
	// Return false for nonsense input. A switch statement is used so that the
	// cover tool will reveal if a case is not covered by the test suite. This
	// would not be possible using a single if statement due to the limitations
	// of the cover tool.
	if merkleRoot == nil {
		return false
	}
	if proofIndex >= numLeaves {
		return false
	}

	// In a Merkle tree, every node except the root node has a sibling.
	// Combining the two siblings in the correct order will create the parent
	// node. Each of the remaining hashes in the proof set is a sibling to a
	// node that can be built from all of the previous elements of the proof
	// set. The next node is built by taking:
	//
	//		H(0x01 || sibling A || sibling B)
	//
	// The difficulty of the algorithm lies in determining whether the supplied
	// hash is sibling A or sibling B. This information can be determined by
	// using the proof index and the total number of leaves in the tree.
	//
	// A pair of two siblings forms a subtree. The subtree is complete if it
	// has 1 << height total leaves. When the subtree is complete, the position
	// of the proof index within the subtree can be determined by looking at
	// the bounds of the subtree and determining if the proof index is in the
	// first or second half of the subtree.
	//
	// When the subtree is not complete, either 1 or 0 of the remaining hashes
	// will be sibling B. All remaining hashes after that will be sibling A.
	// This is true because of the way that orphans are merged into the Merkle
	// tree - an orphan at height n is elevated to height n + 1, and only
	// hashed when it is no longer an orphan. Each subtree will therefore merge
	// with at most 1 orphan to the right before becoming an orphan itself.
	// Orphan nodes are always merged with larger subtrees to the left.
	//
	// One vulnerability with the proof verification is that the proofSet may
	// not be long enough. Before looking at an element of proofSet, a check
	// needs to be made that the element exists.

	// The first element of the set is the original data. A sibling at height 1
	// is created by getting the leafSum of the original data.
	height := 0
	if len(proofSet) <= height {
		return false
	}
	sum := leafSum(h, proofSet[height])

	height++

	// While the current subtree (of height 'height') is complete, determine
	// the position of the next sibling using the complete subtree algorithm.
	// 'stableEnd' tells us the ending index of the last full subtree. It gets
	// initialized to 'proofIndex' because the first full subtree was the
	// subtree of height 1, created above (and had an ending index of
	// 'proofIndex').
	stableEnd := proofIndex
	for {
		// Determine if the subtree is complete. This is accomplished by
		// rounding down the proofIndex to the nearest 1 << 'height', adding 1
		// << 'height', and comparing the result to the number of leaves in the
		// Merkle tree.
		subTreeStartIndex := (proofIndex / (1 << uint(height))) * (1 << uint(height)) // round down to the nearest 1 << height
		subTreeEndIndex := subTreeStartIndex + (1 << (uint(height))) - 1              // subtract 1 because the start index is inclusive
		if subTreeEndIndex >= numLeaves {
			// If the Merkle tree does not have a leaf at index
			// 'subTreeEndIndex', then the subtree of the current height is not
			// a complete subtree.
			break
		}
		stableEnd = subTreeEndIndex

		// Determine if the proofIndex is in the first or the second half of
		// the subtree.
		if len(proofSet) <= height {
			return false
		}
		if proofIndex-subTreeStartIndex < 1<<uint(height-1) {
			sum = nodeSum(h, sum, proofSet[height])
		} else {
			sum = nodeSum(h, proofSet[height], sum)
		}
		height++
	}

	// Determine if the next hash belongs to an orphan that was elevated. This
	// is the case IFF 'stableEnd' (the last index of the largest full subtree)
	// is equal to the number of leaves in the Merkle tree.
	if stableEnd != numLeaves-1 {
		if len(proofSet) <= height {
			return false
		}
		sum = nodeSum(h, sum, proofSet[height])
		height++
	}

	// All remaining elements in the proof set will belong to a left sibling.
	for height < len(proofSet) {
		sum = nodeSum(h, proofSet[height], sum)
		height++
	}

	// Compare our calculated Merkle root to the desired Merkle root.
	return bytes.Equal(sum, merkleRoot)
}
