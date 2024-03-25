package fields_bn254

import (
	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark/frontend"
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

func (e Ext6) Mul(x, y *E6) *E6 {
	t0 := e.Ext2.Mul(&x.B0, &y.B0)
	t1 := e.Ext2.Mul(&x.B1, &y.B1)
	t2 := e.Ext2.Mul(&x.B2, &y.B2)
	c0 := e.Ext2.Add(&x.B1, &x.B2)
	tmp := e.Ext2.Add(&y.B1, &y.B2)
	c0 = e.Ext2.Mul(c0, tmp)
	c0 = e.Ext2.Sub(c0, t1)
	c0 = e.Ext2.Sub(c0, t2)
	c0 = e.Ext2.MulByNonResidue(c0)
	c0 = e.Ext2.Add(c0, t0)
	c1 := e.Ext2.Add(&x.B0, &x.B1)
	tmp = e.Ext2.Add(&y.B0, &y.B1)
	c1 = e.Ext2.Mul(c1, tmp)
	c1 = e.Ext2.Sub(c1, t0)
	c1 = e.Ext2.Sub(c1, t1)
	tmp = e.Ext2.MulByNonResidue(t2)
	c1 = e.Ext2.Add(c1, tmp)
	tmp = e.Ext2.Add(&x.B0, &x.B2)
	c2 := e.Ext2.Add(&y.B0, &y.B2)
	c2 = e.Ext2.Mul(c2, tmp)
	c2 = e.Ext2.Sub(c2, t0)
	c2 = e.Ext2.Sub(c2, t2)
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
	t1 = e.Ext2.Sub(t1, a)
	t1 = e.Ext2.Sub(t1, b)
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
	t1 = e.Ext2.Sub(t1, a)
	t1 = e.Ext2.Sub(t1, b)
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
