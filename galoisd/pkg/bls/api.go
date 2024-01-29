package bls

import (
	"fmt"

	curve "github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark/frontend"
	gadget "github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	"github.com/consensys/gnark/std/algebra/emulated/sw_emulated"
	"github.com/consensys/gnark/std/math/emulated"
)

type BlsAPI struct {
	api       frontend.API
	negG1Gen  sw_emulated.AffinePoint[emulated.BN254Fp]
	Nonnative *sw_emulated.Curve[emulated.BN254Fp, emulated.BN254Fr]
}

func NewBlsAPI(api frontend.API) (*BlsAPI, error) {
	nonnative, err := sw_emulated.New[emulated.BN254Fp, emulated.BN254Fr](api, sw_emulated.GetBN254Params())
	if err != nil {
		return nil, err
	}
	_, _, g1Gen, _ := curve.Generators()
	var g1GenNeg curve.G1Affine
	g1GenNeg.Neg(&g1Gen)

	return &BlsAPI{
		api:       api,
		negG1Gen:  gadget.NewG1Affine(g1GenNeg),
		Nonnative: nonnative,
	}, nil
}

// Trivial version (handling infinity points) would be this, at the expense of extra constraints:
// aggregatedPublicKey = curveArithmetic.AddUnified(aggregatedPublicKey, curveArithmetic.Select(signed, publicKey, &G1Zero))
/*
   # we start with G1Gen because we use a partial, cheap `+` that don't handle infinity point
   aggPK = G1Gen
   for ...
  aggPK =
    if selector {
      if firstPK {
        PK
      } else {
        aggPK + PK
      }
    } else {
      aggPK
    }
*/
func (b *BlsAPI) WithAggregation(callback func(aggregateKey func(selector frontend.Variable, publicKey *sw_emulated.AffinePoint[emulated.BN254Fp])) error) (*sw_emulated.AffinePoint[emulated.BN254Fp], frontend.Variable, error) {
	aggregatedPublicKey := b.Nonnative.Generator()
	nbOfKeys := frontend.Variable(0)
	err := callback(func(selector frontend.Variable, publicKey *sw_emulated.AffinePoint[emulated.BN254Fp]) {
		isFirst := b.api.IsZero(nbOfKeys)
		aggregatedPublicKey = b.Nonnative.Select(
			selector,
			b.Nonnative.Select(
				isFirst,
				publicKey,
				b.Nonnative.Add(
					aggregatedPublicKey,
					publicKey,
				),
			),
			aggregatedPublicKey,
		)
		nbOfKeys = b.api.Add(nbOfKeys, selector)
	})
	if err != nil {
		return nil, nil, err
	}
	return aggregatedPublicKey, nbOfKeys, nil
}

// Union whitepaper: (6)
func (b *BlsAPI) VerifySignature(publicKey *gadget.G1Affine, message *gadget.G2Affine, signature *gadget.G2Affine) error {
	pairing, err := gadget.NewPairing(b.api)
	if err != nil {
		return fmt.Errorf("new pairing: %w", err)
	}
	// Technically not needed as constant
	pairing.AssertIsOnG1(&b.negG1Gen)
	pairing.AssertIsOnG1(publicKey)
	// Technically not needed if the hashing to curve happen in-circuit
	pairing.AssertIsOnG2(message)
	pairing.AssertIsOnG2(signature)
	// Verify that the aggregated signature is correct
	err = pairing.PairingCheck(
		[]*gadget.G1Affine{&b.negG1Gen, publicKey},
		[]*gadget.G2Affine{signature, message},
	)
	if err != nil {
		return fmt.Errorf("pairing check: %w", err)
	}
	return nil
}
