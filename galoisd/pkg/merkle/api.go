package merkle

import (
	// "encoding/binary"
	"encoding/binary"
	mimc "galois/pkg/emulatedmimc"
	realmimc "github.com/consensys/gnark/std/hash/mimc"
	"math"

	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	"github.com/consensys/gnark/std/math/emulated"
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

func encodeLimbs(api frontend.API, limbs []frontend.Variable) []byte {
	limb0, _ := api.Compiler().ConstantValue(limbs[0])
	limb1, _ := api.Compiler().ConstantValue(limbs[1])
	limb2, _ := api.Compiler().ConstantValue(limbs[2])
	limb3, _ := api.Compiler().ConstantValue(limbs[3])

	var outbytes [32]byte

	binary.BigEndian.PutUint64(outbytes[24:32], limb0.Uint64())
	binary.BigEndian.PutUint64(outbytes[16:24], limb1.Uint64())
	binary.BigEndian.PutUint64(outbytes[8:16], limb2.Uint64())
	binary.BigEndian.PutUint64(outbytes[0:8], limb3.Uint64())

	return outbytes[:]
}

// Union whitepaper: (11) H_leaf
func (m *MerkleTreeAPI) LeafHash(leaf []frontend.Variable) frontend.Variable {
	field, err := emulated.NewField[sw_bn254.ScalarField](m.api)
	preimage := make([]*emulated.Element[sw_bn254.ScalarField], 1+len(leaf))
	// Leaf prefix
	preimage[0] = field.NewElement(LeafPrefix)
	for i := 0; i < len(leaf); i++ {
		preimage[i+1] = field.NewElement(leaf[i])
	}

	mimc, err := mimc.NewMiMC[sw_bn254.ScalarField](field)
	if err != nil {
		panic(err)
	}
	mimc.Write(preimage...)

	encoded := encodeLimbs(m.api, mimc.Sum().Limbs)

	return encoded
}

// Union whitepaper: (11) H_inner
func (m *MerkleTreeAPI) InnerHash(left frontend.Variable, right frontend.Variable) frontend.Variable {
	field, err := emulated.NewField[sw_bn254.ScalarField](m.api)
	if err != nil {
		panic(err)
	}
	mimc, err := mimc.NewMiMC[sw_bn254.ScalarField](field)
	if err != nil {
		panic(err)
	}

	mimc.Write(field.NewElement(left))

	// note that these values are not the same
	val, _ := m.api.Compiler().ConstantValue(m.api.Add(left, left))
	m.api.Println(uint64(val.Bits()[0]), uint64(val.Bits()[1]), uint64(val.Bits()[2]), uint64(val.Bits()[3]))
	m.api.Println(field.Sum(field.NewElement(left), field.NewElement(left)).Limbs)
	m.api.Println(41414141)

	mimc2, _ := realmimc.NewMiMC(m.api)
	mimc2.Write(left)

	limbs := mimc.Sum().Limbs
	encoded := encodeLimbs(m.api, limbs)

	return encoded
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
	field, err := emulated.NewField[sw_bn254.ScalarField](m.api)
	if err != nil {
		panic(err)
	}
	mimc, err := mimc.NewMiMC[sw_bn254.ScalarField](field)
	if err != nil {
		panic(err)
	}
	emptyHash := mimc.Sum()
	return m.api.Select(m.api.IsZero(initialSize), emptyHash, leafHashes[0])
}
