// Copyright 2010 The Go Authors. All rights reserved.
// Copyright 2011 ThePiachu. All rights reserved.
// Copyright 2013-2014 The btcsuite developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package btcec

// References:
//   [SECG]: Recommended Elliptic Curve Domain Parameters
//     http://www.secg.org/sec2-v2.pdf
//
//   [GECC]: Guide to Elliptic Curve Cryptography (Hankerson, Menezes, Vanstone)

// This package operates, internally, on Jacobian coordinates. For a given
// (x, y) position on the curve, the Jacobian coordinates are (x1, y1, z1)
// where x = x1/z1² and y = y1/z1³. The greatest speedups come when the whole
// calculation can be performed within the transform (as in ScalarMult and
// ScalarBaseMult). But even for Add and Double, it's faster to apply and
// reverse the transform than to operate in affine coordinates.

import (
	secp "github.com/decred/dcrd/dcrec/secp256k1/v4"
)

// KoblitzCurve provides an implementation for secp256k1 that fits the ECC
// Curve interface from crypto/elliptic.
type KoblitzCurve = secp.KoblitzCurve

// S256 returns a Curve which implements secp256k1.
func S256() *KoblitzCurve {
	return secp.S256()
}

// CurveParams contains the parameters for the secp256k1 curve.
type CurveParams = secp.CurveParams

// Params returns the secp256k1 curve parameters for convenience.
func Params() *CurveParams {
	return secp.Params()
}

// Generator returns the public key at the Generator Point.
func Generator() *PublicKey {
	var (
		result JacobianPoint
		k      secp.ModNScalar
	)

	k.SetInt(1)
	ScalarBaseMultNonConst(&k, &result)

	result.ToAffine()

	return NewPublicKey(&result.X, &result.Y)
}
