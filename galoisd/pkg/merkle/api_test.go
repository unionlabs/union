package merkle

import (
	"fmt"
	// "math/big"
	// "math/rand"
	"testing"

	"github.com/cometbft/cometbft/crypto/merkle"
	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	"github.com/consensys/gnark/std/math/emulated"
	"github.com/consensys/gnark/test"
	"github.com/stretchr/testify/assert"
)

const MaxLeaves = 128

type MerkleRoot struct {
	Root       frontend.Variable
	LeavesData [MaxLeaves]frontend.Variable
	NbOfLeaves frontend.Variable
}

func (c *MerkleRoot) Define(api frontend.API) error {
	field, _ := emulated.NewField[sw_bn254.ScalarField](api)
	merkle := NewMerkleTreeAPI(api)
	leavesHash := make([]*emulated.Element[sw_bn254.ScalarField], MaxLeaves)
	for i := 0; i < MaxLeaves; i++ {
		leavesHash[i] = merkle.LeafHash([]*emulated.Element[sw_bn254.ScalarField]{field.NewElement(c.LeavesData[i])})
	}
	field.AssertIsEqual(field.NewElement(c.Root), merkle.RootHash(leavesHash, c.NbOfLeaves))
	return nil
}

func TestEmptyMerkleRoot(t *testing.T) {
	leaves := [MaxLeaves]frontend.Variable{}
	for i := 0; i < MaxLeaves; i++ {
		leaves[i] = 0xCAFEBABE
	}
	err := test.IsSolved(
		&MerkleRoot{},
		&MerkleRoot{
			Root:       merkle.MimcHashFromByteSlices([][]byte{}),
			LeavesData: leaves,
			NbOfLeaves: 0,
		},
		ecc.BN254.ScalarField(),
	)
	assert.NoError(t, err)
}

func TestMerkleRoot(t *testing.T) {
	t.Parallel()
	for i := 1; i < 3; i++ {
		k := i
		t.Run(fmt.Sprintf("%d", k), func(t *testing.T) {
			t.Parallel()
			leaves := [MaxLeaves][]byte{}
			circuitLeaves := [MaxLeaves]frontend.Variable{}
			for j := 0; j < k; j++ {
				var leafValue fr.Element
				_, err := leafValue.SetRandom()
				assert.NoError(t, err)
				leaves[j] = leafValue.Marshal()
				circuitLeaves[j] = leafValue.Marshal()
			}
			for j := k; j < MaxLeaves; j++ {
				circuitLeaves[j] = 0
			}
			err := test.IsSolved(
				&MerkleRoot{},
				&MerkleRoot{
					Root:       merkle.MimcHashFromByteSlices(leaves[:k]),
					LeavesData: circuitLeaves,
					NbOfLeaves: k,
				},
				ecc.BN254.ScalarField(),
			)
			assert.NoError(t, err)
		})
	}
}

// func FuzzMerkleRoot(f *testing.F) {
// 	f.Fuzz(func(t *testing.T, seed int64) {
// 		t.Parallel()
// 		rng := rand.New(rand.NewSource(seed))
// 		k := max(1, rng.Intn(MaxLeaves))
// 		leaves := [MaxLeaves][]byte{}
// 		circuitLeaves := [MaxLeaves]frontend.Variable{}
// 		buff := make([]byte, 32)
// 		for j := 0; j < k; j++ {
// 			_, err := rng.Read(buff)
// 			assert.NoError(t, err)
// 			var leafValue fr.Element
// 			leafValue.SetBigInt(new(big.Int).SetBytes(buff))
// 			leaves[j] = leafValue.Marshal()
// 			circuitLeaves[j] = leafValue.Marshal()
// 		}
// 		for j := k; j < MaxLeaves; j++ {
// 			circuitLeaves[j] = 0
// 		}
// 		err := test.IsSolved(
// 			&MerkleRoot{},
// 			&MerkleRoot{
// 				Root:       merkle.MimcHashFromByteSlices(leaves[:k]),
// 				LeavesData: circuitLeaves,
// 				NbOfLeaves: k,
// 			},
// 			ecc.BN254.ScalarField(),
// 		)
// 		assert.NoError(t, err)
// 	})
// }
