package merkle

import (
	mimc "galois/pkg/emulatedmimc"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	"github.com/consensys/gnark/std/math/emulated"
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

// Union whitepaper: (11) H_leaf
func (m *MerkleTreeAPI) LeafHash(field *emulated.Field[sw_bn254.ScalarField], leaf []*emulated.Element[sw_bn254.ScalarField]) *emulated.Element[sw_bn254.ScalarField] {
	preimage := make([]*emulated.Element[sw_bn254.ScalarField], 1+len(leaf))
	// Leaf prefix
	preimage[0] = field.Zero()
	for i := 0; i < len(leaf); i++ {
		preimage[i+1] = leaf[i]
	}

	mimc, err := mimc.NewMiMC[sw_bn254.ScalarField](field)
	if err != nil {
		panic(err)
	}
	mimc.Write(preimage...)

	return mimc.Sum()
}

// Union whitepaper: (11) H_inner
func (m *MerkleTreeAPI) InnerHash(field *emulated.Field[sw_bn254.ScalarField], left *emulated.Element[sw_bn254.ScalarField], right *emulated.Element[sw_bn254.ScalarField]) *emulated.Element[sw_bn254.ScalarField] {
	mimc, err := mimc.NewMiMC[sw_bn254.ScalarField](field)
	if err != nil {
		panic(err)
	}

	mimc.Write(field.One(), left, right)

	return mimc.Sum()
}

// Union whitepaper: (11) merkle_root
//
// Compute merkle root in place at leafHashes[0]
func (m *MerkleTreeAPI) RootHash(field *emulated.Field[sw_bn254.ScalarField], leafHashes []*emulated.Element[sw_bn254.ScalarField], size frontend.Variable) *emulated.Element[sw_bn254.ScalarField] {
	initialSize := size
	maxLeaves := len(leafHashes)
	for i := 0; i < int(math.Ceil(math.Log2(float64(maxLeaves)))); i++ {
		r := size
		w := 0
		for j := 0; j < int(math.Ceil(float64(maxLeaves)/math.Pow(2, float64(i)))); j += 2 {
			left := leafHashes[j]
			right := leafHashes[j+1]
			root := m.InnerHash(field, left, right)
			isOrphan := m.api.Or(m.api.IsZero(r), m.api.IsZero(m.api.Sub(r, 1)))
			leafHashes[w] = field.Select(isOrphan, left, root)
			size = m.api.Select(m.api.IsZero(size), 0, m.api.Select(isOrphan, size, m.api.Sub(size, 1)))
			r = m.api.Select(m.api.IsZero(r), r, m.api.Sub(r, 1))
			r = m.api.Select(m.api.IsZero(r), r, m.api.Sub(r, 1))
			w += 1
		}
	}
	return field.Select(m.api.IsZero(initialSize), field.Zero(), leafHashes[0])
}
