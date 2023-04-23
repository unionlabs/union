// Copyright (c) 2017 George Tankersley. All rights reserved.
// Copyright (c) 2019 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Package edwards25519 implements group logic for the twisted Edwards curve
//
//     -x^2 + y^2 = 1 + -(121665/121666)*x^2*y^2
//
// This is better known as the Edwards curve equivalent to curve25519, and is
// the curve used by the Ed25519 signature scheme.
package edwards25519

import (
	"github.com/gtank/ristretto255/internal/radix51"
)

// D is a constant in the curve equation.
var D = &radix51.FieldElement{929955233495203, 466365720129213,
	1662059464998953, 2033849074728123, 1442794654840575}
var d2 = new(radix51.FieldElement).Add(D, D)

// Point types.

// TODO: write documentation
// TODO: rename (T,X,Y,Z) to (W0,W1,W2,W3) for P2 and P3 models?
// https://doc-internal.dalek.rs/curve25519_dalek/backend/serial/curve_models/index.html

type ProjP1xP1 struct {
	X, Y, Z, T radix51.FieldElement
}

type ProjP2 struct {
	X, Y, Z radix51.FieldElement
}

type ProjP3 struct {
	X, Y, Z, T radix51.FieldElement
}

type ProjCached struct {
	YplusX, YminusX, Z, T2d radix51.FieldElement
}

type AffineCached struct {
	YplusX, YminusX, T2d radix51.FieldElement
}

// B is the Ed25519 basepoint.
var B = ProjP3{
	X: radix51.FieldElement([5]uint64{1738742601995546, 1146398526822698, 2070867633025821, 562264141797630, 587772402128613}),
	Y: radix51.FieldElement([5]uint64{1801439850948184, 1351079888211148, 450359962737049, 900719925474099, 1801439850948198}),
	Z: radix51.FieldElement([5]uint64{1, 0, 0, 0, 0}),
	T: radix51.FieldElement([5]uint64{1841354044333475, 16398895984059, 755974180946558, 900171276175154, 1821297809914039}),
}

// Constructors.

func (v *ProjP1xP1) Zero() *ProjP1xP1 {
	v.X.Zero()
	v.Y.One()
	v.Z.One()
	v.T.One()
	return v
}

func (v *ProjP2) Zero() *ProjP2 {
	v.X.Zero()
	v.Y.One()
	v.Z.One()
	return v
}

func (v *ProjP3) Zero() *ProjP3 {
	v.X.Zero()
	v.Y.One()
	v.Z.One()
	v.T.Zero()
	return v
}

func (v *ProjCached) Zero() *ProjCached {
	v.YplusX.One()
	v.YminusX.One()
	v.Z.One()
	v.T2d.Zero()
	return v
}

func (v *AffineCached) Zero() *AffineCached {
	v.YplusX.One()
	v.YminusX.One()
	v.T2d.Zero()
	return v
}

// Assignments.

func (v *ProjP3) Set(u *ProjP3) *ProjP3 {
	*v = *u
	return v
}

// Conversions.

func (v *ProjP2) FromP1xP1(p *ProjP1xP1) *ProjP2 {
	v.X.Mul(&p.X, &p.T)
	v.Y.Mul(&p.Y, &p.Z)
	v.Z.Mul(&p.Z, &p.T)
	return v
}

func (v *ProjP2) FromP3(p *ProjP3) *ProjP2 {
	v.X.Set(&p.X)
	v.Y.Set(&p.Y)
	v.Z.Set(&p.Z)
	return v
}

func (v *ProjP3) FromP1xP1(p *ProjP1xP1) *ProjP3 {
	v.X.Mul(&p.X, &p.T)
	v.Y.Mul(&p.Y, &p.Z)
	v.Z.Mul(&p.Z, &p.T)
	v.T.Mul(&p.X, &p.Y)
	return v
}

func (v *ProjP3) FromP2(p *ProjP2) *ProjP3 {
	v.X.Mul(&p.X, &p.Z)
	v.Y.Mul(&p.Y, &p.Z)
	v.Z.Square(&p.Z)
	v.T.Mul(&p.X, &p.Y)
	return v
}

func (v *ProjCached) FromP3(p *ProjP3) *ProjCached {
	v.YplusX.Add(&p.Y, &p.X)
	v.YminusX.Sub(&p.Y, &p.X)
	v.Z.Set(&p.Z)
	v.T2d.Mul(&p.T, d2)
	return v
}

func (v *AffineCached) FromP3(p *ProjP3) *AffineCached {
	v.YplusX.Add(&p.Y, &p.X)
	v.YminusX.Sub(&p.Y, &p.X)
	v.T2d.Mul(&p.T, d2)

	var invZ radix51.FieldElement
	invZ.Invert(&p.Z)
	v.YplusX.Mul(&v.YplusX, &invZ)
	v.YminusX.Mul(&v.YminusX, &invZ)
	v.T2d.Mul(&v.T2d, &invZ)
	return v
}

// (Re)addition and subtraction.

func (v *ProjP3) Add(p, q *ProjP3) *ProjP3 {
	result := ProjP1xP1{}
	qCached := ProjCached{}
	qCached.FromP3(q)
	result.Add(p, &qCached)
	v.FromP1xP1(&result)
	return v
}

func (v *ProjP3) Sub(p, q *ProjP3) *ProjP3 {
	result := ProjP1xP1{}
	qCached := ProjCached{}
	qCached.FromP3(q)
	result.Sub(p, &qCached)
	v.FromP1xP1(&result)
	return v
}

func (v *ProjP1xP1) Add(p *ProjP3, q *ProjCached) *ProjP1xP1 {
	var YplusX, YminusX, PP, MM, TT2d, ZZ2 radix51.FieldElement

	YplusX.Add(&p.Y, &p.X)
	YminusX.Sub(&p.Y, &p.X)

	PP.Mul(&YplusX, &q.YplusX)
	MM.Mul(&YminusX, &q.YminusX)
	TT2d.Mul(&p.T, &q.T2d)
	ZZ2.Mul(&p.Z, &q.Z)

	ZZ2.Add(&ZZ2, &ZZ2)

	v.X.Sub(&PP, &MM)
	v.Y.Add(&PP, &MM)
	v.Z.Add(&ZZ2, &TT2d)
	v.T.Sub(&ZZ2, &TT2d)
	return v
}

func (v *ProjP1xP1) Sub(p *ProjP3, q *ProjCached) *ProjP1xP1 {
	var YplusX, YminusX, PP, MM, TT2d, ZZ2 radix51.FieldElement

	YplusX.Add(&p.Y, &p.X)
	YminusX.Sub(&p.Y, &p.X)

	PP.Mul(&YplusX, &q.YminusX) // flipped sign
	MM.Mul(&YminusX, &q.YplusX) // flipped sign
	TT2d.Mul(&p.T, &q.T2d)
	ZZ2.Mul(&p.Z, &q.Z)

	ZZ2.Add(&ZZ2, &ZZ2)

	v.X.Sub(&PP, &MM)
	v.Y.Add(&PP, &MM)
	v.Z.Sub(&ZZ2, &TT2d) // flipped sign
	v.T.Add(&ZZ2, &TT2d) // flipped sign
	return v
}

func (v *ProjP1xP1) AddAffine(p *ProjP3, q *AffineCached) *ProjP1xP1 {
	var YplusX, YminusX, PP, MM, TT2d, Z2 radix51.FieldElement

	YplusX.Add(&p.Y, &p.X)
	YminusX.Sub(&p.Y, &p.X)

	PP.Mul(&YplusX, &q.YplusX)
	MM.Mul(&YminusX, &q.YminusX)
	TT2d.Mul(&p.T, &q.T2d)

	Z2.Add(&p.Z, &p.Z)

	v.X.Sub(&PP, &MM)
	v.Y.Add(&PP, &MM)
	v.Z.Add(&Z2, &TT2d)
	v.T.Sub(&Z2, &TT2d)
	return v
}

func (v *ProjP1xP1) SubAffine(p *ProjP3, q *AffineCached) *ProjP1xP1 {
	var YplusX, YminusX, PP, MM, TT2d, Z2 radix51.FieldElement

	YplusX.Add(&p.Y, &p.X)
	YminusX.Sub(&p.Y, &p.X)

	PP.Mul(&YplusX, &q.YminusX) // flipped sign
	MM.Mul(&YminusX, &q.YplusX) // flipped sign
	TT2d.Mul(&p.T, &q.T2d)

	Z2.Add(&p.Z, &p.Z)

	v.X.Sub(&PP, &MM)
	v.Y.Add(&PP, &MM)
	v.Z.Sub(&Z2, &TT2d) // flipped sign
	v.T.Add(&Z2, &TT2d) // flipped sign
	return v
}

// Doubling.

func (v *ProjP1xP1) Double(p *ProjP2) *ProjP1xP1 {
	var XX, YY, ZZ2, XplusYsq radix51.FieldElement

	XX.Square(&p.X)
	YY.Square(&p.Y)
	ZZ2.Square(&p.Z)
	ZZ2.Add(&ZZ2, &ZZ2)
	XplusYsq.Add(&p.X, &p.Y)
	XplusYsq.Square(&XplusYsq)

	v.Y.Add(&YY, &XX)
	v.Z.Sub(&YY, &XX)

	v.X.Sub(&XplusYsq, &v.Y)
	v.T.Sub(&ZZ2, &v.Z)
	return v
}

// Negation.

func (v *ProjP3) Neg(p *ProjP3) *ProjP3 {
	v.X.Neg(&p.X)
	v.Y.Set(&p.Y)
	v.Z.Set(&p.Z)
	v.T.Neg(&p.T)
	return v
}

// by @ebfull
// https://github.com/dalek-cryptography/curve25519-dalek/pull/226/files
func (v *ProjP3) Equal(u *ProjP3) int {
	var t1, t2, t3, t4 radix51.FieldElement
	t1.Mul(&v.X, &u.Z)
	t2.Mul(&u.X, &v.Z)
	t3.Mul(&v.Y, &u.Z)
	t4.Mul(&u.Y, &v.Z)

	return t1.Equal(&t2) & t3.Equal(&t4)
}

// Constant-time operations

// Select sets v to a if cond == 1 and to b if cond == 0.
func (v *ProjCached) Select(a, b *ProjCached, cond int) *ProjCached {
	v.YplusX.Select(&a.YplusX, &b.YplusX, cond)
	v.YminusX.Select(&a.YminusX, &b.YminusX, cond)
	v.Z.Select(&a.Z, &b.Z, cond)
	v.T2d.Select(&a.T2d, &b.T2d, cond)
	return v
}

// Select sets v to a if cond == 1 and to b if cond == 0.
func (v *AffineCached) Select(a, b *AffineCached, cond int) *AffineCached {
	v.YplusX.Select(&a.YplusX, &b.YplusX, cond)
	v.YminusX.Select(&a.YminusX, &b.YminusX, cond)
	v.T2d.Select(&a.T2d, &b.T2d, cond)
	return v
}

// CondNeg negates v if cond == 1 and leaves it unchanged if cond == 0.
func (v *ProjCached) CondNeg(cond int) *ProjCached {
	radix51.CondSwap(&v.YplusX, &v.YminusX, cond)
	v.T2d.CondNeg(&v.T2d, cond)
	return v
}

// CondNeg negates v if cond == 1 and leaves it unchanged if cond == 0.
func (v *AffineCached) CondNeg(cond int) *AffineCached {
	radix51.CondSwap(&v.YplusX, &v.YminusX, cond)
	v.T2d.CondNeg(&v.T2d, cond)
	return v
}
