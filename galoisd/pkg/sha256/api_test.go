package sha256

import (
	"crypto/sha256"
	"fmt"
	"testing"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/test"
)

const compressThreshold = 1000

const MaxPreimageLength = 32

const ImageLength = 32

type sha256Circuit struct {
	PreimageLength frontend.Variable
	Preimage       [MaxPreimageLength]frontend.Variable
	Image          [ImageLength]frontend.Variable
}

func (c *sha256Circuit) Define(api frontend.API) error {
	api.AssertIsLessOrEqual(c.PreimageLength, MaxPreimageLength)
	hash := NewSHA256(api)
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
	for i := 0; i < 10; i++ {
		t.Run(fmt.Sprintf("PreimageLength = %d", i), func(t *testing.T) {
			message := make([]byte, MaxPreimageLength)

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
