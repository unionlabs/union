package g2

import (
	"testing"

	cometbft_bn254 "github.com/cometbft/cometbft/crypto/bn254"
	"github.com/consensys/gnark-crypto/ecc"
	curve "github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/algebra/emulated/fields_bn254"
	gadget "github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	"github.com/consensys/gnark/std/math/emulated"
	"github.com/consensys/gnark/test"
	"github.com/stretchr/testify/assert"

	bn254fr "github.com/consensys/gnark-crypto/ecc/bn254/fr"
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
	f.Fuzz(func(t *testing.T, x []byte, y []byte) {
		t.Parallel()
		var preimage curve.E2
		preimage.A0.SetBytes(x)
		preimage.A1.SetBytes(y)
		err := test.IsSolved(
			&MapToCurve{},
			&MapToCurve{
				Preimage: fields_bn254.FromE2(&preimage),
				Image:    gadget.NewG2Affine(curve.MapToCurve2(&preimage)),
			},
			ecc.BN254.ScalarField(),
		)
		assert.NoError(t, err)
	})
}

type ExpandMsgXmd struct {
	Preimage frontend.Variable
	Domain   frontend.Variable
	Image    [192]frontend.Variable
}

func (c *ExpandMsgXmd) Define(api frontend.API) error {
	emulated, err := NewEmulatedAPI(api)
	if err != nil {
		return err
	}
	image, err := emulated.ExpandMsgXmd(c.Preimage, c.Domain)
	if err != nil {
		return err
	}
	api.AssertIsEqual(192*8, len(image))
	for i := 0; i < 192; i++ {
		imageBits := api.ToBinary(c.Image[i], 8)
		for j := 0; j < 8; j++ {
			api.AssertIsEqual(image[i*8+j], imageBits[j])
		}
	}
	return nil
}

func FuzzExpandMsgXmd(f *testing.F) {
	f.Fuzz(func(t *testing.T, msgIn []byte, dstIn []byte) {
		t.Parallel()
		var msg bn254fr.Element
		msg.SetBytes(msgIn)
		var dst bn254fr.Element
		dst.SetBytes(dstIn)
		var msgBytes [32]byte
		bn254fr.LittleEndian.PutElement(&msgBytes, msg)
		var dstBytes [32]byte
		bn254fr.LittleEndian.PutElement(&dstBytes, dst)
		v, err := cometbft_bn254.ExpandMsgXmdMiMC(msgBytes[:], dstBytes[:], 192)
		assert.NoError(t, err)
		var image [192]frontend.Variable
		for i := 0; i < 192; i++ {
			image[i] = v[i]
		}
		err = test.IsSolved(
			&ExpandMsgXmd{},
			&ExpandMsgXmd{
				Preimage: msg,
				Domain:   dst,
				Image:    image,
			},
			ecc.BN254.ScalarField(),
		)
		assert.NoError(t, err)
	})
}

type HashToFieldC struct {
	Preimage frontend.Variable
	Domain   frontend.Variable
	Image    [4]emulated.Element[emulated.BN254Fp]
}

func (c *HashToFieldC) Define(api frontend.API) error {
	e, err := NewEmulatedAPI(api)
	if err != nil {
		return err
	}
	elements, err := e.HashToField(c.Preimage, c.Domain)
	if err != nil {
		return err
	}
	for i := 0; i < 4; i++ {
		e.field.AssertIsEqual(&c.Image[i], elements[i])
	}
	return nil
}

func FuzzHashMiMC(f *testing.F) {
	f.Fuzz(func(t *testing.T, msgIn []byte, dstIn []byte) {
		t.Parallel()
		var msg bn254fr.Element
		msg.SetBytes(msgIn)
		var dst bn254fr.Element
		dst.SetBytes(dstIn)
		var msgBytes [32]byte
		bn254fr.LittleEndian.PutElement(&msgBytes, msg)
		var dstBytes [32]byte
		bn254fr.LittleEndian.PutElement(&dstBytes, dst)
		v, err := cometbft_bn254.HashToFieldMiMC(msgBytes[:], dstBytes[:])
		assert.NoError(t, err)
		var image [4]emulated.Element[emulated.BN254Fp]
		for i := 0; i < 4; i++ {
			image[i] = emulated.ValueOf[emulated.BN254Fp](v[i])
		}
		err = test.IsSolved(
			&HashToFieldC{},
			&HashToFieldC{
				Preimage: msg,
				Domain:   dst,
				Image:    image,
			},
			ecc.BN254.ScalarField(),
		)
		assert.NoError(t, err)
	})
}

type HashToG2 struct {
	Preimage frontend.Variable
	Domain   frontend.Variable
	Image    gadget.G2Affine
}

func (c *HashToG2) Define(api frontend.API) error {
	emulated, err := NewEmulatedAPI(api)
	if err != nil {
		return err
	}
	image, err := emulated.HashToG2(c.Preimage, c.Domain)
	if err != nil {
		return err
	}
	emulated.AssertIsEqual(image, &c.Image)
	return nil
}

func FuzzHashToG2(f *testing.F) {
	f.Fuzz(func(t *testing.T, msgIn []byte, dstIn []byte) {
		t.Parallel()
		var msg bn254fr.Element
		msg.SetBytes(msgIn)
		var dst bn254fr.Element
		dst.SetBytes(dstIn)
		var msgBytes [32]byte
		bn254fr.LittleEndian.PutElement(&msgBytes, msg)
		var dstBytes [32]byte
		bn254fr.LittleEndian.PutElement(&dstBytes, dst)
		image, err := cometbft_bn254.HashToG2MiMC(msgBytes[:], dstBytes[:])
		assert.NoError(t, err)
		err = test.IsSolved(
			&HashToG2{},
			&HashToG2{
				Preimage: msg,
				Domain:   dst,
				Image:    gadget.NewG2Affine(image),
			},
			ecc.BN254.ScalarField(),
		)
		assert.NoError(t, err)
	})
}
