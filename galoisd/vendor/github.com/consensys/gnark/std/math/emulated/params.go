package emulated

import (
	"math/big"

	"github.com/consensys/gnark/std/math/emulated/emparams"
)

// FieldParams describes the emulated field characteristics. For a list of
// included built-in emulation params refer to the [emparams] package.
// For backwards compatibility, the current package contains the following
// parameters:
//   - [Goldilocks]
//   - [Secp256k1Fp] and [Secp256k1Fr]
//   - [BN254Fp] and [BN254Fr]
//   - [BLS12377Fp]
//   - [BLS12381Fp] and [BLS12381Fr]
//   - [P256Fp] and [P256Fr]
//   - [P384Fp] and [P384Fr]
type FieldParams interface {
	NbLimbs() uint     // number of limbs to represent field element
	BitsPerLimb() uint // number of bits per limb. Top limb may contain less than limbSize bits.
	IsPrime() bool     // indicates if the modulus is prime
	Modulus() *big.Int // returns modulus. Do not modify.
}

type (
	Goldilocks  = emparams.Goldilocks
	Secp256k1Fp = emparams.Secp256k1Fp
	Secp256k1Fr = emparams.Secp256k1Fr
	BN254Fp     = emparams.BN254Fp
	BN254Fr     = emparams.BN254Fr
	BLS12377Fp  = emparams.BLS12377Fp
	BLS12381Fp  = emparams.BLS12381Fp
	BLS12381Fr  = emparams.BLS12381Fr
	P256Fp      = emparams.P256Fp
	P256Fr      = emparams.P256Fr
	P384Fp      = emparams.P384Fp
	P384Fr      = emparams.P384Fr
	BW6761Fp    = emparams.BW6761Fp
	BW6761Fr    = emparams.BW6761Fr
)
