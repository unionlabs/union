package g2

import (
	// "crypto/rand"
	cometbft_bn254 "github.com/cometbft/cometbft/crypto/bn254"
	"github.com/consensys/gnark-crypto/ecc"
	curve "github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark/backend"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/algebra/emulated/fields_bn254"
	gadget "github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	"github.com/consensys/gnark/test"
	"math/big"
	"testing"
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

func FuzzMapToCurve(f *testing.F) {
	f.Fuzz(func(t *testing.T, message []byte) {
		t.Parallel()
		messageX, messageY := cometbft_bn254.HashToField2(message)
		var messagePoint curve.E2
		messagePoint.A0.SetBigInt(messageX.BigInt(new(big.Int)))
		messagePoint.A1.SetBigInt(messageY.BigInt(new(big.Int)))
		test.NewAssert(t).ProverSucceeded(
			&MapToCurve{},
			&MapToCurve{
				Preimage: fields_bn254.FromE2(&messagePoint),
				Image:    gadget.NewG2Affine(curve.MapToCurve2(&messagePoint)),
			},
			test.WithCurves(ecc.BN254),
			test.NoFuzzing(),
			test.WithCurves(ecc.BN254),
			test.WithBackends(backend.GROTH16),
		)
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

func FuzzMapToG2(f *testing.F) {
	f.Fuzz(func(t *testing.T, message []byte) {
		t.Parallel()
		messageX, messageY := cometbft_bn254.HashToField2(message[:])
		var messagePoint curve.E2
		messagePoint.A0.SetBigInt(messageX.BigInt(new(big.Int)))
		messagePoint.A1.SetBigInt(messageY.BigInt(new(big.Int)))
		test.NewAssert(t).ProverSucceeded(
			&MapToG2{},
			&MapToG2{
				Preimage: fields_bn254.FromE2(&messagePoint),
				Image:    gadget.NewG2Affine(curve.MapToG2(messagePoint)),
			},
			test.WithCurves(ecc.BN254),
			test.NoFuzzing(),
			test.WithCurves(ecc.BN254),
			test.WithBackends(backend.GROTH16),
		)
	})
}
