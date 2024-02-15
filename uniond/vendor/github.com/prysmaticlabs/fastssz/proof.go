package ssz

import (
	"bytes"
	"errors"
	"fmt"
	"math"
	"sort"

	"github.com/minio/sha256-simd"
)

// VerifyProof verifies a single merkle branch. It's more
// efficient than VerifyMultiproof for proving one leaf.
func VerifyProof(root []byte, proof *Proof) (bool, error) {
	if len(proof.Hashes) != getPathLength(proof.Index) {
		return false, errors.New("invalid proof length")
	}

	node := proof.Leaf[:]
	tmp := make([]byte, 64)
	for i, h := range proof.Hashes {
		if getPosAtLevel(proof.Index, i) {
			copy(tmp[:32], h[:])
			copy(tmp[32:], node[:])
			node = hashFn(tmp)
		} else {
			copy(tmp[:32], node[:])
			copy(tmp[32:], h[:])
			node = hashFn(tmp)
		}
	}

	return bytes.Equal(root, node), nil
}

// VerifyMultiproof verifies a proof for multiple leaves against the given root.
func VerifyMultiproof(root []byte, proof [][]byte, leaves [][]byte, indices []int) (bool, error) {
	if len(leaves) != len(indices) {
		return false, errors.New("number of leaves and indices mismatch")
	}

	reqIndices := getRequiredIndices(indices)
	if len(reqIndices) != len(proof) {
		return false, fmt.Errorf("number of proof hashes %d and required indices %d mismatch", len(proof), len(reqIndices))
	}

	keys := make([]int, len(indices)+len(reqIndices))
	nk := 0
	// Create database of index -> value (hash)
	// from inputs
	db := make(map[int][]byte)
	for i, leaf := range leaves {
		db[indices[i]] = leaf
		keys[nk] = indices[i]
		nk++
	}
	for i, h := range proof {
		db[reqIndices[i]] = h
		keys[nk] = reqIndices[i]
		nk++
	}
	sort.Sort(sort.Reverse(sort.IntSlice(keys)))

	pos := 0
	tmp := make([]byte, 64)
	for pos < len(keys) {
		k := keys[pos]
		// Root has been reached
		if k == 1 {
			break
		}

		_, hasParent := db[getParent(k)]
		if hasParent {
			pos++
			continue
		}

		left, hasLeft := db[(k|1)^1]
		right, hasRight := db[k|1]
		if !hasRight || !hasLeft {
			return false, fmt.Errorf("proof is missing required nodes, either %d or %d", (k|1)^1, k|1)
		}

		copy(tmp[:32], left[:])
		copy(tmp[32:], right[:])
		db[getParent(k)] = hashFn(tmp)
		keys = append(keys, getParent(k))

		pos++
	}

	res, ok := db[1]
	if !ok {
		return false, fmt.Errorf("root was not computed during proof verification")
	}

	return bytes.Equal(res, root), nil
}

// Returns the position (i.e. false for left, true for right)
// of an index at a given level.
// Level 0 is the actual index's level, Level 1 is the position
// of the parent, etc.
func getPosAtLevel(index int, level int) bool {
	return (index & (1 << level)) > 0
}

// Returns the length of the path to a node represented by its generalized index.
func getPathLength(index int) int {
	return int(math.Log2(float64(index)))
}

// Returns the generalized index for a node's sibling.
func getSibling(index int) int {
	return index ^ 1
}

// Returns the generalized index for a node's parent.
func getParent(index int) int {
	return index >> 1
}

// Returns generalized indices for all nodes in the tree that are
// required to prove the given leaf indices. The returned indices
// are in a decreasing order.
func getRequiredIndices(leafIndices []int) []int {
	exists := struct{}{}
	// Sibling hashes needed for verification
	required := make(map[int]struct{})
	// Set of hashes that will be computed
	// on the path from leaf to root.
	computed := make(map[int]struct{})
	leaves := make(map[int]struct{})

	for _, leaf := range leafIndices {
		leaves[leaf] = exists
		cur := leaf
		for cur > 1 {
			sibling := getSibling(cur)
			parent := getParent(cur)
			required[sibling] = exists
			computed[parent] = exists
			cur = parent
		}
	}

	requiredList := make([]int, 0, len(required))
	// Remove computed indices from required ones
	for r := range required {
		_, isComputed := computed[r]
		_, isLeaf := leaves[r]
		if !isComputed && !isLeaf {
			requiredList = append(requiredList, r)
		}
	}

	sort.Sort(sort.Reverse(sort.IntSlice(requiredList)))
	return requiredList
}

func hashFn(data []byte) []byte {
	res := sha256.Sum256(data)
	return res[:]
}
