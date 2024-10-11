// Package trie defines utilities for sparse merkle tries for Ethereum consensus.
package trie

import (
	"bytes"
	"encoding/binary"
	"fmt"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	protodb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// SparseMerkleTrie implements a sparse, general purpose Merkle trie to be used
// across Ethereum consensus functionality.
type SparseMerkleTrie struct {
	depth         uint
	branches      [][][]byte
	originalItems [][]byte // list of provided items before hashing them into leaves.
}

// NewTrie returns a new merkle trie filled with zerohashes to use.
func NewTrie(depth uint64) (*SparseMerkleTrie, error) {
	var zeroBytes [32]byte
	items := [][]byte{zeroBytes[:]}
	return GenerateTrieFromItems(items, depth)
}

// CreateTrieFromProto creates a Sparse Merkle Trie from its corresponding merkle trie.
func CreateTrieFromProto(trieObj *protodb.SparseMerkleTrie) (*SparseMerkleTrie, error) {
	trie := &SparseMerkleTrie{
		depth:         uint(trieObj.Depth),
		originalItems: trieObj.OriginalItems,
	}
	branches := make([][][]byte, len(trieObj.Layers))
	for i, layer := range trieObj.Layers {
		branches[i] = layer.Layer
	}
	trie.branches = branches

	if err := trie.validate(); err != nil {
		return nil, errors.Wrap(err, "invalid sparse merkle trie")
	}

	return trie, nil
}

func (m *SparseMerkleTrie) validate() error {
	if len(m.branches) == 0 {
		return errors.New("no branches")
	}
	if len(m.branches[len(m.branches)-1]) == 0 {
		return errors.New("invalid branches provided")
	}
	if m.depth >= uint(len(m.branches)) {
		return errors.New("depth is greater than or equal to number of branches")
	}
	if m.depth >= 63 {
		return errors.New("supported merkle trie depth exceeded (max uint64 depth is 63, " +
			"theoretical max sparse merkle trie depth is 64)") // PowerOf2 would overflow.
	}

	return nil
}

// GenerateTrieFromItems constructs a Merkle trie from a sequence of byte slices.
func GenerateTrieFromItems(items [][]byte, depth uint64) (*SparseMerkleTrie, error) {
	if len(items) == 0 {
		return nil, errors.New("no items provided to generate Merkle trie")
	}
	if depth >= 63 {
		return nil, errors.New("supported merkle trie depth exceeded (max uint64 depth is 63, " +
			"theoretical max sparse merkle trie depth is 64)") // PowerOf2 would overflow
	}

	leaves := items
	layers := make([][][]byte, depth+1)
	transformedLeaves := make([][]byte, len(leaves))
	for i := range leaves {
		arr := bytesutil.ToBytes32(leaves[i])
		transformedLeaves[i] = arr[:]
	}
	layers[0] = transformedLeaves
	for i := uint64(0); i < depth; i++ {
		if len(layers[i])%2 == 1 {
			layers[i] = append(layers[i], ZeroHashes[i][:])
		}
		updatedValues := make([][]byte, 0)
		for j := 0; j < len(layers[i]); j += 2 {
			concat := hash.Hash(append(layers[i][j], layers[i][j+1]...))
			updatedValues = append(updatedValues, concat[:])
		}
		layers[i+1] = updatedValues
	}
	return &SparseMerkleTrie{
		branches:      layers,
		originalItems: items,
		depth:         uint(depth),
	}, nil
}

// Items returns the original items passed in when creating the Merkle trie.
func (m *SparseMerkleTrie) Items() [][]byte {
	return m.originalItems
}

// HashTreeRoot of the Merkle trie as defined in the deposit contract.
//
//	Spec Definition:
//	 sha256(concat(node, self.to_little_endian_64(self.deposit_count), slice(zero_bytes32, start=0, len=24)))
func (m *SparseMerkleTrie) HashTreeRoot() ([32]byte, error) {
	var enc [32]byte
	depositCount := uint64(len(m.originalItems))
	if len(m.originalItems) == 1 && bytes.Equal(m.originalItems[0], ZeroHashes[0][:]) {
		// Accounting for empty tries
		depositCount = 0
	}
	binary.LittleEndian.PutUint64(enc[:], depositCount)
	return hash.Hash(append(m.branches[len(m.branches)-1][0], enc[:]...)), nil
}

// Insert an item into the trie.
func (m *SparseMerkleTrie) Insert(item []byte, index int) error {
	if index < 0 {
		return fmt.Errorf("negative index provided: %d", index)
	}
	for index >= len(m.branches[0]) {
		m.branches[0] = append(m.branches[0], ZeroHashes[0][:])
	}
	someItem := bytesutil.ToBytes32(item)
	m.branches[0][index] = someItem[:]
	if index >= len(m.originalItems) {
		m.originalItems = append(m.originalItems, someItem[:])
	} else {
		m.originalItems[index] = someItem[:]
	}
	currentIndex := index
	root := bytesutil.ToBytes32(item)
	for i := 0; i < int(m.depth); i++ {
		isLeft := currentIndex%2 == 0
		neighborIdx := currentIndex ^ 1
		var neighbor []byte
		if neighborIdx >= len(m.branches[i]) {
			neighbor = ZeroHashes[i][:]
		} else {
			neighbor = m.branches[i][neighborIdx]
		}
		if isLeft {
			parentHash := hash.Hash(append(root[:], neighbor...))
			root = parentHash
		} else {
			parentHash := hash.Hash(append(neighbor, root[:]...))
			root = parentHash
		}
		parentIdx := currentIndex / 2
		if len(m.branches[i+1]) == 0 || parentIdx >= len(m.branches[i+1]) {
			newItem := root
			m.branches[i+1] = append(m.branches[i+1], newItem[:])
		} else {
			newItem := root
			m.branches[i+1][parentIdx] = newItem[:]
		}
		currentIndex = parentIdx
	}
	return nil
}

// MerkleProof computes a proof from a trie's branches using a Merkle index.
func (m *SparseMerkleTrie) MerkleProof(index int) ([][]byte, error) {
	if index < 0 {
		return nil, fmt.Errorf("merkle index is negative: %d", index)
	}
	leaves := m.branches[0]
	if index >= len(leaves) {
		return nil, fmt.Errorf("merkle index out of range in trie, max range: %d, received: %d", len(leaves), index)
	}
	merkleIndex := uint(index)
	proof := make([][]byte, m.depth+1)
	for i := uint(0); i < m.depth; i++ {
		subIndex := (merkleIndex / (1 << i)) ^ 1
		if subIndex < uint(len(m.branches[i])) {
			item := bytesutil.ToBytes32(m.branches[i][subIndex])
			proof[i] = item[:]
		} else {
			proof[i] = ZeroHashes[i][:]
		}
	}
	var enc [32]byte
	binary.LittleEndian.PutUint64(enc[:], uint64(len(m.originalItems)))
	proof[len(proof)-1] = enc[:]
	return proof, nil
}

// ToProto converts the underlying trie into its corresponding
// proto object
func (m *SparseMerkleTrie) ToProto() *protodb.SparseMerkleTrie {
	trie := &protodb.SparseMerkleTrie{
		Depth:         uint64(m.depth),
		Layers:        make([]*protodb.TrieLayer, len(m.branches)),
		OriginalItems: m.originalItems,
	}
	for i, l := range m.branches {
		trie.Layers[i] = &protodb.TrieLayer{
			Layer: l,
		}
	}
	return trie
}

// VerifyMerkleProofWithDepth verifies a Merkle branch against a root of a trie.
func VerifyMerkleProofWithDepth(root, item []byte, merkleIndex uint64, proof [][]byte, depth uint64) bool {
	if uint64(len(proof)) != depth+1 {
		return false
	}
	node := bytesutil.ToBytes32(item)
	for i := uint64(0); i <= depth; i++ {
		if (merkleIndex & 1) == 1 {
			node = hash.Hash(append(proof[i], node[:]...))
		} else {
			node = hash.Hash(append(node[:], proof[i]...))
		}
		merkleIndex /= 2
	}
	return bytes.Equal(root, node[:])
}

// VerifyMerkleProof given a trie root, a leaf, the generalized merkle index
// of the leaf in the trie, and the proof itself.
func VerifyMerkleProof(root, item []byte, merkleIndex uint64, proof [][]byte) bool {
	if len(proof) == 0 {
		return false
	}
	return VerifyMerkleProofWithDepth(root, item, merkleIndex, proof, uint64(len(proof)-1))
}

// Copy performs a deep copy of the trie.
func (m *SparseMerkleTrie) Copy() *SparseMerkleTrie {
	dstBranches := make([][][]byte, len(m.branches))
	for i1, srcB1 := range m.branches {
		dstBranches[i1] = bytesutil.SafeCopy2dBytes(srcB1)
	}

	return &SparseMerkleTrie{
		depth:         m.depth,
		branches:      dstBranches,
		originalItems: bytesutil.SafeCopy2dBytes(m.originalItems),
	}
}

// NumOfItems returns the num of items stored in
// the sparse merkle trie. We handle a special case
// where if there is only one item stored and it is an
// empty 32-byte root.
func (m *SparseMerkleTrie) NumOfItems() int {
	var zeroBytes [32]byte
	if len(m.originalItems) == 1 && bytes.Equal(m.originalItems[0], zeroBytes[:]) {
		return 0
	}
	return len(m.originalItems)
}
