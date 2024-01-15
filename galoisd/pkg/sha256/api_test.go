package sha256

import (
	"crypto/rand"
	"crypto/sha256"
	"fmt"
	"testing"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/test"
)

const compressThreshold = 1000

const ImageLength = 32

const MaxBlocks = 1

const MaxPreimageLength = 55 + (MaxBlocks-1)*64

type sha256Circuit struct {
	PreimageLength frontend.Variable
	Preimage       [MaxPreimageLength]frontend.Variable
	Image          [ImageLength]frontend.Variable
}

func (c *sha256Circuit) Define(api frontend.API) error {
	api.AssertIsLessOrEqual(c.PreimageLength, MaxPreimageLength)
	hash := NewSHA256(api, MaxBlocks)
	actualPreimage := make([]frontend.Variable, MaxPreimageLength)
	for i := 0; i < MaxPreimageLength; i++ {
		actualPreimage[i] = c.Preimage[i]
	}
	image := hash.Hash(actualPreimage, c.PreimageLength)
	for i := 0; i < ImageLength; i++ {
		api.AssertIsEqual(image[i], c.Image[i])
	}
	return nil
}

func TestSha256(t *testing.T) {
	t.Parallel()
	for i := 0; i < MaxPreimageLength; i++ {
		k := i
		t.Run(fmt.Sprintf("PreimageLength = %d", k), func(t *testing.T) {
			t.Parallel()
			message := make([]byte, k)
			_, err := rand.Read(message)
			if err != nil {
				t.Fatal(err)
			}

			nativeHasher := sha256.New()
			nativeHasher.Write(message)
			final := nativeHasher.Sum(nil)

			var preimage [MaxPreimageLength]frontend.Variable
			for i := 0; i < MaxPreimageLength; i++ {
				if i < len(message) {
					preimage[i] = message[i]
				} else {
					preimage[i] = 0
				}
			}

			var image [ImageLength]frontend.Variable
			for i := 0; i < ImageLength; i++ {
				image[i] = final[i]
			}

			circuit := sha256Circuit{}
			assignment := sha256Circuit{
				Preimage:       preimage,
				PreimageLength: len(message),
				Image:          image,
			}
			test.NewAssert(t).ProverSucceeded(
				&circuit,
				&assignment,
				test.WithCurves(ecc.BN254),
				test.NoFuzzing(),
				test.WithCurves(ecc.BN254),
				test.WithBackends(backend.GROTH16),
			)
		})
	}
}
