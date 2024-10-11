// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

package secp256k1

import "gitlab.com/yawning/secp256k1-voi/internal/field"

// These are the internal routines specific to working with points
// represented in projective coordinates.
//
// As an explicit simplicity/performance tradeoff, this representation
// was chosen so that it is possible to use the complete addition
// formulas.
//
// See:
// - https://eprint.iacr.org/2015/1060.pdf
// - https://hyperelliptic.org/EFD/g1p/auto-shortw-projective.html

// feB3 is the constant `b * 3`, used in the point addition algorithm.
var feB3 = field.NewElementFromUint64(7 * 3)

// addComplete sets `v = p + q`, and returns `v`.
func (v *Point) addComplete(p, q *Point) *Point {
	// Algorithm 7 from "Complete addition formulas for prime
	// order elliptic curves" by Renes, Costello, and Batina.
	//
	// The formula is complete in that it is valid for all p and q,
	// without exceptions or extra assumptions about the inputs.
	//
	// The operation costs are `12M + 2m3b + 19a`, though our current
	// field implmentation lacks a multiply tailored for small
	// multiples (`m3b`).
	//
	// If you are looking to use this formula for something else
	// note that it is specialized for `a = 0`.

	var (
		t0 = field.NewElement()
		t1 = field.NewElement()
		t2 = field.NewElement()
		t3 = field.NewElement()
		t4 = field.NewElement()

		x1 = &p.x
		y1 = &p.y
		z1 = &p.z

		x2 = &q.x
		y2 = &q.y
		z2 = &q.z

		// To make this alias-safe, allocate these.
		x3 = field.NewElement()
		y3 = field.NewElement()
		z3 = field.NewElement()
	)

	// t0 := X1 * X2 ; t1 := Y1 * Y2 ; t2 := Z1 * Z2 ;
	t0.Multiply(x1, x2)
	t1.Multiply(y1, y2)
	t2.Multiply(z1, z2)

	// t3 := X1 + Y1 ; t4 := X2 + Y2 ; t3 := t3 * t4 ;
	t3.Add(x1, y1)
	t4.Add(x2, y2)
	t3.Multiply(t3, t4)

	// t4 := t0 + t1 ; t3 := t3 - t4 ; t4 := Y1 + Z1 ;
	t4.Add(t0, t1)
	t3.Subtract(t3, t4)
	t4.Add(y1, z1)

	// X3 := Y2 + Z2 ; t4 := t4 * X3 ; X3 := t1 + t2 ;
	x3.Add(y2, z2)
	t4.Multiply(t4, x3)
	x3.Add(t1, t2)

	// t4 := t4 - X3 ; X3 := X1 + Z1 ; Y3 := X2 + Z2 ;
	t4.Subtract(t4, x3)
	x3.Add(x1, z1)
	y3.Add(x2, z2)

	// X3 := X3 * Y3 ; Y3 := t0 + t2 ; Y3 := X3 - Y3 ;
	x3.Multiply(x3, y3)
	y3.Add(t0, t2)
	y3.Subtract(x3, y3)

	// X3 := t0 + t0 ; t0 := X3 + t0 ; t2 := b3 * t2 ;
	x3.Add(t0, t0)
	t0.Add(x3, t0)
	t2.Multiply(feB3, t2)

	// Z3 := t1 + t2 ; t1 := t1 - t2 ; Y3 := b3 * Y3 ;
	z3.Add(t1, t2)
	t1.Subtract(t1, t2)
	y3.Multiply(feB3, y3)

	// X3 := t4 * Y3 ; t2 := t3 * t1 ; X3 := t2 - X3 ;
	x3.Multiply(t4, y3)
	t2.Multiply(t3, t1)
	x3.Subtract(t2, x3)

	// Y3 := Y3 * t0 ; t1 := t1 * Z3 ; Y3 := t1 + Y3 ;
	y3.Multiply(y3, t0)
	t1.Multiply(t1, z3)
	y3.Add(t1, y3)

	// t0 := t0 * t3 ; Z3 := Z3 * t4 ; Z3 := Z3 + t0 ;
	t0.Multiply(t0, t3)
	z3.Multiply(z3, t4)
	z3.Add(z3, t0)

	// return X3 , Y3 , Z3 ;
	v.x.Set(x3)
	v.y.Set(y3)
	v.z.Set(z3)

	return v
}

// addMixed sets `v = p + (x2, y2, 1)`, and returns `v`.
func (v *Point) addMixed(p *Point, x2, y2 *field.Element) *Point {
	// Algorithm 8 from "Complete addition formulas for prime
	// order elliptic curves" by Renes, Costello, and Batina.
	//
	// The formula is mixed in that it assumes the z-coordinate
	// of the addend (`Z2`) is `1`, meaning that it CAN NOT
	// handle the addend being the point at infinity.
	//
	// The operation costs are `11M + 2m3b + 13a`, though our current
	// field implmentation lacks a multiply tailored for small
	// multiples (`m3b`).  This saves `1M + 6a` over `addComplete`.
	//
	// If you are looking to use this formula for something else
	// note that it is specialized for `a = 0`.

	var (
		t0 = field.NewElement()
		t1 = field.NewElement()
		t2 = field.NewElement()
		t3 = field.NewElement()
		t4 = field.NewElement()

		x1 = &p.x
		y1 = &p.y
		z1 = &p.z

		// To make this alias-safe, allocate these.
		x3 = field.NewElement()
		y3 = field.NewElement()
		z3 = field.NewElement()
	)

	// t0 := X1 * X2 ; t1 := Y1 * Y2 ; t3 := X2 + Y2 ;
	t0.Multiply(x1, x2)
	t1.Multiply(y1, y2)
	t3.Add(x2, y2)

	// t4 := X1 + Y1 ; t3 := t3 * t4 ; t4 := t0 + t1 ;
	t4.Add(x1, y1)
	t3.Multiply(t3, t4)
	t4.Add(t0, t1)

	// t3 := t3 - t4 ; t4 := Y2 * Z1 ; t4 := t4 + Y1 ;
	t3.Subtract(t3, t4)
	t4.Multiply(y2, z1)
	t4.Add(t4, y1)

	// Y3 := X2 * Z1 ; Y3 := Y3 + X1 ; X3 := t0 + t0 ;
	y3.Multiply(x2, z1)
	y3.Add(y3, x1)
	x3.Add(t0, t0)

	// t0 := X3 + t0 ; t2 := b3 * Z1 ; Z3 := t1 + t2 ;
	t0.Add(x3, t0)
	t2.Multiply(feB3, z1)
	z3.Add(t1, t2)

	// t1 := t1 - t2 ; Y3 := b3 * Y3 ; X3 := t4 * Y3 ;
	t1.Subtract(t1, t2)
	y3.Multiply(feB3, y3)
	x3.Multiply(t4, y3)

	// t2 := t3 * t1 ; X3 := t2 - X3 ; Y3 := Y3 * t0 ;
	t2.Multiply(t3, t1)
	x3.Subtract(t2, x3)
	y3.Multiply(y3, t0)

	// t1 := t1 * Z3 ; Y3 := t1 + Y3 ; t0 := t0 * t3 ;
	t1.Multiply(t1, z3)
	y3.Add(t1, y3)
	t0.Multiply(t0, t3)

	// Z3 := Z3 * t4 ; Z3 := Z3 + t0
	z3.Multiply(z3, t4)
	z3.Add(z3, t0)

	// return X3 , Y3 , Z3 ;
	v.x.Set(x3)
	v.y.Set(y3)
	v.z.Set(z3)

	return v
}

// doubleComplete sets `v = p + p`, and returns `v`.
func (v *Point) doubleComplete(p *Point) *Point { //nolint:unparam
	// Algorithm 9 from "Complete addition formulas for prime
	// order elliptic curves" by Renes, Costello, and Batina.
	//
	// The formula is complete in that it is valid for all p,
	// without exceptions or extra assumptions about the inputs.
	//
	// The operation costs are `6M + 2S + 1m3b + 9a`, though our
	// current field implmentation lacks a multiply tailored for
	// small multiples (`m3b`).
	//
	// If you are looking to use this formula for something else
	// note that it is specialized for `a = 0`.

	var (
		t0 = field.NewElement()
		t1 = field.NewElement()
		t2 = field.NewElement()

		x = &p.x
		y = &p.y
		z = &p.z

		// To make this alias-safe, allocate these.
		x3 = field.NewElement()
		y3 = field.NewElement()
		z3 = field.NewElement()
	)

	// t0 := Y ^2; Z3 := t0 + t0 ; Z3 := Z3 + Z3 ;
	t0.Square(y)
	z3.Add(t0, t0)
	z3.Add(z3, z3)

	// Z3 := Z3 + Z3 ; t1 := Y * Z ; t2 := Z ^2;
	z3.Add(z3, z3)
	t1.Multiply(y, z)
	t2.Square(z)

	// t2 := b3 * t2 ; X3 := t2 * Z3 ; Y3 := t0 + t2 ;
	t2.Multiply(feB3, t2)
	x3.Multiply(t2, z3)
	y3.Add(t0, t2)

	// Z3 := t1 * Z3 ; t1 := t2 + t2 ; t2 := t1 + t2 ;
	z3.Multiply(t1, z3)
	t1.Add(t2, t2)
	t2.Add(t1, t2)

	// t0 := t0 - t2 ; Y3 := t0 * Y3 ; Y3 := X3 + Y3 ;
	t0.Subtract(t0, t2)
	y3.Multiply(t0, y3)
	y3.Add(x3, y3)

	// t1 := X * Y ; X3 := t0 * t1 ; X3 := X3 + X3 ;
	t1.Multiply(x, y)
	x3.Multiply(t0, t1)
	x3.Add(x3, x3)

	// return X3 , Y3 , Z3 ;
	v.x.Set(x3)
	v.y.Set(y3)
	v.z.Set(z3)

	return v
}

// rescale scales the point such that Z = 1.
//
// Note: This is quite expensive, and should only done when serializing points.
func (v *Point) rescale(p *Point) *Point {
	assertPointsValid(p)

	// A = 1/Z1
	// X3 = A*X1
	// Y3 = A*Y1
	// Z3 = 1
	//
	// As per "From A to Z: Projective coordinates leakage in the wild"
	// leaking the Z-coordinate is bad.  The modular inversion algorithm
	// used in this library is based on Fermat's Little Theorem
	// (ie: Z^-1 = Z^(p -2) mod p). Bernstein-Yang also would be safe.
	//
	// See: https://eprint.iacr.org/2020/432.pdf

	scaled := newRcvr()
	a := field.NewElement().Invert(&p.z)
	scaled.x.Multiply(a, &p.x)
	scaled.y.Multiply(a, &p.y)
	scaled.z.One()
	scaled.isValid = p.isValid

	// Iff p is the point at infinity, set v to (0, 1, 0).
	return v.ConditionalSelect(scaled, NewIdentityPoint(), p.IsIdentity())
}
