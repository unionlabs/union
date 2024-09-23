package fields_bn254

import (
	"math/big"

	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/internal/frontendtype"
)

type E6 struct {
	B0, B1, B2 E2
}

type Ext6 struct {
	*Ext2
}

func NewExt6(api frontend.API) *Ext6 {
	return &Ext6{Ext2: NewExt2(api)}
}

func (e Ext6) One() *E6 {
	z0 := e.Ext2.One()
	z1 := e.Ext2.Zero()
	z2 := e.Ext2.Zero()
	return &E6{
		B0: *z0,
		B1: *z1,
		B2: *z2,
	}
}

func (e Ext6) Zero() *E6 {
	z0 := e.Ext2.Zero()
	z1 := e.Ext2.Zero()
	z2 := e.Ext2.Zero()
	return &E6{
		B0: *z0,
		B1: *z1,
		B2: *z2,
	}
}

func (e Ext6) IsZero(z *E6) frontend.Variable {
	b0 := e.Ext2.IsZero(&z.B0)
	b1 := e.Ext2.IsZero(&z.B1)
	b2 := e.Ext2.IsZero(&z.B2)
	return e.api.And(e.api.And(b0, b1), b2)
}

func (e Ext6) Add(x, y *E6) *E6 {
	z0 := e.Ext2.Add(&x.B0, &y.B0)
	z1 := e.Ext2.Add(&x.B1, &y.B1)
	z2 := e.Ext2.Add(&x.B2, &y.B2)
	return &E6{
		B0: *z0,
		B1: *z1,
		B2: *z2,
	}
}

func (e Ext6) Neg(x *E6) *E6 {
	z0 := e.Ext2.Neg(&x.B0)
	z1 := e.Ext2.Neg(&x.B1)
	z2 := e.Ext2.Neg(&x.B2)
	return &E6{
		B0: *z0,
		B1: *z1,
		B2: *z2,
	}
}

func (e Ext6) Sub(x, y *E6) *E6 {
	z0 := e.Ext2.Sub(&x.B0, &y.B0)
	z1 := e.Ext2.Sub(&x.B1, &y.B1)
	z2 := e.Ext2.Sub(&x.B2, &y.B2)
	return &E6{
		B0: *z0,
		B1: *z1,
		B2: *z2,
	}
}

// Mul multiplies two E6 elmts
func (e Ext6) Mul(x, y *E6) *E6 {
	if ft, ok := e.api.(frontendtype.FrontendTyper); ok {
		switch ft.FrontendType() {
		case frontendtype.R1CS:
			return e.mulToom3OverKaratsuba(x, y)
		case frontendtype.SCS:
			return e.mulKaratsubaOverKaratsuba(x, y)
		}
	}
	return e.mulKaratsubaOverKaratsuba(x, y)
}

func (e Ext6) mulToom3OverKaratsuba(x, y *E6) *E6 {
	// Toom-Cook-3x over Karatsuba:
	// We start by computing five interpolation points – these are evaluations of
	// the product x(u)y(u) with u ∈ {0, ±1, 2, ∞}:
	//
	// v0 = x(0)y(0) = x.A0 * y.A0
	// v1 = x(1)y(1) = (x.A0 + x.A1 + x.A2)(y.A0 + y.A1 + y.A2)
	// v2 = x(−1)y(−1) = (x.A0 − x.A1 + x.A2)(y.A0 − y.A1 + y.A2)
	// v3 = x(2)y(2) = (x.A0 + 2x.A1 + 4x.A2)(y.A0 + 2y.A1 + 4y.A2)
	// v4 = x(∞)y(∞) = x.A2 * y.A2

	v0 := e.Ext2.Mul(&x.B0, &y.B0)

	t1 := e.Ext2.Add(&x.B0, &x.B2)
	t2 := e.Ext2.Add(&y.B0, &y.B2)
	t3 := e.Ext2.Add(t2, &y.B1)
	v1 := e.Ext2.Add(t1, &x.B1)
	v1 = e.Ext2.Mul(v1, t3)

	t3 = e.Ext2.Sub(t2, &y.B1)
	v2 := e.Ext2.Sub(t1, &x.B1)
	v2 = e.Ext2.Mul(v2, t3)

	t1 = e.Ext2.MulByConstElement(&x.B1, big.NewInt(2))
	t2 = e.Ext2.MulByConstElement(&x.B2, big.NewInt(4))
	v3 := e.Ext2.Add(t1, t2)
	v3 = e.Ext2.Add(v3, &x.B0)
	t1 = e.Ext2.MulByConstElement(&y.B1, big.NewInt(2))
	t2 = e.Ext2.MulByConstElement(&y.B2, big.NewInt(4))
	t3 = e.Ext2.Add(t1, t2)
	t3 = e.Ext2.Add(t3, &y.B0)
	v3 = e.Ext2.Mul(v3, t3)

	v4 := e.Ext2.Mul(&x.B2, &y.B2)

	// Then the interpolation is performed as:
	//
	// a0 = v0 + β((1/2)v0 − (1/2)v1 − (1/6)v2 + (1/6)v3 − 2v4)
	// a1 = −(1/2)v0 + v1 − (1/3)v2 − (1/6)v3 + 2v4 + βv4
	// a2 = −v0 + (1/2)v1 + (1/2)v2 − v4
	//
	// where β is the cubic non-residue.
	//
	// In-circuit, we compute 6*x*y as
	// c0 = 6v0 + β(3v0 − 3v1 − v2 + v3 − 12v4)
	// a1 = -(3v0 + 2v2 + v3) + 6(v1 + 2v4 + βv4)
	// a2 = 3(v1 + v2 - 2(v0 + v4))
	//
	// and then divide a0, a1 and a2 by 6 using a hint.

	a0 := e.Ext2.MulByConstElement(v0, big.NewInt(6))
	t1 = e.Ext2.Sub(v0, v1)
	t1 = e.Ext2.MulByConstElement(t1, big.NewInt(3))
	t1 = e.Ext2.Sub(t1, v2)
	t1 = e.Ext2.Add(t1, v3)
	t2 = e.Ext2.MulByConstElement(v4, big.NewInt(12))
	t1 = e.Ext2.Sub(t1, t2)
	t1 = e.Ext2.MulByNonResidue(t1)
	a0 = e.Ext2.Add(a0, t1)

	a1 := e.Ext2.MulByConstElement(v0, big.NewInt(3))
	t1 = e.Ext2.MulByConstElement(v2, big.NewInt(2))
	a1 = e.Ext2.Add(a1, t1)
	a1 = e.Ext2.Add(a1, v3)
	t1 = e.Ext2.MulByConstElement(v4, big.NewInt(2))
	t1 = e.Ext2.Add(t1, v1)
	t2 = e.Ext2.MulByNonResidue(v4)
	t1 = e.Ext2.Add(t1, t2)
	t1 = e.Ext2.MulByConstElement(t1, big.NewInt(6))
	a1 = e.Ext2.Sub(t1, a1)

	a2 := e.Ext2.Add(v1, v2)
	a2 = e.Ext2.MulByConstElement(a2, big.NewInt(3))
	t1 = e.Ext2.Add(v0, v4)
	t1 = e.Ext2.MulByConstElement(t1, big.NewInt(6))
	a2 = e.Ext2.Sub(a2, t1)

	res := e.divE6By6([6]*baseEl{&a0.A0, &a0.A1, &a1.A0, &a1.A1, &a2.A0, &a2.A1})
	return &E6{
		B0: E2{
			A0: *res[0],
			A1: *res[1],
		},
		B1: E2{
			A0: *res[2],
			A1: *res[3],
		},
		B2: E2{
			A0: *res[4],
			A1: *res[5],
		},
	}
}

func (e Ext6) mulKaratsubaOverKaratsuba(x, y *E6) *E6 {
	// Karatsuba over Karatsuba:
	// Algorithm 13 from https://eprint.iacr.org/2010/354.pdf
	t0 := e.Ext2.Mul(&x.B0, &y.B0)
	t1 := e.Ext2.Mul(&x.B1, &y.B1)
	t2 := e.Ext2.Mul(&x.B2, &y.B2)
	c0 := e.Ext2.Add(&x.B1, &x.B2)
	tmp := e.Ext2.Add(&y.B1, &y.B2)
	c0 = e.Ext2.Mul(c0, tmp)
	tmp = e.Ext2.Add(t2, t1)
	c0 = e.Ext2.Sub(c0, tmp)
	c0 = e.Ext2.MulByNonResidue(c0)
	c0 = e.Ext2.Add(c0, t0)
	c1 := e.Ext2.Add(&x.B0, &x.B1)
	tmp = e.Ext2.Add(&y.B0, &y.B1)
	c1 = e.Ext2.Mul(c1, tmp)
	tmp = e.Ext2.Add(t0, t1)
	c1 = e.Ext2.Sub(c1, tmp)
	tmp = e.Ext2.MulByNonResidue(t2)
	c1 = e.Ext2.Add(c1, tmp)
	tmp = e.Ext2.Add(&x.B0, &x.B2)
	c2 := e.Ext2.Add(&y.B0, &y.B2)
	c2 = e.Ext2.Mul(c2, tmp)
	tmp = e.Ext2.Add(t0, t2)
	c2 = e.Ext2.Sub(c2, tmp)
	c2 = e.Ext2.Add(c2, t1)
	return &E6{
		B0: *c0,
		B1: *c1,
		B2: *c2,
	}
}

func (e Ext6) Double(x *E6) *E6 {
	z0 := e.Ext2.Double(&x.B0)
	z1 := e.Ext2.Double(&x.B1)
	z2 := e.Ext2.Double(&x.B2)
	return &E6{
		B0: *z0,
		B1: *z1,
		B2: *z2,
	}
}

func (e Ext6) Square(x *E6) *E6 {
	c4 := e.Ext2.Mul(&x.B0, &x.B1)
	c4 = e.Ext2.Double(c4)
	c5 := e.Ext2.Square(&x.B2)
	c1 := e.Ext2.MulByNonResidue(c5)
	c1 = e.Ext2.Add(c1, c4)
	c2 := e.Ext2.Sub(c4, c5)
	c3 := e.Ext2.Square(&x.B0)
	c4 = e.Ext2.Sub(&x.B0, &x.B1)
	c4 = e.Ext2.Add(c4, &x.B2)
	c5 = e.Ext2.Mul(&x.B1, &x.B2)
	c5 = e.Ext2.Double(c5)
	c4 = e.Ext2.Square(c4)
	c0 := e.Ext2.MulByNonResidue(c5)
	c0 = e.Ext2.Add(c0, c3)
	z2 := e.Ext2.Add(c2, c4)
	z2 = e.Ext2.Add(z2, c5)
	z2 = e.Ext2.Sub(z2, c3)
	z0 := c0
	z1 := c1
	return &E6{
		B0: *z0,
		B1: *z1,
		B2: *z2,
	}
}

func (e Ext6) MulByConstE2(x *E6, y *E2) *E6 {
	z0 := e.Ext2.Mul(&x.B0, y)
	z1 := e.Ext2.Mul(&x.B1, y)
	z2 := e.Ext2.Mul(&x.B2, y)
	return &E6{
		B0: *z0,
		B1: *z1,
		B2: *z2,
	}
}

func (e Ext6) MulByE2(x *E6, y *E2) *E6 {
	z0 := e.Ext2.Mul(&x.B0, y)
	z1 := e.Ext2.Mul(&x.B1, y)
	z2 := e.Ext2.Mul(&x.B2, y)
	return &E6{
		B0: *z0,
		B1: *z1,
		B2: *z2,
	}
}

// MulBy0 multiplies z by an E6 sparse element of the form
//
//	E6{
//		B0: c0,
//		B1: 0,
//		B2: 0,
//	}
func (e Ext6) MulBy0(z *E6, c0 *E2) *E6 {
	a := e.Ext2.Mul(&z.B0, c0)
	tmp := e.Ext2.Add(&z.B0, &z.B2)
	t2 := e.Ext2.Mul(c0, tmp)
	t2 = e.Ext2.Sub(t2, a)
	tmp = e.Ext2.Add(&z.B0, &z.B1)
	t1 := e.Ext2.Mul(c0, tmp)
	t1 = e.Ext2.Sub(t1, a)
	return &E6{
		B0: *a,
		B1: *t1,
		B2: *t2,
	}
}

// MulBy01 multiplies z by an E6 sparse element of the form
//
//	E6{
//		B0: c0,
//		B1: c1,
//		B2: 0,
//	}
func (e Ext6) MulBy01(z *E6, c0, c1 *E2) *E6 {
	a := e.Ext2.Mul(&z.B0, c0)
	b := e.Ext2.Mul(&z.B1, c1)
	tmp := e.Ext2.Add(&z.B1, &z.B2)
	t0 := e.Ext2.Mul(c1, tmp)
	t0 = e.Ext2.Sub(t0, b)
	t0 = e.Ext2.MulByNonResidue(t0)
	t0 = e.Ext2.Add(t0, a)
	// for t2, schoolbook is faster than karatsuba
	// c2 = a0b2 + a1b1 + a2b0,
	// c2 = a2b0 + b ∵ b2 = 0, b = a1b1
	t2 := e.Ext2.Mul(&z.B2, c0)
	t2 = e.Ext2.Add(t2, b)
	t1 := e.Ext2.Add(c0, c1)
	tmp = e.Ext2.Add(&z.B0, &z.B1)
	t1 = e.Ext2.Mul(t1, tmp)
	tmp = e.Ext2.Add(a, b)
	t1 = e.Ext2.Sub(t1, tmp)
	return &E6{
		B0: *t0,
		B1: *t1,
		B2: *t2,
	}
}

// Mul01By01 multiplies two E6 sparse element of the form:
//
//	E6{
//		B0: c0,
//		B1: c1,
//		B2: 0,
//	}
//
// and
//
//	E6{
//		B0: d0,
//		B1: d1,
//		B2: 0,
//	}
func (e Ext6) Mul01By01(c0, c1, d0, d1 *E2) *E6 {
	a := e.Ext2.Mul(d0, c0)
	b := e.Ext2.Mul(d1, c1)
	t1 := e.Ext2.Add(c0, c1)
	tmp := e.Ext2.Add(d0, d1)
	t1 = e.Ext2.Mul(t1, tmp)
	tmp = e.Ext2.Add(a, b)
	t1 = e.Ext2.Sub(t1, tmp)
	return &E6{
		B0: *a,
		B1: *t1,
		B2: *b,
	}
}

func (e Ext6) MulByNonResidue(x *E6) *E6 {
	z2, z1, z0 := &x.B1, &x.B0, &x.B2
	z0 = e.Ext2.MulByNonResidue(z0)
	return &E6{
		B0: *z0,
		B1: *z1,
		B2: *z2,
	}
}

func (e Ext6) FrobeniusSquare(x *E6) *E6 {
	z01 := e.Ext2.MulByNonResidue2Power2(&x.B1)
	z02 := e.Ext2.MulByNonResidue2Power4(&x.B2)
	return &E6{B0: x.B0, B1: *z01, B2: *z02}
}

func (e Ext6) AssertIsEqual(x, y *E6) {
	e.Ext2.AssertIsEqual(&x.B0, &y.B0)
	e.Ext2.AssertIsEqual(&x.B1, &y.B1)
	e.Ext2.AssertIsEqual(&x.B2, &y.B2)
}

func FromE6(y *bn254.E6) E6 {
	return E6{
		B0: FromE2(&y.B0),
		B1: FromE2(&y.B1),
		B2: FromE2(&y.B2),
	}

}

func (e Ext6) Inverse(x *E6) *E6 {
	res, err := e.fp.NewHint(inverseE6Hint, 6, &x.B0.A0, &x.B0.A1, &x.B1.A0, &x.B1.A1, &x.B2.A0, &x.B2.A1)
	if err != nil {
		// err is non-nil only for invalid number of inputs
		panic(err)
	}

	inv := E6{
		B0: E2{A0: *res[0], A1: *res[1]},
		B1: E2{A0: *res[2], A1: *res[3]},
		B2: E2{A0: *res[4], A1: *res[5]},
	}

	one := e.One()

	// 1 == inv * x
	_one := e.Mul(&inv, x)
	e.AssertIsEqual(one, _one)

	return &inv

}

func (e Ext6) DivUnchecked(x, y *E6) *E6 {
	res, err := e.fp.NewHint(divE6Hint, 6, &x.B0.A0, &x.B0.A1, &x.B1.A0, &x.B1.A1, &x.B2.A0, &x.B2.A1, &y.B0.A0, &y.B0.A1, &y.B1.A0, &y.B1.A1, &y.B2.A0, &y.B2.A1)
	if err != nil {
		// err is non-nil only for invalid number of inputs
		panic(err)
	}

	div := E6{
		B0: E2{A0: *res[0], A1: *res[1]},
		B1: E2{A0: *res[2], A1: *res[3]},
		B2: E2{A0: *res[4], A1: *res[5]},
	}

	// x == div * y
	_x := e.Mul(&div, y)
	e.AssertIsEqual(x, _x)

	return &div
}

func (e Ext6) divE6By6(x [6]*baseEl) [6]*baseEl {
	res, err := e.fp.NewHint(divE6By6Hint, 6, x[0], x[1], x[2], x[3], x[4], x[5])
	if err != nil {
		// err is non-nil only for invalid number of inputs
		panic(err)
	}

	y0 := *res[0]
	y1 := *res[1]
	y2 := *res[2]
	y3 := *res[3]
	y4 := *res[4]
	y5 := *res[5]

	// xi == 6 * yi
	x0 := e.fp.MulConst(&y0, big.NewInt(6))
	x1 := e.fp.MulConst(&y1, big.NewInt(6))
	x2 := e.fp.MulConst(&y2, big.NewInt(6))
	x3 := e.fp.MulConst(&y3, big.NewInt(6))
	x4 := e.fp.MulConst(&y4, big.NewInt(6))
	x5 := e.fp.MulConst(&y5, big.NewInt(6))
	e.fp.AssertIsEqual(x[0], x0)
	e.fp.AssertIsEqual(x[1], x1)
	e.fp.AssertIsEqual(x[2], x2)
	e.fp.AssertIsEqual(x[3], x3)
	e.fp.AssertIsEqual(x[4], x4)
	e.fp.AssertIsEqual(x[5], x5)

	return [6]*baseEl{&y0, &y1, &y2, &y3, &y4, &y5}
}

func (e Ext6) Select(selector frontend.Variable, z1, z0 *E6) *E6 {
	b0 := e.Ext2.Select(selector, &z1.B0, &z0.B0)
	b1 := e.Ext2.Select(selector, &z1.B1, &z0.B1)
	b2 := e.Ext2.Select(selector, &z1.B2, &z0.B2)
	return &E6{B0: *b0, B1: *b1, B2: *b2}
}

func (e Ext6) Lookup2(s1, s2 frontend.Variable, a, b, c, d *E6) *E6 {
	b0 := e.Ext2.Lookup2(s1, s2, &a.B0, &b.B0, &c.B0, &d.B0)
	b1 := e.Ext2.Lookup2(s1, s2, &a.B1, &b.B1, &c.B1, &d.B1)
	b2 := e.Ext2.Lookup2(s1, s2, &a.B2, &b.B2, &c.B2, &d.B2)
	return &E6{B0: *b0, B1: *b1, B2: *b2}
}
