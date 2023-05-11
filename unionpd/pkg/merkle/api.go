package merkle

import (
	"cometbls-prover/pkg/sha256"
	"github.com/consensys/gnark/frontend"
	"math"
)

const (
	LeafPrefix  = 0
	InnerPrefix = 1
)

type MerkleTreeAPI struct {
	api frontend.API
}

func NewMerkleTreeAPI(api frontend.API) *MerkleTreeAPI {
	return &MerkleTreeAPI{api: api}
}

func (m *MerkleTreeAPI) LeafHash(leaf []frontend.Variable, size frontend.Variable) []frontend.Variable {
	preimage := make([]frontend.Variable, 1+len(leaf))
	// Leaf prefix
	preimage[0] = LeafPrefix
	for i := 0; i < len(leaf); i++ {
		preimage[i+1] = leaf[i]
	}
	sha := sha256.NewSHA256(m.api)
	hash := sha.Hash(preimage[:], m.api.Add(size, 1))
	return hash[:]
}

func (m *MerkleTreeAPI) InnerHash(left []frontend.Variable, right []frontend.Variable) []frontend.Variable {
	preimage := make([]frontend.Variable, 1+len(left)+len(right))
	// Inner prefix
	preimage[0] = InnerPrefix
	for i := 0; i < len(left); i++ {
		preimage[i+1] = left[i]
	}
	for i := 0; i < len(right); i++ {
		preimage[i+len(left)+1] = right[i]
	}
	sha := sha256.NewSHA256(m.api)
	hash := sha.Hash(preimage[:], 1+len(left)+len(right))
	return hash[:]
}

// Compute merkle root in place at leafHashes[0]
func (m *MerkleTreeAPI) RootHash(leafHashes [][]frontend.Variable, size frontend.Variable) []frontend.Variable {
	maxLeaves := len(leafHashes)
	for i := 0; i < int(math.Log2(float64(maxLeaves))); i++ {
		r := size
		w := 0
		for j := 0; j < maxLeaves/int(math.Pow(2, float64(i))); j += 2 {
			left := leafHashes[j]
			right := leafHashes[j+1]
			root := m.InnerHash(left, right)
			isOrphan := m.api.Or(m.api.IsZero(r), m.api.IsZero(m.api.Sub(r, 1)))
			for k := 0; k < 32; k++ {
				leafHashes[w][k] = m.api.Select(isOrphan, left[k], root[k])
			}
			size = m.api.Select(m.api.Or(isOrphan, m.api.IsZero(size)), size, m.api.Sub(size, 1))
			r = m.api.Select(m.api.IsZero(r), r, m.api.Sub(r, 1))
			r = m.api.Select(m.api.IsZero(r), r, m.api.Sub(r, 1))
			w += 1
		}
	}
	return leafHashes[0]
}
