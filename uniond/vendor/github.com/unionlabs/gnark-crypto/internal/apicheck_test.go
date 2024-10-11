package main

import (
	bls377 "github.com/consensys/gnark-crypto/ecc/bls12-377"
	bls378 "github.com/consensys/gnark-crypto/ecc/bls12-378"
	bls381 "github.com/consensys/gnark-crypto/ecc/bls12-381"
	"github.com/consensys/gnark-crypto/ecc/bn254"
	bw761 "github.com/consensys/gnark-crypto/ecc/bw6-761"
)

// note: pairing API is not code generated, and don't use interfaces{} for performance reasons
// we end up having some API disparities -- this section ensures that we don't forget to update some APIs

var err error

var (
	gtbls377 bls377.GT
	gtbls378 bls378.GT
	gtbls381 bls381.GT
	gtbn254  bn254.GT
	gtbw761  bw761.GT
)

func init() {
	// Pair
	gtbls377, err = bls377.Pair([]bls377.G1Affine{}, []bls377.G2Affine{})
	gtbls378, err = bls378.Pair([]bls378.G1Affine{}, []bls378.G2Affine{})
	gtbls381, err = bls381.Pair([]bls381.G1Affine{}, []bls381.G2Affine{})
	gtbn254, err = bn254.Pair([]bn254.G1Affine{}, []bn254.G2Affine{})
	gtbw761, err = bw761.Pair([]bw761.G1Affine{}, []bw761.G2Affine{})

	// MillerLoop
	gtbls377, err = bls377.MillerLoop([]bls377.G1Affine{}, []bls377.G2Affine{})
	gtbls378, err = bls378.MillerLoop([]bls378.G1Affine{}, []bls378.G2Affine{})
	gtbls381, err = bls381.MillerLoop([]bls381.G1Affine{}, []bls381.G2Affine{})
	gtbn254, err = bn254.MillerLoop([]bn254.G1Affine{}, []bn254.G2Affine{})
	gtbw761, err = bw761.MillerLoop([]bw761.G1Affine{}, []bw761.G2Affine{})

	// FinalExp
	gtbls377 = bls377.FinalExponentiation(&gtbls377)
	gtbls378 = bls378.FinalExponentiation(&gtbls378)
	gtbls381 = bls381.FinalExponentiation(&gtbls381)
	gtbn254 = bn254.FinalExponentiation(&gtbn254)
	gtbw761 = bw761.FinalExponentiation(&gtbw761)
}
