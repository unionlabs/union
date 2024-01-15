package bls

import (
	"crypto/rand"
	"fmt"
	"math/big"
	"testing"

	cometbn254 "github.com/cometbft/cometbft/crypto/bn254"
	"github.com/consensys/gnark-crypto/ecc"
	curve "github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark/frontend"
	gadget "github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	"github.com/consensys/gnark/std/algebra/emulated/sw_emulated"
	"github.com/consensys/gnark/std/math/emulated"
	"github.com/consensys/gnark/test"
	"github.com/stretchr/testify/assert"
)

const MaxKeys = 128

type BlsAgg struct {
	PublicKeys          [MaxKeys]gadget.G1Affine
	Bitmap              [MaxKeys]frontend.Variable
	AggregatedPublicKey gadget.G1Affine
}

func (c *BlsAgg) Define(api frontend.API) error {
	bls, err := NewBlsAPI(api)
	if err != nil {
		return err
	}
	aggregatedPublicKey, _, err :=
		bls.WithAggregation(func(aggregateKey func(selector frontend.Variable, publicKey *sw_emulated.AffinePoint[emulated.BN254Fp])) error {
			for i := 0; i < MaxKeys; i++ {
				aggregateKey(c.Bitmap[i], &c.PublicKeys[i])
			}
			return nil
		})
	bls.Nonnative.AssertIsEqual(aggregatedPublicKey, &c.AggregatedPublicKey)
	return nil
}

func TestBlsAdd(t *testing.T) {
	t.Parallel()
	for i := 1; i < MaxKeys; i++ {
		k := i
		t.Run(fmt.Sprintf("%d", k), func(t *testing.T) {
			t.Parallel()
			pks := [MaxKeys]gadget.G1Affine{}
			bitmap := [MaxKeys]frontend.Variable{}
			var expectedAggPK curve.G1Affine
			for j := 0; j < k; j++ {
				r, err := rand.Int(rand.Reader, big.NewInt(2))
				assert.NoError(t, err)
				if j == 0 {
					// At least one PK to agg
					r.SetInt64(1)
				}
				var pk curve.G1Affine
				_, err = pk.SetBytes(cometbn254.GenPrivKey().PubKey().Bytes())
				assert.NoError(t, err)
				pks[j] = gadget.NewG1Affine(pk)
				bitmap[j] = r
				if r.Int64() == 1 {
					expectedAggPK.Add(&expectedAggPK, &pk)
				}
			}
			for j := k; j < MaxKeys; j++ {
				pks[j] = gadget.NewG1Affine(curve.G1Affine{})
				bitmap[j] = 0
			}
			err := test.IsSolved(
				&BlsAgg{},
				&BlsAgg{
					PublicKeys:          pks,
					Bitmap:              bitmap,
					AggregatedPublicKey: gadget.NewG1Affine(expectedAggPK),
				},
				ecc.BN254.ScalarField(),
			)
			assert.NoError(t, err)
		})
	}
}

type BlsSig struct {
	PublicKey gadget.G1Affine
	Signature gadget.G2Affine
	Message   gadget.G2Affine
}

func (c *BlsSig) Define(api frontend.API) error {
	bls, err := NewBlsAPI(api)
	if err != nil {
		return err
	}
	err = bls.VerifySignature(&c.PublicKey, &c.Message, &c.Signature)
	if err != nil {
		return err
	}
	return nil
}

func TestBlsSig(t *testing.T) {
	t.Parallel()
	message := make([]byte, 256)
	_, err := rand.Read(message)
	sk := cometbn254.GenPrivKey()
	rawSig, err := sk.Sign(message)
	assert.NoError(t, err)
	hashed := cometbn254.HashToG2(message)
	var pk curve.G1Affine
	_, err = pk.SetBytes(sk.PubKey().Bytes())
	assert.NoError(t, err)
	var sig curve.G2Affine
	_, err = sig.SetBytes(rawSig)
	assert.NoError(t, err)
	err = test.IsSolved(
		&BlsSig{},
		&BlsSig{
			PublicKey: gadget.NewG1Affine(pk),
			Signature: gadget.NewG2Affine(sig),
			Message:   gadget.NewG2Affine(hashed),
		},
		ecc.BN254.ScalarField(),
	)
	assert.NoError(t, err)
}
