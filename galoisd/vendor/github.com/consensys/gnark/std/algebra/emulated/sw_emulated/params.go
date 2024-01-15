package sw_emulated

import (
	"crypto/elliptic"
	"math/big"

	bls12381 "github.com/consensys/gnark-crypto/ecc/bls12-381"
	"github.com/consensys/gnark-crypto/ecc/bn254"
	bw6761 "github.com/consensys/gnark-crypto/ecc/bw6-761"
	"github.com/consensys/gnark-crypto/ecc/secp256k1"
	"github.com/consensys/gnark/std/math/emulated"
)

// CurveParams defines parameters of an elliptic curve in short Weierstrass form
// given by the equation
//
//	Y² = X³ + aX + b
//
// The base point is defined by (Gx, Gy).
type CurveParams struct {
	A            *big.Int      // a in curve equation
	B            *big.Int      // b in curve equation
	Gx           *big.Int      // base point x
	Gy           *big.Int      // base point y
	Gm           [][2]*big.Int // m*base point coords
	Eigenvalue   *big.Int      // endomorphism eigenvalue
	ThirdRootOne *big.Int      // endomorphism image scaler
}

// GetSecp256k1Params returns curve parameters for the curve secp256k1. When
// initialising new curve, use the base field [emulated.Secp256k1Fp] and scalar
// field [emulated.Secp256k1Fr].
func GetSecp256k1Params() CurveParams {
	_, g1aff := secp256k1.Generators()
	lambda, _ := new(big.Int).SetString("37718080363155996902926221483475020450927657555482586988616620542887997980018", 10)
	omega, _ := new(big.Int).SetString("55594575648329892869085402983802832744385952214688224221778511981742606582254", 10)
	return CurveParams{
		A:            big.NewInt(0),
		B:            big.NewInt(7),
		Gx:           g1aff.X.BigInt(new(big.Int)),
		Gy:           g1aff.Y.BigInt(new(big.Int)),
		Gm:           computeSecp256k1Table(),
		Eigenvalue:   lambda,
		ThirdRootOne: omega,
	}
}

// GetBN254Params returns the curve parameters for the curve BN254 (alt_bn128).
// When initialising new curve, use the base field [emulated.BN254Fp] and scalar
// field [emulated.BN254Fr].
func GetBN254Params() CurveParams {
	_, _, g1aff, _ := bn254.Generators()
	lambda, _ := new(big.Int).SetString("4407920970296243842393367215006156084916469457145843978461", 10)
	omega, _ := new(big.Int).SetString("2203960485148121921418603742825762020974279258880205651966", 10)
	return CurveParams{
		A:            big.NewInt(0),
		B:            big.NewInt(3),
		Gx:           g1aff.X.BigInt(new(big.Int)),
		Gy:           g1aff.Y.BigInt(new(big.Int)),
		Gm:           computeBN254Table(),
		Eigenvalue:   lambda,
		ThirdRootOne: omega,
	}
}

// GetBLS12381Params returns the curve parameters for the curve BLS12-381.
// When initialising new curve, use the base field [emulated.BLS12381Fp] and scalar
// field [emulated.BLS12381Fr].
func GetBLS12381Params() CurveParams {
	_, _, g1aff, _ := bls12381.Generators()
	lambda, _ := new(big.Int).SetString("228988810152649578064853576960394133503", 10)
	omega, _ := new(big.Int).SetString("4002409555221667392624310435006688643935503118305586438271171395842971157480381377015405980053539358417135540939436", 10)
	return CurveParams{
		A:            big.NewInt(0),
		B:            big.NewInt(4),
		Gx:           g1aff.X.BigInt(new(big.Int)),
		Gy:           g1aff.Y.BigInt(new(big.Int)),
		Gm:           computeBLS12381Table(),
		Eigenvalue:   lambda,
		ThirdRootOne: omega,
	}
}

// GetP256Params returns the curve parameters for the curve P-256 (also
// SECP256r1). When initialising new curve, use the base field
// [emulated.P256Fp] and scalar field [emulated.P256Fr].
func GetP256Params() CurveParams {
	pr := elliptic.P256().Params()
	a := new(big.Int).Sub(pr.P, big.NewInt(3))
	return CurveParams{
		A:            a,
		B:            pr.B,
		Gx:           pr.Gx,
		Gy:           pr.Gy,
		Gm:           computeP256Table(),
		Eigenvalue:   nil,
		ThirdRootOne: nil,
	}
}

// GetP384Params returns the curve parameters for the curve P-384 (also
// SECP384r1). When initialising new curve, use the base field
// [emulated.P384Fp] and scalar field [emulated.P384Fr].
func GetP384Params() CurveParams {
	pr := elliptic.P384().Params()
	a := new(big.Int).Sub(pr.P, big.NewInt(3))
	return CurveParams{
		A:            a,
		B:            pr.B,
		Gx:           pr.Gx,
		Gy:           pr.Gy,
		Gm:           computeP384Table(),
		Eigenvalue:   nil,
		ThirdRootOne: nil,
	}
}

// GetBW6761Params returns the curve parameters for the curve BW6-761.
// When initialising new curve, use the base field [emulated.BW6761Fp] and scalar
// field [emulated.BW6761Fr].
func GetBW6761Params() CurveParams {
	_, _, g1aff, _ := bw6761.Generators()
	lambda, _ := new(big.Int).SetString("80949648264912719408558363140637477264845294720710499478137287262712535938301461879813459410945", 10)
	omega, _ := new(big.Int).SetString("1968985824090209297278610739700577151397666382303825728450741611566800370218827257750865013421937292370006175842381275743914023380727582819905021229583192207421122272650305267822868639090213645505120388400344940985710520836292650", 10)
	return CurveParams{
		A:            big.NewInt(0),
		B:            big.NewInt(-1),
		Gx:           g1aff.X.BigInt(new(big.Int)),
		Gy:           g1aff.Y.BigInt(new(big.Int)),
		Gm:           computeBW6761Table(),
		Eigenvalue:   lambda,
		ThirdRootOne: omega,
	}
}

// GetCurveParams returns suitable curve parameters given the parametric type
// Base as base field. It caches the parameters and modifying the values in the
// parameters struct leads to undefined behaviour.
func GetCurveParams[Base emulated.FieldParams]() CurveParams {
	var t Base
	switch t.Modulus().String() {
	case emulated.Secp256k1Fp{}.Modulus().String():
		return secp256k1Params
	case emulated.BN254Fp{}.Modulus().String():
		return bn254Params
	case emulated.BLS12381Fp{}.Modulus().String():
		return bls12381Params
	case emulated.P256Fp{}.Modulus().String():
		return p256Params
	case emulated.P384Fp{}.Modulus().String():
		return p384Params
	case emulated.BW6761Fp{}.Modulus().String():
		return bw6761Params
	default:
		panic("no stored parameters")
	}
}

var (
	secp256k1Params CurveParams
	bn254Params     CurveParams
	bls12381Params  CurveParams
	p256Params      CurveParams
	p384Params      CurveParams
	bw6761Params    CurveParams
)

func init() {
	secp256k1Params = GetSecp256k1Params()
	bn254Params = GetBN254Params()
	bls12381Params = GetBLS12381Params()
	p256Params = GetP256Params()
	p384Params = GetP384Params()
	bw6761Params = GetBW6761Params()
}
