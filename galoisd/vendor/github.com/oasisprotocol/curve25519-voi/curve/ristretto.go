// Copyright (c) 2016-2019 isis agora lovecruft. All rights reserved.
// Copyright (c) 2016-2019 Henry de Valence. All rights reserved.
// Copyright (c) 2020-2021 Oasis Labs Inc. All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
// 1. Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright
// notice, this list of conditions and the following disclaimer in the
// documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS
// IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED
// TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A
// PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED
// TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

package curve

import (
	"crypto/rand"
	"fmt"
	"io"

	"github.com/oasisprotocol/curve25519-voi/curve/scalar"
	"github.com/oasisprotocol/curve25519-voi/internal/field"
	"github.com/oasisprotocol/curve25519-voi/internal/subtle"
)

// CompressedRistretto represents a Ristretto point in wire format.
type CompressedRistretto [CompressedPointSize]byte

// MarshalBinary encodes the compressed Ristretto point into a binary form
// and returns the result.
func (p *CompressedRistretto) MarshalBinary() ([]byte, error) {
	b := make([]byte, CompressedPointSize)
	copy(b, p[:])
	return b, nil
}

// UnmarshalBinary decodes a binary serialized compressed Ristretto point.
func (p *CompressedRistretto) UnmarshalBinary(data []byte) error {
	p.Identity() // Foot + gun avoidance.

	var rp RistrettoPoint
	if err := rp.UnmarshalBinary(data); err != nil {
		return err
	}

	_, _ = p.SetBytes(data) // Can not fail.

	return nil
}

// SetBytes constructs a compressed Ristretto point from a byte representation.
func (p *CompressedRistretto) SetBytes(in []byte) (*CompressedRistretto, error) {
	if len(in) != CompressedPointSize {
		return nil, fmt.Errorf("curve/ristretto: unexpected input size")
	}

	copy(p[:], in)

	return p, nil
}

// SetRistrettoPoint compresses a Ristretto point into a CompressedRistretto.
func (p *CompressedRistretto) SetRistrettoPoint(ristrettoPoint *RistrettoPoint) *CompressedRistretto {
	ip := &ristrettoPoint.inner // Make this look less ugly.
	X := ip.inner.X
	Y := ip.inner.Y
	Z := ip.inner.Z
	T := ip.inner.T

	var u1, u2, tmp field.Element
	u1.Add(&Z, &Y)
	tmp.Sub(&Z, &Y)
	u1.Mul(&u1, &tmp)
	u2.Mul(&X, &Y)

	// Ignore return value since this is always square.
	var invsqrt field.Element
	invsqrt.Square(&u2)
	invsqrt.Mul(&u1, &invsqrt)
	_, _ = invsqrt.InvSqrt()
	var i1, i2, zInv, denInv field.Element
	i1.Mul(&invsqrt, &u1)
	i2.Mul(&invsqrt, &u2)
	zInv.Mul(&i2, &T)
	zInv.Mul(&i1, &zInv)
	denInv.Set(&i2)

	var iX, iY, enchantedDenominator field.Element
	iX.Mul(&X, &field.SQRT_M1)
	iY.Mul(&Y, &field.SQRT_M1)
	enchantedDenominator.Mul(&i1, &constINVSQRT_A_MINUS_D)

	tmp.Mul(&T, &zInv)
	rotate := tmp.IsNegative()

	X.ConditionalAssign(&iY, rotate)
	Y.ConditionalAssign(&iX, rotate)
	denInv.ConditionalAssign(&enchantedDenominator, rotate)

	tmp.Mul(&X, &zInv)
	Y.ConditionalNegate(tmp.IsNegative())

	var s field.Element
	s.Sub(&Z, &Y)
	s.Mul(&denInv, &s)

	sIsNegative := s.IsNegative()
	s.ConditionalNegate(sIsNegative)

	_ = s.ToBytes(p[:])

	return p
}

// Equal returns 1 iff the compressed points are equal, 0 otherwise.
// This function will execute in constant-time.
func (p *CompressedRistretto) Equal(other *CompressedRistretto) int {
	return subtle.ConstantTimeCompareBytes(p[:], other[:])
}

// Identity sets the compressed point to the identity element.
func (p *CompressedRistretto) Identity() *CompressedRistretto {
	for i := range p {
		p[i] = 0
	}
	return p
}

// NewCompressedRistretto constructs a new compressed Ristretto point,
// set to the identity element.
func NewCompressedRistretto() *CompressedRistretto {
	var p CompressedRistretto
	return p.Identity()
}

// RistrettoPoint represents a point in the Ristretto group for Curve25519.
//
// The default value is NOT valid and MUST only be used as a receiver.
type RistrettoPoint struct {
	inner EdwardsPoint
}

// MarshalBinary encodes the Ristretto point into a binary form and
// returns the result.
func (p *RistrettoPoint) MarshalBinary() ([]byte, error) {
	var cp CompressedRistretto
	cp.SetRistrettoPoint(p)
	return cp.MarshalBinary()
}

// UnmarshalBinary decodes a binary serialized Ristretto point.
func (p *RistrettoPoint) UnmarshalBinary(data []byte) error {
	p.Identity() // Foot + gun avoidance.

	var cp CompressedRistretto
	if _, err := cp.SetBytes(data); err != nil {
		return nil
	}
	_, err := p.SetCompressed(&cp)
	return err
}

// Identity sets the Ristretto point to the identity element.
func (p *RistrettoPoint) Identity() *RistrettoPoint {
	p.inner.Identity()
	return p
}

// Set sets `p = t`, and returns p.
func (p *RistrettoPoint) Set(t *RistrettoPoint) *RistrettoPoint {
	p.inner.Set(&t.inner)
	return p
}

// SetExpanded sets the Ristretto point to the expanded point.
func (p *RistrettoPoint) SetExpanded(expandedPoint *ExpandedRistrettoPoint) *RistrettoPoint {
	p.inner.Set(&expandedPoint.inner.point)
	return p
}

// SetCompressed attempts to decompress a CompressedRistretto into a
// RistrettoPoint.
func (p *RistrettoPoint) SetCompressed(compressed *CompressedRistretto) (*RistrettoPoint, error) {
	// Step 1. Check s for validity:
	// 1.a) s must be 32 bytes (we get this from the type system)
	// 1.b) s < p
	// 1.c) s is nonnegative
	//
	// Our decoding routine ignores the high bit, so the only
	// possible failure for 1.b) is if someone encodes s in 0..18
	// as s+p in 2^255-19..2^255-1.  We can check this by
	// converting back to bytes, and checking that we get the
	// original input, since our encoding routine is canonical.

	var (
		s           field.Element
		sBytesCheck [field.ElementSize]byte
	)
	if _, err := s.SetBytes(compressed[:]); err != nil {
		return nil, fmt.Errorf("curve/ristretto: failed to deserialize s: %w", err)
	}
	_ = s.ToBytes(sBytesCheck[:])
	sEncodingIsCanonical := subtle.ConstantTimeCompareBytes(compressed[:], sBytesCheck[:])
	sIsNegative := s.IsNegative()

	if sEncodingIsCanonical != 1 || sIsNegative == 1 {
		return nil, fmt.Errorf("curve/ristretto: s is not a canonical encoding")
	}

	// Step 2. Compute (X:Y:Z:T).
	var u1, u2, ss, u1Sqr, u2Sqr field.Element
	ss.Square(&s)
	u1.Sub(&field.One, &ss) // 1 + as^2
	u2.Add(&field.One, &ss) // 1 - as^2 where a = -1
	u1Sqr.Square(&u1)
	u2Sqr.Square(&u2)

	// v == ad(1+as^2)^2 - (1-as^2)^2 where d=-121665/121666
	var v field.Element
	v.Neg(&constEDWARDS_D)
	v.Mul(&v, &u1Sqr)
	v.Sub(&v, &u2Sqr)

	var I field.Element
	I.Mul(&v, &u2Sqr)
	_, ok := I.InvSqrt() // 1/sqrt(v*u_2^2)

	var Dx, Dy field.Element
	Dx.Mul(&I, &u2) // 1/sqrt(v)
	Dy.Mul(&Dx, &v)
	Dy.Mul(&I, &Dy) // 1/u2

	// x == | 2s/sqrt(v) | == + sqrt(4s^2/(ad(1+as^2)^2 - (1-as^2)^2))
	var x field.Element
	x.Add(&s, &s)
	x.Mul(&x, &Dx)
	x.ConditionalNegate(x.IsNegative())

	// y == (1-as^2)/(1+as^2)
	var y field.Element
	y.Mul(&u1, &Dy)

	// t == ((1+as^2) sqrt(4s^2/(ad(1+as^2)^2 - (1-as^2)^@)))/(1-as^2)
	var t field.Element
	t.Mul(&x, &y)

	if ok != 1 || t.IsNegative() == 1 || y.IsZero() == 1 {
		return nil, fmt.Errorf("curve/ristretto: s is is not a valid point")
	}

	p.inner = EdwardsPoint{edwardsPointInner{x, y, field.One, t}}

	return p, nil
}

// SetRandom sets the point to one chosen uniformly at random using entropy
// from the user-provided io.Reader.  If rng is nil, the runtime library's
// entropy source will be used.
func (p *RistrettoPoint) SetRandom(rng io.Reader) (*RistrettoPoint, error) {
	var pointBytes [RistrettoUniformSize]byte

	if rng == nil {
		rng = rand.Reader
	}
	if _, err := io.ReadFull(rng, pointBytes[:]); err != nil {
		return nil, fmt.Errorf("curve/ristretto: failed to read entropy: %w", err)
	}

	return p.SetUniformBytes(pointBytes[:])
}

// SetUniformBytes sets the point to that from 64 bytes of random
// data.  If the input bytes are uniformly distributed, the resulting point
// will be uniformly distributed over the group, and its discrete log with
// respect to other points should be unknown.
func (p *RistrettoPoint) SetUniformBytes(in []byte) (*RistrettoPoint, error) {
	if len(in) != RistrettoUniformSize {
		return nil, fmt.Errorf("curve/ristretto: unexpected input size")
	}
	var (
		r_1, r_2 field.Element
		R_1, R_2 RistrettoPoint
	)
	if _, err := r_1.SetBytes(in[:32]); err != nil {
		return nil, fmt.Errorf("curve/ristretto: failed to deserialize r_1: %w", err)
	}
	R_1.elligatorRistrettoFlavor(&r_1)
	if _, err := r_2.SetBytes(in[32:]); err != nil {
		return nil, fmt.Errorf("curve/ristretto: failed to deserialize r_2: %w", err)
	}
	R_2.elligatorRistrettoFlavor(&r_2)

	// Applying Elligator twice and adding the results ensures a
	// uniform distribution.
	p.Add(&R_1, &R_2)

	return p, nil
}

// ConditionalSelect sets the point to a iff choice == 0 and b iff
// choice == 1.
func (p *RistrettoPoint) ConditionalSelect(a, b *RistrettoPoint, choice int) {
	p.inner.ConditionalSelect(&a.inner, &b.inner, choice)
}

// Equal returns 1 iff the points are equal, 0 otherwise. This function
// will execute in constant-time.
func (p *RistrettoPoint) Equal(other *RistrettoPoint) int {
	pI, oI := &p.inner, &other.inner // Make this look less ugly.
	var X1Y2, Y1X2, X1X2, Y1Y2 field.Element
	X1Y2.Mul(&pI.inner.X, &oI.inner.Y)
	Y1X2.Mul(&pI.inner.Y, &oI.inner.X)
	X1X2.Mul(&pI.inner.X, &oI.inner.X)
	Y1Y2.Mul(&pI.inner.Y, &oI.inner.Y)

	return X1Y2.Equal(&Y1X2) | X1X2.Equal(&Y1Y2)
}

// Add sets `p = a + b`, and returns p.
func (p *RistrettoPoint) Add(a, b *RistrettoPoint) *RistrettoPoint {
	p.inner.Add(&a.inner, &b.inner)
	return p
}

// Sub sets `p = a - b`, and returns p.
func (p *RistrettoPoint) Sub(a, b *RistrettoPoint) *RistrettoPoint {
	p.inner.Sub(&a.inner, &b.inner)
	return p
}

// Sum sets p to the sum of values, and returns p.
func (p *RistrettoPoint) Sum(values []*RistrettoPoint) *RistrettoPoint {
	p.Identity()
	for _, v := range values {
		p.Add(p, v)
	}
	return p
}

// Neg sets `p = -t`, and returns p.
func (p *RistrettoPoint) Neg(t *RistrettoPoint) *RistrettoPoint {
	p.inner.Neg(&t.inner)
	return p
}

// Mul sets `p = point * scalar` in constant-time (variable-base scalar
// multiplication), and returns p.
func (p *RistrettoPoint) Mul(point *RistrettoPoint, scalar *scalar.Scalar) *RistrettoPoint {
	p.inner.Mul(&point.inner, scalar)
	return p
}

// MulBasepoint sets `p = basepoint * scalar` in constant-time, and returns p.
func (p *RistrettoPoint) MulBasepoint(basepoint *RistrettoBasepointTable, scalar *scalar.Scalar) *RistrettoPoint {
	p.inner.MulBasepoint(&basepoint.inner, scalar)
	return p
}

// DoubleScalarMulBasepointVartime sets `p = (aA + bB)` in variable-time,
// where B is the Ristretto basepoint, and returns p.
func (p *RistrettoPoint) DoubleScalarMulBasepointVartime(a *scalar.Scalar, A *RistrettoPoint, b *scalar.Scalar) *RistrettoPoint {
	p.inner.DoubleScalarMulBasepointVartime(a, &A.inner, b)
	return p
}

// TripleScalarMulBasepoint sets `p = [delta a]A + [delta b]B - [delta]C`
// in variable-time, where delta is a value invertible mod ell, which
// is selected internally to this method.
func (p *RistrettoPoint) TripleScalarMulBasepointVartime(a *scalar.Scalar, A *RistrettoPoint, b *scalar.Scalar, C *RistrettoPoint) *RistrettoPoint {
	p.inner.TripleScalarMulBasepointVartime(a, &A.inner, b, &C.inner)
	return p
}

// MultiscalarMul sets `p = scalars[0] * points[0] + ... scalars[n] * points[n]`
// in constant-time, and returns p.
//
// WARNING: This function will panic if `len(scalars) != len(points)`.
func (p *RistrettoPoint) MultiscalarMul(scalars []*scalar.Scalar, points []*RistrettoPoint) *RistrettoPoint {
	edwardsPoints := make([]*EdwardsPoint, 0, len(points))
	for _, point := range points {
		edwardsPoints = append(edwardsPoints, &point.inner)
	}

	p.inner.MultiscalarMul(scalars, edwardsPoints)
	return p
}

// MultiscalarMulVartime sets `p = scalars[0] * points[0] + ... scalars[n] * points[n]`
// in variable-time, and returns p.
//
// WARNING: This function will panic if `len(scalars) != len(points)`.
func (p *RistrettoPoint) MultiscalarMulVartime(scalars []*scalar.Scalar, points []*RistrettoPoint) *RistrettoPoint {
	edwardsPoints := make([]*EdwardsPoint, 0, len(points))
	for _, point := range points {
		edwardsPoints = append(edwardsPoints, &point.inner)
	}

	p.inner.MultiscalarMulVartime(scalars, edwardsPoints)
	return p
}

// IsIdentity returns true iff the point is equivalent to the identity element
// of the curve.
func (p *RistrettoPoint) IsIdentity() bool {
	var id RistrettoPoint
	return p.Equal(id.Identity()) == 1
}

func (p *RistrettoPoint) elligatorRistrettoFlavor(r_0 *field.Element) {
	c := constMINUS_ONE

	var r field.Element
	r.Square(r_0)
	r.Mul(&field.SQRT_M1, &r)
	var N_s field.Element
	N_s.Add(&r, &field.One)
	N_s.Mul(&N_s, &constONE_MINUS_EDWARDS_D_SQUARED)
	var D, tmp field.Element
	tmp.Add(&r, &constEDWARDS_D)
	D.Mul(&constEDWARDS_D, &r)
	D.Sub(&c, &D)
	D.Mul(&D, &tmp)

	var s, s_prime field.Element
	_, Ns_D_is_sq := s.SqrtRatioI(&N_s, &D)
	s_prime.Mul(&s, r_0)
	s_prime_is_pos := s_prime.IsNegative() ^ 1
	s_prime.ConditionalNegate(s_prime_is_pos)

	Ns_D_is_not_sq := Ns_D_is_sq ^ 1

	s.ConditionalAssign(&s_prime, Ns_D_is_not_sq)
	c.ConditionalAssign(&r, Ns_D_is_not_sq)

	var N_t field.Element
	N_t.Sub(&r, &field.One)
	N_t.Mul(&c, &N_t)
	N_t.Mul(&N_t, &constEDWARDS_D_MINUS_ONE_SQUARED)
	N_t.Sub(&N_t, &D)

	var s_sq field.Element
	s_sq.Square(&s)

	var cp completedPoint
	cp.X.Add(&s, &s)
	cp.X.Mul(&cp.X, &D)
	cp.Z.Mul(&N_t, &constSQRT_AD_MINUS_ONE)
	cp.Y.Sub(&field.One, &s_sq)
	cp.T.Add(&field.One, &s_sq)

	// The conversion from W_i is exactly the conversion from P1xP1.
	p.inner.setCompleted(&cp)
}

// NewRistrettoPoint constructs a new Ristretto point set to the identity element.
func NewRistrettoPoint() *RistrettoPoint {
	var p RistrettoPoint
	return p.Identity()
}

// RistrettoBasepointTable defines a precomputed table of multiples of a
// basepoint, for accelerating fixed-based scalar multiplication.
type RistrettoBasepointTable struct {
	inner EdwardsBasepointTable
}

// Basepoint returns the basepoint of the table.
func (tbl *RistrettoBasepointTable) Basepoint() *RistrettoPoint {
	return &RistrettoPoint{
		inner: *tbl.inner.Basepoint(),
	}
}

// NewRistrettoBasepointTable creates a table of precomputed multiples of
// `basepoint`.
func NewRistrettoBasepointTable(basepoint *RistrettoPoint) *RistrettoBasepointTable {
	return &RistrettoBasepointTable{
		inner: *NewEdwardsBasepointTable(&basepoint.inner),
	}
}

// Omitted:
//  * DoubleAndCompressBatch
