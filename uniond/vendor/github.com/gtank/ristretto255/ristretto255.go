// Copyright 2019 The Go Authors. All rights reserved.
// Copyright 2019 George Tankersley. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Package ristretto255 implements the group of prime order
//
//     2**252 + 27742317777372353535851937790883648493
//
// as specified in draft-hdevalence-cfrg-ristretto-01.
//
// All operations are constant time unless otherwise specified.
package ristretto255

import (
	"bytes"
	"encoding/base64"
	"errors"

	"github.com/gtank/ristretto255/internal/edwards25519"
	"github.com/gtank/ristretto255/internal/radix51"
	"github.com/gtank/ristretto255/internal/scalar"
)

// Constants from draft-hdevalence-cfrg-ristretto-01, Section 3.1.
var (
	sqrtM1 = fieldElementFromDecimal(
		"19681161376707505956807079304988542015446066515923890162744021073123829784752")
	sqrtADMinusOne = fieldElementFromDecimal(
		"25063068953384623474111414158702152701244531502492656460079210482610430750235")
	invSqrtAMinusD = fieldElementFromDecimal(
		"54469307008909316920995813868745141605393597292927456921205312896311721017578")
	oneMinusDSQ = fieldElementFromDecimal(
		"1159843021668779879193775521855586647937357759715417654439879720876111806838")
	dMinusOneSQ = fieldElementFromDecimal(
		"40440834346308536858101042469323190826248399146238708352240133220865137265952")
)

// Element is an element of the ristretto255 prime-order group.
type Element struct {
	r edwards25519.ProjP3
}

// NewElement returns a new Element set to the identity value.
func NewElement() *Element {
	return (&Element{}).Zero()
}

// Equal returns 1 if e is equivalent to ee, and 0 otherwise.
//
// Note that Elements must not be compared in any other way.
func (e *Element) Equal(ee *Element) int {
	var f0, f1 radix51.FieldElement

	f0.Mul(&e.r.X, &ee.r.Y) // x1 * y2
	f1.Mul(&e.r.Y, &ee.r.X) // y1 * x2
	out := f0.Equal(&f1)

	f0.Mul(&e.r.Y, &ee.r.Y) // y1 * y2
	f1.Mul(&e.r.X, &ee.r.X) // x1 * x2
	out = out | f0.Equal(&f1)

	return out
}

// FromUniformBytes maps the 64-byte slice b to e uniformly and
// deterministically, and returns e. This can be used for hash-to-group
// operations or to obtain a random element.
func (e *Element) FromUniformBytes(b []byte) *Element {
	if len(b) != 64 {
		panic("ristretto255: FromUniformBytes: input is not 64 bytes long")
	}

	f := &radix51.FieldElement{}

	f.FromBytes(b[:32])
	point1 := &Element{}
	mapToPoint(&point1.r, f)

	f.FromBytes(b[32:])
	point2 := &Element{}
	mapToPoint(&point2.r, f)

	return e.Add(point1, point2)
}

// mapToPoint implements MAP from Section 3.2.4 of draft-hdevalence-cfrg-ristretto-00.
func mapToPoint(out *edwards25519.ProjP3, t *radix51.FieldElement) {
	// r = SQRT_M1 * t^2
	r := &radix51.FieldElement{}
	r.Mul(sqrtM1, r.Square(t))

	// u = (r + 1) * ONE_MINUS_D_SQ
	u := &radix51.FieldElement{}
	u.Mul(u.Add(r, radix51.One), oneMinusDSQ)

	// c = -1
	c := &radix51.FieldElement{}
	c.Set(radix51.MinusOne)

	// v = (c - r*D) * (r + D)
	rPlusD := &radix51.FieldElement{}
	rPlusD.Add(r, edwards25519.D)
	v := &radix51.FieldElement{}
	v.Mul(v.Sub(c, v.Mul(r, edwards25519.D)), rPlusD)

	// (was_square, s) = SQRT_RATIO_M1(u, v)
	s := &radix51.FieldElement{}
	wasSquare := feSqrtRatio(s, u, v)

	// s_prime = -CT_ABS(s*t)
	sPrime := &radix51.FieldElement{}
	sPrime.Neg(sPrime.Abs(sPrime.Mul(s, t)))

	// s = CT_SELECT(s IF was_square ELSE s_prime)
	s.Select(s, sPrime, wasSquare)
	// c = CT_SELECT(c IF was_square ELSE r)
	c.Select(c, r, wasSquare)

	// N = c * (r - 1) * D_MINUS_ONE_SQ - v
	N := &radix51.FieldElement{}
	N.Mul(c, N.Sub(r, radix51.One))
	N.Sub(N.Mul(N, dMinusOneSQ), v)

	s2 := &radix51.FieldElement{}
	s2.Square(s)

	// w0 = 2 * s * v
	w0 := &radix51.FieldElement{}
	w0.Add(w0, w0.Mul(s, v))
	// w1 = N * SQRT_AD_MINUS_ONE
	w1 := &radix51.FieldElement{}
	w1.Mul(N, sqrtADMinusOne)
	// w2 = 1 - s^2
	w2 := &radix51.FieldElement{}
	w2.Sub(radix51.One, s2)
	// w3 = 1 + s^2
	w3 := &radix51.FieldElement{}
	w3.Add(radix51.One, s2)

	// return (w0*w3, w2*w1, w1*w3, w0*w2)
	out.X.Mul(w0, w3)
	out.Y.Mul(w2, w1)
	out.Z.Mul(w1, w3)
	out.T.Mul(w0, w2)
}

// Encode appends the 32 bytes canonical encoding of e to b
// and returns the result.
func (e *Element) Encode(b []byte) []byte {
	tmp := &radix51.FieldElement{}

	// u1 = (z0 + y0) * (z0 - y0)
	u1 := &radix51.FieldElement{}
	u1.Add(&e.r.Z, &e.r.Y).Mul(u1, tmp.Sub(&e.r.Z, &e.r.Y))

	// u2 = x0 * y0
	u2 := &radix51.FieldElement{}
	u2.Mul(&e.r.X, &e.r.Y)

	// Ignore was_square since this is always square
	// (_, invsqrt) = SQRT_RATIO_M1(1, u1 * u2^2)
	invSqrt := &radix51.FieldElement{}
	feSqrtRatio(invSqrt, radix51.One, tmp.Square(u2).Mul(tmp, u1))

	// den1 = invsqrt * u1
	// den2 = invsqrt * u2
	den1, den2 := &radix51.FieldElement{}, &radix51.FieldElement{}
	den1.Mul(invSqrt, u1)
	den2.Mul(invSqrt, u2)
	// z_inv = den1 * den2 * t0
	zInv := &radix51.FieldElement{}
	zInv.Mul(den1, den2).Mul(zInv, &e.r.T)

	// ix0 = x0 * SQRT_M1
	// iy0 = y0 * SQRT_M1
	ix0, iy0 := &radix51.FieldElement{}, &radix51.FieldElement{}
	ix0.Mul(&e.r.X, sqrtM1)
	iy0.Mul(&e.r.Y, sqrtM1)
	// enchanted_denominator = den1 * INVSQRT_A_MINUS_D
	enchantedDenominator := &radix51.FieldElement{}
	enchantedDenominator.Mul(den1, invSqrtAMinusD)

	// rotate = IS_NEGATIVE(t0 * z_inv)
	rotate := tmp.Mul(&e.r.T, zInv).IsNegative()

	// x = CT_SELECT(iy0 IF rotate ELSE x0)
	// y = CT_SELECT(ix0 IF rotate ELSE y0)
	x, y := &radix51.FieldElement{}, &radix51.FieldElement{}
	x.Select(iy0, &e.r.X, rotate)
	y.Select(ix0, &e.r.Y, rotate)
	// z = z0
	z := &e.r.Z
	// den_inv = CT_SELECT(enchanted_denominator IF rotate ELSE den2)
	denInv := &radix51.FieldElement{}
	denInv.Select(enchantedDenominator, den2, rotate)

	// y = CT_NEG(y, IS_NEGATIVE(x * z_inv))
	y.CondNeg(y, tmp.Mul(x, zInv).IsNegative())

	// s = CT_ABS(den_inv * (z - y))
	s := tmp.Sub(z, y).Mul(tmp, denInv).Abs(tmp)

	// Return the canonical little-endian encoding of s.
	return s.Bytes(b)
}

var errInvalidEncoding = errors.New("invalid Ristretto encoding")

// Decode sets e to the decoded value of in. If in is not a 32 byte canonical
// encoding, Decode returns an error, and the receiver is unchanged.
func (e *Element) Decode(in []byte) error {
	if len(in) != 32 {
		return errInvalidEncoding
	}

	// First, interpret the string as an integer s in little-endian representation.
	s := &radix51.FieldElement{}
	s.FromBytes(in)

	// If the resulting value is >= p, decoding fails.
	var buf [32]byte
	if !bytes.Equal(s.Bytes(buf[:0]), in) {
		return errInvalidEncoding
	}

	// If IS_NEGATIVE(s) returns TRUE, decoding fails.
	if s.IsNegative() == 1 {
		return errInvalidEncoding
	}

	// ss = s^2
	sSqr := &radix51.FieldElement{}
	sSqr.Square(s)

	// u1 = 1 - ss
	u1 := &radix51.FieldElement{}
	u1.Sub(radix51.One, sSqr)

	// u2 = 1 + ss
	u2 := &radix51.FieldElement{}
	u2.Add(radix51.One, sSqr)

	// u2_sqr = u2^2
	u2Sqr := &radix51.FieldElement{}
	u2Sqr.Square(u2)

	// v = -(D * u1^2) - u2_sqr
	v := &radix51.FieldElement{}
	v.Square(u1).Mul(v, edwards25519.D).Neg(v).Sub(v, u2Sqr)

	// (was_square, invsqrt) = SQRT_RATIO_M1(1, v * u2_sqr)
	invSqrt, tmp := &radix51.FieldElement{}, &radix51.FieldElement{}
	wasSquare := feSqrtRatio(invSqrt, radix51.One, tmp.Mul(v, u2Sqr))

	// den_x = invsqrt * u2
	// den_y = invsqrt * den_x * v
	denX, denY := &radix51.FieldElement{}, &radix51.FieldElement{}
	denX.Mul(invSqrt, u2)
	denY.Mul(invSqrt, denX).Mul(denY, v)

	// x = CT_ABS(2 * s * den_x)
	// y = u1 * den_y
	// t = x * y
	var out edwards25519.ProjP3
	out.X.Mul(radix51.Two, s).Mul(&out.X, denX).Abs(&out.X)
	out.Y.Mul(u1, denY)
	out.Z.One()
	out.T.Mul(&out.X, &out.Y)

	// If was_square is FALSE, or IS_NEGATIVE(t) returns TRUE, or y = 0, decoding fails.
	if wasSquare == 0 || out.T.IsNegative() == 1 || out.Y.Equal(radix51.Zero) == 1 {
		return errInvalidEncoding
	}

	// Otherwise, return the internal representation in extended coordinates (x, y, 1, t).
	e.r.Set(&out)
	return nil
}

// ScalarBaseMult sets e = s * B, where B is the canonical generator, and returns e.
func (e *Element) ScalarBaseMult(s *Scalar) *Element {
	e.r.BasepointMul(&s.s)
	return e
}

// ScalarMult sets e = s * p, and returns e.
func (e *Element) ScalarMult(s *Scalar, p *Element) *Element {
	e.r.ScalarMul(&s.s, &p.r)
	return e
}

// MultiScalarMult sets e = sum(s[i] * p[i]), and returns e.
//
// Execution time depends only on the lengths of the two slices, which must match.
func (e *Element) MultiScalarMult(s []*Scalar, p []*Element) *Element {
	if len(p) != len(s) {
		panic("ristretto255: MultiScalarMult invoked with mismatched slice lengths")
	}
	points := make([]*edwards25519.ProjP3, len(p))
	scalars := make([]scalar.Scalar, len(s))
	for i := range s {
		points[i] = &p[i].r
		scalars[i] = s[i].s
	}
	e.r.MultiscalarMul(scalars, points)
	return e
}

// VarTimeMultiScalarMult sets e = sum(s[i] * p[i]), and returns e.
//
// Execution time depends on the inputs.
func (e *Element) VarTimeMultiScalarMult(s []*Scalar, p []*Element) *Element {
	if len(p) != len(s) {
		panic("ristretto255: MultiScalarMult invoked with mismatched slice lengths")
	}
	points := make([]*edwards25519.ProjP3, len(p))
	scalars := make([]scalar.Scalar, len(s))
	for i := range s {
		points[i] = &p[i].r
		scalars[i] = s[i].s
	}
	e.r.VartimeMultiscalarMul(scalars, points)
	return e
}

// VarTimeDoubleScalarBaseMult sets e = a * A + b * B, where B is the canonical
// generator, and returns e.
//
// Execution time depends on the inputs.
func (e *Element) VarTimeDoubleScalarBaseMult(a *Scalar, A *Element, b *Scalar) *Element {
	e.r.VartimeDoubleBaseMul(&a.s, &A.r, &b.s)
	return e
}

// Add sets e = p + q, and returns e.
func (e *Element) Add(p, q *Element) *Element {
	e.r.Add(&p.r, &q.r)
	return e
}

// Subtract sets e = p - q, and returns e.
func (e *Element) Subtract(p, q *Element) *Element {
	e.r.Sub(&p.r, &q.r)
	return e
}

// Negate sets e = -p, and returns e.
func (e *Element) Negate(p *Element) *Element {
	e.r.Neg(&p.r)
	return e
}

// Zero sets e to the identity element of the group, and returns e.
func (e *Element) Zero() *Element {
	e.r.Zero()
	return e
}

// Base sets e to the canonical generator specified in
// draft-hdevalence-cfrg-ristretto-01, Section 3, and returns e.
func (e *Element) Base() *Element {
	e.r.Set(&edwards25519.B)
	return e
}

// MarshalText implements encoding/TextMarshaler interface
func (e *Element) MarshalText() (text []byte, err error) {
	b := e.Encode([]byte{})
	return []byte(base64.StdEncoding.EncodeToString(b)), nil
}

// UnmarshalText implements encoding/TextMarshaler interface
func (e *Element) UnmarshalText(text []byte) error {
	eb, err := base64.StdEncoding.DecodeString(string(text))
	if err == nil {
		return e.Decode(eb)
	}
	return err
}

// String implements the Stringer interface
func (e *Element) String() string {
	result, _ := e.MarshalText()
	return string(result)
}
