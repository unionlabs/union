// Package emparams contains emulation parameters for well known fields.
//
// We define some well-known parameters in this package for compatibility and
// ease of use. When needing to use parameters not defined in this package it is
// sufficient to define a new type implementing [FieldParams]. For example, as:
//
//	type SmallField struct {}
//	func (SmallField) NbLimbs() uint { return 1 }
//	func (SmallField) BitsPerLimb() uint { return 11 }
//	func (SmallField) IsPrime() bool { return true }
//	func (SmallField) Modulus() *big.Int { return big.NewInt(1032) }
package emparams

import (
	"crypto/elliptic"
	"math/big"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark-crypto/field/goldilocks"
)

type fourLimbPrimeField struct{}

func (fourLimbPrimeField) NbLimbs() uint     { return 4 }
func (fourLimbPrimeField) BitsPerLimb() uint { return 64 }
func (fourLimbPrimeField) IsPrime() bool     { return true }

type fiveLimbPrimeField struct{}

func (fiveLimbPrimeField) NbLimbs() uint     { return 5 }
func (fiveLimbPrimeField) BitsPerLimb() uint { return 64 }
func (fiveLimbPrimeField) IsPrime() bool     { return true }

type sixLimbPrimeField struct{}

func (sixLimbPrimeField) NbLimbs() uint     { return 6 }
func (sixLimbPrimeField) BitsPerLimb() uint { return 64 }
func (sixLimbPrimeField) IsPrime() bool     { return true }

type twelveLimbPrimeField struct{}

func (twelveLimbPrimeField) NbLimbs() uint     { return 12 }
func (twelveLimbPrimeField) BitsPerLimb() uint { return 64 }
func (twelveLimbPrimeField) IsPrime() bool     { return true }

// Goldilocks provides type parametrization for field emulation:
//   - limbs: 1
//   - limb width: 64 bits
//
// The prime modulus for type parametrisation is:
//
//	0xffffffff00000001 (base 16)
//	18446744069414584321 (base 10)
type Goldilocks struct{}

func (fp Goldilocks) NbLimbs() uint     { return 1 }
func (fp Goldilocks) BitsPerLimb() uint { return 64 }
func (fp Goldilocks) IsPrime() bool     { return true }
func (fp Goldilocks) Modulus() *big.Int { return goldilocks.Modulus() }

// Secp256k1Fp provides type parametrization for field emulation:
//   - limbs: 4
//   - limb width: 64 bits
//
// The prime modulus for type parametrisation is:
//
//	0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f (base 16)
//	115792089237316195423570985008687907853269984665640564039457584007908834671663 (base 10)
//
// This is the base field of the SECP256k1 curve.
type Secp256k1Fp struct{ fourLimbPrimeField }

func (fp Secp256k1Fp) Modulus() *big.Int { return ecc.SECP256K1.BaseField() }

// Secp256k1Fr provides type parametrization for field emulation:
//   - limbs: 4
//   - limb width: 64 bits
//
// The prime modulus for type parametrisation is:
//
//	0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141 (base 16)
//	115792089237316195423570985008687907852837564279074904382605163141518161494337 (base 10)
//
// This is the scalar field of the SECP256k1 curve.
type Secp256k1Fr struct{ fourLimbPrimeField }

func (fp Secp256k1Fr) Modulus() *big.Int { return ecc.SECP256K1.ScalarField() }

// BN254Fp provides type parametrization for field emulation:
//   - limbs: 4
//   - limb width: 64 bits
//
// The prime modulus for type parametrisation is:
//
//	0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47 (base 16)
//	21888242871839275222246405745257275088696311157297823662689037894645226208583 (base 10)
//
// This is the base field of the BN254 curve.
type BN254Fp struct{ fourLimbPrimeField }

func (fp BN254Fp) Modulus() *big.Int { return ecc.BN254.BaseField() }

// BN254Fr provides type parametrization for field emulation:
//   - limbs: 4
//   - limb width: 64 bits
//
// The prime modulus for type parametrisation is:
//
//	0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001 (base 16)
//	21888242871839275222246405745257275088548364400416034343698204186575808495617 (base 10)
//
// This is the scalar field of the BN254 curve.
type BN254Fr struct{ fourLimbPrimeField }

func (fp BN254Fr) Modulus() *big.Int { return ecc.BN254.ScalarField() }

// BLS12377Fp provides type parametrization for field emulation:
//   - limbs: 6
//   - limb width: 64 bits
//
// The prime modulus for type parametrisation is:
//
//	0x1ae3a4617c510eac63b05c06ca1493b1a22d9f300f5138f1ef3622fba094800170b5d44300000008508c00000000001 (base 16)
//	258664426012969094010652733694893533536393512754914660539884262666720468348340822774968888139573360124440321458177 (base 10)
//
// This is the base field of the BLS12-377 curve.
type BLS12377Fp struct{ sixLimbPrimeField }

func (fp BLS12377Fp) Modulus() *big.Int { return ecc.BLS12_377.BaseField() }

// BLS12377Fr provides type parametrization for field emulation:
//   - limbs: 4
//   - limb width: 64 bits
//
// The prime modulus for type parametrisation is:
//
//	0x12ab655e9a2ca55660b44d1e5c37b00159aa76fed00000010a11800000000001 (base 16)
//	8444461749428370424248824938781546531375899335154063827935233455917409239041 (base 10)
//
// This is the scalar field of the BLS12-377 curve.
type BLS12377Fr struct{ fourLimbPrimeField }

func (fr BLS12377Fr) Modulus() *big.Int { return ecc.BLS12_377.ScalarField() }

// BLS12381Fp provides type parametrization for field emulation:
//   - limbs: 6
//   - limb width: 64 bits
//
// The prime modulus for type parametrisation is:
//
//	0x1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab (base 16)
//	4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559787 (base 10)
//
// This is the base field of the BLS12-381 curve.
type BLS12381Fp struct{ sixLimbPrimeField }

func (fp BLS12381Fp) Modulus() *big.Int { return ecc.BLS12_381.BaseField() }

// BLS12381Fr provides type parametrization for field emulation:
//   - limbs: 4
//   - limb width: 64 bits
//
// The prime modulus for type parametrisation is:
//
//	0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001 (base 16)
//	52435875175126190479447740508185965837690552500527637822603658699938581184513 (base 10)
//
// This is the scalar field of the BLS12-381 curve.
type BLS12381Fr struct{ fourLimbPrimeField }

func (fp BLS12381Fr) Modulus() *big.Int { return ecc.BLS12_381.ScalarField() }

// P256Fp provides type parametrization for field emulation:
//   - limbs: 4
//   - limb width: 64 bits
//
// The prime modulus for type parametrisation is:
//
//	0xffffffff00000001000000000000000000000000ffffffffffffffffffffffff (base 16)
//	115792089210356248762697446949407573530086143415290314195533631308867097853951 (base 10)
//
// This is the base field of the P-256 (also SECP256r1) curve.
type P256Fp struct{ fourLimbPrimeField }

func (P256Fp) Modulus() *big.Int { return elliptic.P256().Params().P }

// P256Fr provides type parametrization for field emulation:
//   - limbs: 4
//   - limb width: 64 bits
//
// The prime modulus for type parametrisation is:
//
//	0xffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551 (base 16)
//	115792089210356248762697446949407573529996955224135760342422259061068512044369 (base 10)
//
// This is the base field of the P-256 (also SECP256r1) curve.
type P256Fr struct{ fourLimbPrimeField }

func (P256Fr) Modulus() *big.Int { return elliptic.P256().Params().N }

// P384Fp provides type parametrization for field emulation:
//   - limbs: 6
//   - limb width: 64 bits
//
// The prime modulus for type parametrisation is:
//
//	0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffeffffffff0000000000000000ffffffff (base 16)
//	39402006196394479212279040100143613805079739270465446667948293404245721771496870329047266088258938001861606973112319 (base 10)
//
// This is the base field of the P-384 (also SECP384r1) curve.
type P384Fp struct{ sixLimbPrimeField }

func (P384Fp) Modulus() *big.Int { return elliptic.P384().Params().P }

// P384Fr provides type parametrization for field emulation:
//   - limbs: 6
//   - limb width: 64 bits
//
// The prime modulus for type parametrisation is:
//
//	0xffffffffffffffffffffffffffffffffffffffffffffffffc7634d81f4372ddf581a0db248b0a77aecec196accc52973 (base 16)
//	39402006196394479212279040100143613805079739270465446667946905279627659399113263569398956308152294913554433653942643 (base 10)
//
// This is the scalar field of the P-384 (also SECP384r1) curve.
type P384Fr struct{ sixLimbPrimeField }

func (P384Fr) Modulus() *big.Int { return elliptic.P384().Params().N }

// BW6761Fp provides type parametrization for field emulation:
//   - limbs: 12
//   - limb width: 64 bits
//
// The prime modulus for type parametrisation is:
//
//	0x122e824fb83ce0ad187c94004faff3eb926186a81d14688528275ef8087be41707ba638e584e91903cebaff25b423048689c8ed12f9fd9071dcd3dc73ebff2e98a116c25667a8f8160cf8aeeaf0a437e6913e6870000082f49d00000000008b (base 16)
//	6891450384315732539396789682275657542479668912536150109513790160209623422243491736087683183289411687640864567753786613451161759120554247759349511699125301598951605099378508850372543631423596795951899700429969112842764913119068299 (base 10)
//
// This is the base field of the BW6-761 curve.
type BW6761Fp struct{ twelveLimbPrimeField }

func (fp BW6761Fp) Modulus() *big.Int { return ecc.BW6_761.BaseField() }

// BW6761Fr provides type parametrization for field emulation:
//   - limbs: 6
//   - limb width: 64 bits
//
// The prime modulus for type parametrisation is:
//
//	0x1ae3a4617c510eac63b05c06ca1493b1a22d9f300f5138f1ef3622fba094800170b5d44300000008508c00000000001 (base 16)
//	258664426012969094010652733694893533536393512754914660539884262666720468348340822774968888139573360124440321458177 (base 10)
//
// This is the scalar field of the BW6-761 curve.
type BW6761Fr struct{ sixLimbPrimeField }

func (fp BW6761Fr) Modulus() *big.Int { return ecc.BW6_761.ScalarField() }

// BLS12315Fp provides type parametrization for field emulation:
//   - limbs: 5
//   - limb width: 64 bits
//
// The prime modulus for type parametrisation is:
//
//	0x4c23a02b586d650d3f7498be97c5eafdec1d01aa27a1ae0421ee5da52bde5026fe802ff40300001 (base 16)
//	39705142709513438335025689890408969744933502416914749335064285505637884093126342347073617133569 (base 10)
//
// This is the base field of the BLS24-315 curve.
type BLS12315Fp struct{ fiveLimbPrimeField }

func (fp BLS12315Fp) Modulus() *big.Int { return ecc.BLS24_315.BaseField() }

// BLS12315Fr provides type parametrization for field emulation:
//   - limbs: 4
//   - limb width: 64 bits
//
// The prime modulus for type parametrisation is:
//
//	11502027791375260645628074404575422495959608200132055716665986169834464870401 (base 16)
//	0x196deac24a9da12b25fc7ec9cf927a98c8c480ece644e36419d0c5fd00c00001 (base 10)
//
// This is the scalar field of the BLS24-315 curve.
type BLS12315Fr struct{ fourLimbPrimeField }

func (fr BLS12315Fr) Modulus() *big.Int { return ecc.BLS24_315.ScalarField() }
