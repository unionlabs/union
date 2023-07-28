package fields_bn254

import (
	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark/frontend"
)

type E12 struct {
	C0, C1 E6
}

type Ext12 struct {
	*Ext6
}

func NewExt12(api frontend.API) *Ext12 {
	return &Ext12{Ext6: NewExt6(api)}
}

func (e Ext12) Add(x, y *E12) *E12 {
	z0 := e.Ext6.Add(&x.C0, &y.C0)
	z1 := e.Ext6.Add(&x.C1, &y.C1)
	return &E12{
		C0: *z0,
		C1: *z1,
	}
}

func (e Ext12) Sub(x, y *E12) *E12 {
	z0 := e.Ext6.Sub(&x.C0, &y.C0)
	z1 := e.Ext6.Sub(&x.C1, &y.C1)
	return &E12{
		C0: *z0,
		C1: *z1,
	}
}

func (e Ext12) Conjugate(x *E12) *E12 {
	z1 := e.Ext6.Neg(&x.C1)
	return &E12{
		C0: x.C0,
		C1: *z1,
	}
}

func (e Ext12) Mul(x, y *E12) *E12 {
	a := e.Ext6.Add(&x.C0, &x.C1)
	b := e.Ext6.Add(&y.C0, &y.C1)
	a = e.Ext6.Mul(a, b)
	b = e.Ext6.Mul(&x.C0, &y.C0)
	c := e.Ext6.Mul(&x.C1, &y.C1)
	z1 := e.Ext6.Sub(a, b)
	z1 = e.Ext6.Sub(z1, c)
	z0 := e.Ext6.MulByNonResidue(c)
	z0 = e.Ext6.Add(z0, b)
	return &E12{
		C0: *z0,
		C1: *z1,
	}
}

func (e Ext12) Zero() *E12 {
	zero := e.fp.Zero()
	return &E12{
		C0: E6{
			B0: E2{A0: *zero, A1: *zero},
			B1: E2{A0: *zero, A1: *zero},
			B2: E2{A0: *zero, A1: *zero},
		},
		C1: E6{
			B0: E2{A0: *zero, A1: *zero},
			B1: E2{A0: *zero, A1: *zero},
			B2: E2{A0: *zero, A1: *zero},
		},
	}
}

func (e Ext12) One() *E12 {
	z000 := e.fp.One()
	zero := e.fp.Zero()
	return &E12{
		C0: E6{
			B0: E2{A0: *z000, A1: *zero},
			B1: E2{A0: *zero, A1: *zero},
			B2: E2{A0: *zero, A1: *zero},
		},
		C1: E6{
			B0: E2{A0: *zero, A1: *zero},
			B1: E2{A0: *zero, A1: *zero},
			B2: E2{A0: *zero, A1: *zero},
		},
	}
}

func (e Ext12) IsZero(z *E12) frontend.Variable {
	c0 := e.Ext6.IsZero(&z.C0)
	c1 := e.Ext6.IsZero(&z.C1)
	return e.api.And(c0, c1)
}

func (e Ext12) Square(x *E12) *E12 {
	c0 := e.Ext6.Sub(&x.C0, &x.C1)
	c3 := e.Ext6.MulByNonResidue(&x.C1)
	c3 = e.Ext6.Neg(c3)
	c3 = e.Ext6.Add(&x.C0, c3)
	c2 := e.Ext6.Mul(&x.C0, &x.C1)
	c0 = e.Ext6.Mul(c0, c3)
	c0 = e.Ext6.Add(c0, c2)
	z1 := e.Ext6.Double(c2)
	c2 = e.Ext6.MulByNonResidue(c2)
	z0 := e.Ext6.Add(c0, c2)
	return &E12{
		C0: *z0,
		C1: *z1,
	}
}

func (e Ext12) AssertIsEqual(x, y *E12) {
	e.Ext6.AssertIsEqual(&x.C0, &y.C0)
	e.Ext6.AssertIsEqual(&x.C1, &y.C1)
}

func FromE12(y *bn254.E12) E12 {
	return E12{
		C0: FromE6(&y.C0),
		C1: FromE6(&y.C1),
	}

}

func (e Ext12) Inverse(x *E12) *E12 {
	res, err := e.fp.NewHint(inverseE12Hint, 12, &x.C0.B0.A0, &x.C0.B0.A1, &x.C0.B1.A0, &x.C0.B1.A1, &x.C0.B2.A0, &x.C0.B2.A1, &x.C1.B0.A0, &x.C1.B0.A1, &x.C1.B1.A0, &x.C1.B1.A1, &x.C1.B2.A0, &x.C1.B2.A1)
	if err != nil {
		// err is non-nil only for invalid number of inputs
		panic(err)
	}

	inv := E12{
		C0: E6{
			B0: E2{A0: *res[0], A1: *res[1]},
			B1: E2{A0: *res[2], A1: *res[3]},
			B2: E2{A0: *res[4], A1: *res[5]},
		},
		C1: E6{
			B0: E2{A0: *res[6], A1: *res[7]},
			B1: E2{A0: *res[8], A1: *res[9]},
			B2: E2{A0: *res[10], A1: *res[11]},
		},
	}

	one := e.One()

	// 1 == inv * x
	_one := e.Mul(&inv, x)
	e.AssertIsEqual(one, _one)

	return &inv

}

func (e Ext12) DivUnchecked(x, y *E12) *E12 {
	res, err := e.fp.NewHint(divE12Hint, 12, &x.C0.B0.A0, &x.C0.B0.A1, &x.C0.B1.A0, &x.C0.B1.A1, &x.C0.B2.A0, &x.C0.B2.A1, &x.C1.B0.A0, &x.C1.B0.A1, &x.C1.B1.A0, &x.C1.B1.A1, &x.C1.B2.A0, &x.C1.B2.A1, &y.C0.B0.A0, &y.C0.B0.A1, &y.C0.B1.A0, &y.C0.B1.A1, &y.C0.B2.A0, &y.C0.B2.A1, &y.C1.B0.A0, &y.C1.B0.A1, &y.C1.B1.A0, &y.C1.B1.A1, &y.C1.B2.A0, &y.C1.B2.A1)

	if err != nil {
		// err is non-nil only for invalid number of inputs
		panic(err)
	}

	div := E12{
		C0: E6{
			B0: E2{A0: *res[0], A1: *res[1]},
			B1: E2{A0: *res[2], A1: *res[3]},
			B2: E2{A0: *res[4], A1: *res[5]},
		},
		C1: E6{
			B0: E2{A0: *res[6], A1: *res[7]},
			B1: E2{A0: *res[8], A1: *res[9]},
			B2: E2{A0: *res[10], A1: *res[11]},
		},
	}

	// x == div * y
	_x := e.Mul(&div, y)
	e.AssertIsEqual(x, _x)

	return &div
}

func (e Ext12) Select(selector frontend.Variable, z1, z0 *E12) *E12 {
	c0 := e.Ext6.Select(selector, &z1.C0, &z0.C0)
	c1 := e.Ext6.Select(selector, &z1.C1, &z0.C1)
	return &E12{C0: *c0, C1: *c1}
}

func (e Ext12) Lookup2(s1, s2 frontend.Variable, a, b, c, d *E12) *E12 {
	c0 := e.Ext6.Lookup2(s1, s2, &a.C0, &b.C0, &c.C0, &d.C0)
	c1 := e.Ext6.Lookup2(s1, s2, &a.C1, &b.C1, &c.C1, &d.C1)
	return &E12{C0: *c0, C1: *c1}
}
