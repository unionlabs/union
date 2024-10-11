// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

// Package secp256k1 implements the secp256k1 elliptic curve as specified
// in SEC 2, Version 2.0, Section 2.4.1.
package secp256k1

import (
	"fmt"

	"gitlab.com/yawning/secp256k1-voi/internal/disalloweq"
	"gitlab.com/yawning/secp256k1-voi/internal/field"
)

var (
	// feGX is the x-coordinate of the generator.
	feGX = field.NewElementFromCanonicalHex("0x79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798")

	// feGY is the y-coordinate of the generator.
	feGY = field.NewElementFromCanonicalHex("0x483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8")
)

// Point represets a point on the secp256k1 curve.  All arguments and
// receivers are allowed to alias.  The zero value is NOT valid, and
// may only be used as a receiver.
//
// Properly initialized Points will always either be on the curve, or
// the point at infinity, and all of the curve arithmetic routines
// handle the point at infinity correctly.
type Point struct {
	_ disalloweq.DisallowEqual

	// The point internally is represented in projective coordinates
	// (X, Y, Z) where x = X/Z y = Y/Z.
	x, y, z field.Element

	isValid bool
}

// Identity sets `v = id`, and returns `v`.
func (v *Point) Identity() *Point {
	v.x.Zero()
	v.y.One()
	v.z.Zero()
	v.isValid = true

	return v
}

// Generator sets `v = G`, and returns `v`.
func (v *Point) Generator() *Point {
	v.x.Set(feGX)
	v.y.Set(feGY)
	v.z.One()
	v.isValid = true

	return v
}

// Add sets `v = p + q`, and returns `v`.
func (v *Point) Add(p, q *Point) *Point {
	assertPointsValid(p, q)

	v.addComplete(p, q)
	v.isValid = p.isValid && q.isValid

	return v
}

// Double sets `v = p + p`, and returns `v`.  Calling `Add(p, p)` will
// also return correct results, however this method is faster.
func (v *Point) Double(p *Point) *Point {
	assertPointsValid(p)

	v.doubleComplete(p)
	v.isValid = p.isValid

	return v
}

// Subtract sets `v = p - q`, and returns `v`.
func (v *Point) Subtract(p, q *Point) *Point {
	assertPointsValid(p, q)
	return v.Add(p, newRcvr().Negate(q))
}

// Negate sets `v = -p`, and returns `v`.
func (v *Point) Negate(p *Point) *Point {
	assertPointsValid(p)

	// Affine negation formulas: -(x1,y1)=(x1,-y1).
	v.x.Set(&p.x)
	v.y.Negate(&p.y)
	v.z.Set(&p.z)
	v.isValid = p.isValid

	return v
}

// ConditionalNegate sets `v = p` iff `ctrl == 0`, `v = -p` otherwise,
// and returns `v`.
func (v *Point) ConditionalNegate(p *Point, ctrl uint64) *Point {
	assertPointsValid(p)

	v.x.Set(&p.x)
	v.y.ConditionalNegate(&p.y, ctrl)
	v.z.Set(&p.z)
	v.isValid = p.isValid

	return v
}

// ConditionalSelect sets `v = a` iff `ctrl == 0`, `v = b` otherwise,
// and returns `v`.
func (v *Point) ConditionalSelect(a, b *Point, ctrl uint64) *Point {
	assertPointsValid(a, b)

	v.uncheckedConditionalSelect(a, b, ctrl)
	v.isValid = a.isValid && b.isValid

	return v
}

func (v *Point) uncheckedConditionalSelect(a, b *Point, ctrl uint64) *Point {
	v.x.ConditionalSelect(&a.x, &b.x, ctrl)
	v.y.ConditionalSelect(&a.y, &b.y, ctrl)
	v.z.ConditionalSelect(&a.z, &b.z, ctrl)

	return v
}

// Equal returns 1 iff `v == p`, 0 otherwise.
func (v *Point) Equal(p *Point) uint64 {
	assertPointsValid(v, p)

	// Check X1Z2 == X2Z1 Y1Z2 == Y2Z1
	x1z2 := field.NewElement().Multiply(&v.x, &p.z)
	x2z1 := field.NewElement().Multiply(&p.x, &v.z)

	y1z2 := field.NewElement().Multiply(&v.y, &p.z)
	y2z1 := field.NewElement().Multiply(&p.y, &v.z)

	return x1z2.Equal(x2z1) & y1z2.Equal(y2z1)
}

// IsIdentity returns 1 iff `v` is the identity point, 0 otherwise.
func (v *Point) IsIdentity() uint64 {
	assertPointsValid(v)

	return v.z.IsZero()
}

// IsYOdd returns 1 iff `v.y` is odd, 0 otherwise.
func (v *Point) IsYOdd() uint64 {
	assertPointsValid(v)

	scaled := newRcvr().rescale(v) // XXX/perf: Don't need to rescale X.

	return scaled.y.IsOdd()
}

// Set sets `v = p`, and returns `v`.
func (v *Point) Set(p *Point) *Point {
	assertPointsValid(p)

	v.x.Set(&p.x)
	v.y.Set(&p.y)
	v.z.Set(&p.z)
	v.isValid = p.isValid

	return v
}

// NewGeneratorPoint returns a new Point set to the canonical generator.
func NewGeneratorPoint() *Point {
	return newRcvr().Generator()
}

// NewIdentityPoint returns a new Point set to the identity element (point at infinity).
func NewIdentityPoint() *Point {
	// Note: This doesn't use p.Identity(), because x and z are guaranteed
	// to already be 0, and it makes escape analysis upset.

	p := newRcvr()
	// p.x.Zero()
	p.y.One()
	// p.z.Zero()
	p.isValid = true

	return p
}

// NewPointFrom returns a new Point set to an existing Point.
func NewPointFrom(p *Point) *Point {
	assertPointsValid(p)

	return newRcvr().Set(p)
}

// NewPointFromCoords creates a new Point from the big-endian encoded x
// and y coordinates.
func NewPointFromCoords(xBytes, yBytes *[CoordSize]byte) (*Point, error) {
	x, err := field.NewElementFromCanonicalBytes(xBytes)
	if err != nil {
		return nil, fmt.Errorf("secp256k1: invalid x-coordinate: %w", err)
	}
	y, err := field.NewElementFromCanonicalBytes(yBytes)
	if err != nil {
		return nil, fmt.Errorf("secp256k1: invalid y-coordinate: %w", err)
	}

	if xyOnCurve(x, y) != 1 {
		return nil, errPointNotOnCurve
	}

	p := newRcvr()
	p.x.Set(x)
	p.y.Set(y)
	p.z.One()
	p.isValid = true

	return p, nil
}

// assertPointsValid ensures that the points have been initialized.
func assertPointsValid(points ...*Point) {
	for _, p := range points {
		if !p.isValid {
			panic("secp256k1: use of uninitialized Point")
		}
	}
}

func newRcvr() *Point {
	// This is explcitly for nicely creating receivers.
	return &Point{}
}
