package g2

import (
	"crypto/rand"
	cometbft_bn254 "github.com/cometbft/cometbft/crypto/bn254"
	"github.com/consensys/gnark-crypto/ecc"
	curve "github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark/backend"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/algebra/emulated/fields_bn254"
	gadget "github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	"github.com/consensys/gnark/test"
	"github.com/stretchr/testify/assert"
	"math/big"
	"testing"
)

type Add struct {
	X gadget.G2Affine
	Y gadget.G2Affine
	Z gadget.G2Affine
}

func (c *Add) Define(api frontend.API) error {
	emulated, err := NewEmulatedAPI(api)
	if err != nil {
		return err
	}
	emulated.AssertIsEqual(&c.Z, emulated.AddUnified(&c.X, &c.Y))
	return nil
}

func FuzzAdd(f *testing.F) {
	f.Fuzz(func(t *testing.T, in []byte) {
		t.Parallel()
		x, err := curve.HashToG2(in, []byte{0x01})
		assert.NoError(t, err)
		y, err := curve.HashToG2(in, []byte{0x02})
		assert.NoError(t, err)
		var z curve.G2Affine
		z.Add(&x, &y)
		var circuit Add
		assignment := Add{
			X: gadget.NewG2Affine(x),
			Y: gadget.NewG2Affine(y),
			Z: gadget.NewG2Affine(z),
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
		mapped := curve.MapToCurve2(&messagePoint)
		var circuit MapToCurve
		assignment := MapToCurve{
			Preimage: fields_bn254.FromE2(&messagePoint),
			Image:    gadget.NewG2Affine(mapped),
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

type HashToG2 struct {
	Preimage fields_bn254.E2
	Image    gadget.G2Affine
}

func (c *HashToG2) Define(api frontend.API) error {
	emulated, err := NewEmulatedAPI(api)
	if err != nil {
		return err
	}
	emulated.AssertIsEqual(&c.Image, emulated.HashToG2(&c.Preimage))
	return nil
}

func TestHashToG2(t *testing.T) {
	t.Parallel()
	var message [32]byte
	rand.Read(message[:])
	messageX, messageY := cometbft_bn254.HashToField2(message[:])
	var messagePoint curve.E2
	messagePoint.A0.SetBigInt(messageX.BigInt(new(big.Int)))
	messagePoint.A1.SetBigInt(messageY.BigInt(new(big.Int)))
	mapped := curve.MapToG2(messagePoint)
	var circuit HashToG2
	assignment := HashToG2{
		Preimage: fields_bn254.FromE2(&messagePoint),
		Image:    gadget.NewG2Affine(mapped),
	}
	test.NewAssert(t).ProverSucceeded(
		&circuit,
		&assignment,
		test.WithCurves(ecc.BN254),
		test.NoFuzzing(),
		test.WithCurves(ecc.BN254),
		test.WithBackends(backend.GROTH16),
	)
}
