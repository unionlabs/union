package merkle

import (
	"math"

	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/hash/mimc"
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

// Union whitepaper: (11) H_leaf
func (m *MerkleTreeAPI) LeafHash(leaf []frontend.Variable) frontend.Variable {
	preimage := make([]frontend.Variable, 1+len(leaf))
	// Leaf prefix
	preimage[0] = LeafPrefix
	for i := 0; i < len(leaf); i++ {
		preimage[i+1] = leaf[i]
	}
	mimc, err := mimc.NewMiMC(m.api)
	if err != nil {
		panic(err)
	}
	mimc.Write(preimage[:]...)
	return mimc.Sum()
}

// Union whitepaper: (11) H_inner
func (m *MerkleTreeAPI) InnerHash(left frontend.Variable, right frontend.Variable) frontend.Variable {
	mimc, err := mimc.NewMiMC(m.api)
	if err != nil {
		panic(err)
	}
	mimc.Write(InnerPrefix, left, right)
	return mimc.Sum()
}

// Union whitepaper: (11) merkle_root
//
// Compute merkle root in place at leafHashes[0]
func (m *MerkleTreeAPI) RootHash(leafHashes []frontend.Variable, size frontend.Variable) frontend.Variable {
	initialSize := size
	maxLeaves := len(leafHashes)
	for i := 0; i < int(math.Ceil(math.Log2(float64(maxLeaves)))); i++ {
		r := size
		w := 0
		for j := 0; j < int(math.Ceil(float64(maxLeaves)/math.Pow(2, float64(i)))); j += 2 {
			left := leafHashes[j]
			right := leafHashes[j+1]
			root := m.InnerHash(left, right)
			isOrphan := m.api.Or(m.api.IsZero(r), m.api.IsZero(m.api.Sub(r, 1)))
			leafHashes[w] = m.api.Select(isOrphan, left, root)
			size = m.api.Select(m.api.IsZero(size), 0, m.api.Select(isOrphan, size, m.api.Sub(size, 1)))
			r = m.api.Select(m.api.IsZero(r), r, m.api.Sub(r, 1))
			r = m.api.Select(m.api.IsZero(r), r, m.api.Sub(r, 1))
			w += 1
		}
	}
	mimc, err := mimc.NewMiMC(m.api)
	if err != nil {
		panic(err)
	}
	emptyHash := mimc.Sum()
	return m.api.Select(m.api.IsZero(initialSize), emptyHash, leafHashes[0])
}
