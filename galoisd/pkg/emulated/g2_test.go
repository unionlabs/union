package g2

import (
	"crypto/rand"
	"fmt"
	"math/big"
	"testing"

	cometbft_bn254 "github.com/cometbft/cometbft/crypto/bn254"
	"github.com/consensys/gnark-crypto/ecc"
	curve "github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/algebra/emulated/fields_bn254"
	gadget "github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	"github.com/consensys/gnark/test"
	"github.com/stretchr/testify/assert"
)

type MapToCurve struct {
	Preimage fields_bn254.E2
	Image    gadget.G2Affine
}

func (c *MapToCurve) Define(api frontend.API) error {
	emulated, err := NewEmulatedAPI(api)
	if err != nil {
		return err
	}
	emulated.AssertIsEqual(&c.Image, emulated.MapToCurve(&c.Preimage))
	return nil
}

func TestMapToCurve(t *testing.T) {
	t.Parallel()
	for i := 0; i < 100; i++ {
		k := i
		t.Run(fmt.Sprintf("%d", k), func(t *testing.T) {
			t.Parallel()
			message := make([]byte, 256)
			_, err := rand.Read(message)
			assert.NoError(t, err)
			messageX, messageY := cometbft_bn254.HashToField2(message)
			var messagePoint curve.E2
			messagePoint.A0.SetBigInt(messageX.BigInt(new(big.Int)))
			messagePoint.A1.SetBigInt(messageY.BigInt(new(big.Int)))

			err = test.IsSolved(
				&MapToCurve{},
				&MapToCurve{
					Preimage: fields_bn254.FromE2(&messagePoint),
					Image:    gadget.NewG2Affine(curve.MapToCurve2(&messagePoint)),
				},
				ecc.BN254.ScalarField(),
			)
			assert.NoError(t, err)
		})
	}
}

func FuzzMapToCurve(f *testing.F) {
	f.Fuzz(func(t *testing.T, message []byte) {
		t.Parallel()
		messageX, messageY := cometbft_bn254.HashToField2(message)
		var messagePoint curve.E2
		messagePoint.A0.SetBigInt(messageX.BigInt(new(big.Int)))
		messagePoint.A1.SetBigInt(messageY.BigInt(new(big.Int)))
		err := test.IsSolved(
			&MapToCurve{},
			&MapToCurve{
				Preimage: fields_bn254.FromE2(&messagePoint),
				Image:    gadget.NewG2Affine(curve.MapToCurve2(&messagePoint)),
			},
			ecc.BN254.ScalarField(),
		)
		assert.NoError(t, err)
	})
}

type MapToG2 struct {
	Preimage fields_bn254.E2
	Image    gadget.G2Affine
}

func (c *MapToG2) Define(api frontend.API) error {
	emulated, err := NewEmulatedAPI(api)
	if err != nil {
		return err
	}
	emulated.AssertIsEqual(&c.Image, emulated.MapToG2(&c.Preimage))
	return nil
}

func TestMapToG2(t *testing.T) {
	t.Parallel()
	for i := 0; i < 100; i++ {
		k := i
		t.Run(fmt.Sprintf("%d", k), func(t *testing.T) {
			t.Parallel()
			message := make([]byte, 256)
			_, err := rand.Read(message)
			assert.NoError(t, err)
			messageX, messageY := cometbft_bn254.HashToField2(message)
			var messagePoint curve.E2
			messagePoint.A0.SetBigInt(messageX.BigInt(new(big.Int)))
			messagePoint.A1.SetBigInt(messageY.BigInt(new(big.Int)))
			err = test.IsSolved(
				&MapToG2{},
				&MapToG2{
					Preimage: fields_bn254.FromE2(&messagePoint),
					Image:    gadget.NewG2Affine(curve.MapToG2(messagePoint)),
				},
				ecc.BN254.ScalarField(),
			)
			assert.NoError(t, err)
		})
	}
}

func FuzzMapToG2(f *testing.F) {
	f.Fuzz(func(t *testing.T, message []byte) {
		t.Parallel()
		messageX, messageY := cometbft_bn254.HashToField2(message)
		var messagePoint curve.E2
		messagePoint.A0.SetBigInt(messageX.BigInt(new(big.Int)))
		messagePoint.A1.SetBigInt(messageY.BigInt(new(big.Int)))
		err := test.IsSolved(
			&MapToG2{},
			&MapToG2{
				Preimage: fields_bn254.FromE2(&messagePoint),
				Image:    gadget.NewG2Affine(curve.MapToG2(messagePoint)),
			},
			ecc.BN254.ScalarField(),
		)
		assert.NoError(t, err)
	})
}
