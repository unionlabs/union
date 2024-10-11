// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

package secp256k1

import (
	"math/bits"

	fiat "gitlab.com/yawning/secp256k1-voi/internal/fiat/secp256k1montgomeryscalar"
	"gitlab.com/yawning/secp256k1-voi/internal/field"
)

// GLV decomposition is first documented in "Faster Point Multiplication
// on Elliptic Curves with Efficient Endomorphisms" by Gallant, Lambert,
// and Vanstone.  This is the infamous endomorphism-based secp256k1
// acceleration.
//
// Given:
// - P(x,y) on the curve
// - P'(beta*x,y) on the curve
//
// There is a scalar lambda where lambda * P = P'.
//
// For an arbitrary scalar k:
// - Decompose into k = k1 + k2 * lambda mod n
// - Calculate k * P = k1 * P + k2 * lambda * P
//                   = k1 * P + k2 * P'
//
// See:
// - https://www.iacr.org/archive/crypto2001/21390189.pdf
// - https://link.springer.com/book/10.1007/b97644
// - https://bitcointalk.org/index.php?topic=3238.0
// - https://homepages.dcc.ufmg.br/~leob/papers/jcen12.pdf

//nolint:godot
var (
	// Constants shamelessly stolen from libsecp256k1's sage script output.

	// -Lambda = 0xac9c52b33fa3cf1f5ad9e3fd77ed9ba4a880b9fc8ec739c2e0cfc810b51283cf
	scNegLambda = newScalarFromCanonicalHex("0xac9c52b33fa3cf1f5ad9e3fd77ed9ba4a880b9fc8ec739c2e0cfc810b51283cf")

	// Beta = 0x7ae96a2b657c07106e64479eac3434e99cf0497512f58995c1396c28719501ee
	feBeta = field.NewElementFromCanonicalHex("0x7ae96a2b657c07106e64479eac3434e99cf0497512f58995c1396c28719501ee")

	// -B1 = 0xe4437ed6010e88286f547fa90abfe4c3
	scNegB1 = newScalarFromCanonicalHex("0xe4437ed6010e88286f547fa90abfe4c3")

	// -B2 = 0xfffffffffffffffffffffffffffffffe8a280ac50774346dd765cda83db1562c
	scNegB2 = newScalarFromCanonicalHex("0xfffffffffffffffffffffffffffffffe8a280ac50774346dd765cda83db1562c")

	// G1 = 0x3086d221a7d46bcde86c90e49284eb153daa8a1471e8ca7fe893209a45dbb031
	scG1 = newScalarFromCanonicalHex("0x3086d221a7d46bcde86c90e49284eb153daa8a1471e8ca7fe893209a45dbb031")

	// G2 = 0xe4437ed6010e88286f547fa90abfe4c4221208ac9df506c61571b4ae8ac47f71
	scG2 = newScalarFromCanonicalHex("0xe4437ed6010e88286f547fa90abfe4c4221208ac9df506c61571b4ae8ac47f71")
)

func (s *Scalar) splitGLV() (*Scalar, *Scalar) {
	// From "Guide to Elliptic Curve Cryptography" by Hankerson,
	// Menezes, Vanstone, Algorithm 3.74 "Balanced length-two
	// representation of a multiplier":
	//
	//   c1 = round(b2 * k / n)
	//   c2 = round(-b1 * k / n)
	//   k1 = k - c1a1 - c2a2
	//   k2 =    -c1b1 - c2b2
	//
	// As libsecp256k1's implementation and comments notes:
	//
	//   k1 = k - k2 * lambda mod n
	//
	// Which saves having to use a1 and a2.

	// As a further optimization, as Per "Efficient software
	// implementation of public-key cryptography on sensor networks
	// using the MSP430X microcontroller" by Gouvêa, Oliveira, and
	// López, there is a way to do more pre-computation, to remove
	// the need for multiple precision division like thus:
	//
	//   d = a1b2 - a2*b1 (== n)
	//   g1 = round(2^384*b2/d)
	//   g2 = round(2^384*(-b1)/d)
	//
	//   c1 = floored_div_pow2(k * g1, 384)
	//   c2 = floored_div_pow2(k * g2, 384)
	//   k2 = -c1b1 - c2b2
	//
	// Where "the floored division by 2^t is a simple right shift
	// of t bits. The last bit discarded in this right shift must
	// be stored, and if it is 1, then b1 = β1 + 1, otherwise b1
	// = β1. The same applies to β2."
	//
	// This is what libsecp256k1 does, and seeing them do it,
	// inspired the decision to do it here.
	//
	// See:
	// - https://link.springer.com/article/10.1007/s13389-012-0029-z
	//   (https://homepages.dcc.ufmg.br/~leob/papers/jcen12.pdf)

	// c1 = floored_div_pow2(k * g1, 384)
	c1 := NewScalar().mulGFlooredDiv(s, scG1)

	// c2 = floored_div_pow2(k * g2, 384)
	c2 := NewScalar().mulGFlooredDiv(s, scG2)

	// k2 = -c1b1 - c2b2
	k2 := NewScalar().Multiply(c1, scNegB1)
	tmp := NewScalar().Multiply(c2, scNegB2)
	k2 = k2.Add(k2, tmp)

	// k1 = k - k2 * lambda mod n
	k1 := NewScalar().Multiply(k2, scNegLambda)
	k1.Add(s, k1)

	return k1, k2
}

func (s *Scalar) mulGFlooredDiv(k, g *Scalar) *Scalar {
	// First calculate k * g, using the schoolbook method.
	//
	// See: "Guide to Elliptic Curve Cryptography" by Hankerson,
	// Menezes, Vanstone, Algorithm 2.9 "Integer multiplication
	// (operand-scanning form)", though the schoolbook method
	// is what is taught in gradeschool, just with limbs.

	var a, b fiat.NonMontgomeryDomainFieldElement
	fiat.FromMontgomery(&a, &k.m)
	fiat.FromMontgomery(&b, &g.m)

	innerProduct := func(c, a, b, u uint64) (uint64, uint64) {
		// (UV) = c[i+j] + a[i] * b[j] + u
		hi, lo := bits.Mul64(a, b)

		var carry uint64
		lo, carry = bits.Add64(lo, c, 0)
		hi += carry

		lo, carry = bits.Add64(lo, u, 0)
		hi += carry

		return hi, lo
	}

	a0, a1, a2, a3 := a[0], a[1], a[2], a[3]
	b0, b1, b2, b3 := b[0], b[1], b[2], b[3]

	var u, c1, c2, c3, c4, c5, c6, c7 uint64

	// i = 0
	u, _ = innerProduct(0 /* c0 */, a0, b0, 0)
	u, c1 = innerProduct(0 /* c1 */, a0, b1, u)
	u, c2 = innerProduct(0 /* c2 */, a0, b2, u)
	c4, c3 = innerProduct(0 /* c3 */, a0, b3, u)

	// i = 1
	u, _ = innerProduct(c1, a1, b0, 0)
	u, c2 = innerProduct(c2, a1, b1, u)
	u, c3 = innerProduct(c3, a1, b2, u)
	c5, c4 = innerProduct(c4, a1, b3, u)

	// i = 2
	u, _ = innerProduct(c2, a2, b0, 0)
	u, c3 = innerProduct(c3, a2, b1, u)
	u, c4 = innerProduct(c4, a2, b2, u)
	c6, c5 = innerProduct(c5, a2, b3, u)

	// i = 3
	u, _ = innerProduct(c3, a3, b0, 0)
	u, _ = innerProduct(c4, a3, b1, u)
	u, c5 = innerProduct(c5, a3, b2, u)
	c7, c6 = innerProduct(c6, a3, b3, u)

	// Then right shift by 384 bits, preserving the last bit
	// discarded by the shift.  Since the shift happens to
	// be an exact multiple of the limb size, there isn't
	// any actual shifting involved, only discarding the
	// 6 least-significant limbs.
	//
	// The unchecked set is fine here since, the check is
	// to see if the scalar is fully reduced, which is always
	// the case when doing a 128-bit set.

	shouldAdd := (c5 >> 63) & 1
	c6, u = bits.Add64(c6, shouldAdd, 0)
	c7 += u

	return s.uncheckedSetSaturated(&[4]uint64{c6, c7, 0, 0})
}

func (v *Point) mulBeta(p *Point) *Point {
	assertPointsValid(p)

	v.x.Multiply(&p.x, feBeta)
	v.y.Set(&p.y)
	v.z.Set(&p.z)
	v.isValid = p.isValid

	return v
}

// scalarMultVartimeGLV sets `v = s * p`, and returns `v` in variable time.
func (v *Point) scalarMultVartimeGLV(s *Scalar, p *Point) *Point {
	pee := NewPointFrom(p) // Note: Checks p is valid.
	peePrime := newMulBeta(p)

	// Split the scalar.
	//
	// Pick the shorter reprentation for each of the returned scalars
	// by negating both the scalar and it's corresponding point if
	// required.
	k1, k2 := s.splitGLV()
	if k1.IsGreaterThanHalfN() == 1 {
		k1.Negate(k1)
		pee.Negate(pee)
	}
	if k2.IsGreaterThanHalfN() == 1 {
		k2.Negate(k2)
		peePrime.Negate(peePrime)
	}

	pTbl := newProjectivePointMultTable(pee)
	pPrimeTbl := newProjectivePointMultTable(peePrime)

	v.Identity()

	const off = 16
	k1Bytes, k2Bytes := k1.Bytes(), k2.Bytes()
	k1Bytes, k2Bytes = k1Bytes[off:], k2Bytes[off:]

	for i := 0; i < ScalarSize-off; i++ {
		if i != 0 {
			v.doubleComplete(v)
			v.doubleComplete(v)
			v.doubleComplete(v)
			v.doubleComplete(v)
		}

		bK1, bK2 := k1Bytes[i], k2Bytes[i]

		pTbl.SelectAndAddVartime(v, uint64(bK1>>4))
		pPrimeTbl.SelectAndAddVartime(v, uint64(bK2>>4))

		v.doubleComplete(v)
		v.doubleComplete(v)
		v.doubleComplete(v)
		v.doubleComplete(v)

		pTbl.SelectAndAddVartime(v, uint64(bK1&0xf))
		pPrimeTbl.SelectAndAddVartime(v, uint64(bK2&0xf))
	}

	return v
}

// ScalarMult sets `v = s * p`, and returns `v`.
func (v *Point) ScalarMult(s *Scalar, p *Point) *Point {
	pee := NewPointFrom(p) // Note: Checks p is valid.
	peePrime := newMulBeta(p)

	k1, k2 := s.splitGLV()

	negateK1 := k1.IsGreaterThanHalfN()
	k1.ConditionalNegate(k1, negateK1)
	pee.ConditionalNegate(pee, negateK1)

	negateK2 := k2.IsGreaterThanHalfN()
	k2.ConditionalNegate(k2, negateK2)
	peePrime.ConditionalNegate(peePrime, negateK2)

	pTbl := newProjectivePointMultTable(pee)
	pPrimeTbl := newProjectivePointMultTable(peePrime)

	v.Identity()

	const off = 16
	k1Bytes, k2Bytes := k1.Bytes(), k2.Bytes()
	k1Bytes, k2Bytes = k1Bytes[off:], k2Bytes[off:]

	for i := 0; i < ScalarSize-off; i++ {
		if i != 0 {
			v.doubleComplete(v)
			v.doubleComplete(v)
			v.doubleComplete(v)
			v.doubleComplete(v)
		}

		bK1, bK2 := k1Bytes[i], k2Bytes[i]

		pTbl.SelectAndAdd(v, uint64(bK1>>4))
		pPrimeTbl.SelectAndAdd(v, uint64(bK2>>4))

		v.doubleComplete(v)
		v.doubleComplete(v)
		v.doubleComplete(v)
		v.doubleComplete(v)

		pTbl.SelectAndAdd(v, uint64(bK1&0xf))
		pPrimeTbl.SelectAndAdd(v, uint64(bK2&0xf))
	}

	return v
}

// DoubleScalarMultBasepointVartime sets `v = u1 * G + u2 * P`, and returns
// `v` in variable time, where `G` is the generator.
func (v *Point) DoubleScalarMultBasepointVartime(u1, u2 *Scalar, p *Point) *Point {
	// To the best of my knowledge, doing things this way is faster than
	// Straus-Shamir, given our scalar-basepoint multiply implementation,
	// especially if the variable-base multiply is well optimized.
	//
	// This routine is the most performance critical as it is the core
	// of ECDSA verification.
	u1g := newRcvr().scalarBaseMultVartime(u1)
	u2p := newRcvr().scalarMultVartimeGLV(u2, p)
	return v.Add(u1g, u2p)
}

func newMulBeta(p *Point) *Point {
	return newRcvr().mulBeta(p)
}
