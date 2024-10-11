package bn256

import (
	"math/big"
)

// twistPoint implements the elliptic curve y²=x³+3/ξ over GF(p²). Points are
// kept in Jacobian form and t=z² when valid. The group G₂ is the set of
// n-torsion points of this curve over GF(p²) (where n = Order)
type twistPoint struct {
	x, y, z, t gfP2
}

var twistB = &gfP2{
	gfP{0x38e7ecccd1dcff67, 0x65f0b37d93ce0d3e, 0xd749d0dd22ac00aa, 0x0141b9ce4a688d4d},
	gfP{0x3bf938e377b802a8, 0x020b1b273633535d, 0x26b7edf049755260, 0x2514c6324384a86d},
}

// twistGen is the generator of group G₂.
var twistGen = &twistPoint{
	gfP2{
		gfP{0xafb4737da84c6140, 0x6043dd5a5802d8c4, 0x09e950fc52a02f86, 0x14fef0833aea7b6b},
		gfP{0x8e83b5d102bc2026, 0xdceb1935497b0172, 0xfbb8264797811adf, 0x19573841af96503b},
	},
	gfP2{
		gfP{0x64095b56c71856ee, 0xdc57f922327d3cbb, 0x55f935be33351076, 0x0da4a0e693fd6482},
		gfP{0x619dfa9d886be9f6, 0xfe7fd297f59e9b78, 0xff9e1a62231b7dfe, 0x28fd7eebae9e4206},
	},
	gfP2{*newGFp(0), *newGFp(1)},
	gfP2{*newGFp(0), *newGFp(1)},
}

func (c *twistPoint) String() string {
	c.MakeAffine()
	x, y := gfP2Decode(&c.x), gfP2Decode(&c.y)
	return "(" + x.String() + ", " + y.String() + ")"
}

func (c *twistPoint) Set(a *twistPoint) {
	c.x.Set(&a.x)
	c.y.Set(&a.y)
	c.z.Set(&a.z)
	c.t.Set(&a.t)
}

// IsOnCurve returns true iff c is on the curve.
func (c *twistPoint) IsOnCurve() bool {
	c.MakeAffine()
	if c.IsInfinity() {
		return true
	}

	y2, x3 := &gfP2{}, &gfP2{}
	y2.Square(&c.y)
	x3.Square(&c.x).Mul(x3, &c.x).Add(x3, twistB)

	if *y2 != *x3 {
		return false
	}
	cneg := &twistPoint{}
	cneg.Mul(c, Order)
	return cneg.z.IsZero()
}

func (c *twistPoint) SetInfinity() {
	c.x.SetZero()
	c.y.SetOne()
	c.z.SetZero()
	c.t.SetZero()
}

func (c *twistPoint) IsInfinity() bool {
	return c.z.IsZero()
}

func (c *twistPoint) Add(a, b *twistPoint) {
	// For additional comments, see the same function in curve.go.

	if a.IsInfinity() {
		c.Set(b)
		return
	}
	if b.IsInfinity() {
		c.Set(a)
		return
	}

	// See http://hyperelliptic.org/EFD/g1p/auto-code/shortw/jacobian-0/addition/add-2007-bl.op3
	z12 := (&gfP2{}).Square(&a.z)
	z22 := (&gfP2{}).Square(&b.z)
	u1 := (&gfP2{}).Mul(&a.x, z22)
	u2 := (&gfP2{}).Mul(&b.x, z12)

	t := (&gfP2{}).Mul(&b.z, z22)
	s1 := (&gfP2{}).Mul(&a.y, t)

	t.Mul(&a.z, z12)
	s2 := (&gfP2{}).Mul(&b.y, t)

	h := (&gfP2{}).Sub(u2, u1)
	xEqual := h.IsZero()

	t.Add(h, h)
	i := (&gfP2{}).Square(t)
	j := (&gfP2{}).Mul(h, i)

	t.Sub(s2, s1)
	yEqual := t.IsZero()
	if xEqual && yEqual {
		c.Double(a)
		return
	}
	r := (&gfP2{}).Add(t, t)

	v := (&gfP2{}).Mul(u1, i)

	t4 := (&gfP2{}).Square(r)
	t.Add(v, v)
	t6 := (&gfP2{}).Sub(t4, j)
	c.x.Sub(t6, t)

	t.Sub(v, &c.x) // t7
	t4.Mul(s1, j)  // t8
	t6.Add(t4, t4) // t9
	t4.Mul(r, t)   // t10
	c.y.Sub(t4, t6)

	t.Add(&a.z, &b.z) // t11
	t4.Square(t)      // t12
	t.Sub(t4, z12)    // t13
	t4.Sub(t, z22)    // t14
	c.z.Mul(t4, h)
}

func (c *twistPoint) Double(a *twistPoint) {
	// See http://hyperelliptic.org/EFD/g1p/auto-code/shortw/jacobian-0/doubling/dbl-2009-l.op3
	A := (&gfP2{}).Square(&a.x)
	B := (&gfP2{}).Square(&a.y)
	C := (&gfP2{}).Square(B)

	t := (&gfP2{}).Add(&a.x, B)
	t2 := (&gfP2{}).Square(t)
	t.Sub(t2, A)
	t2.Sub(t, C)
	d := (&gfP2{}).Add(t2, t2)
	t.Add(A, A)
	e := (&gfP2{}).Add(t, A)
	f := (&gfP2{}).Square(e)

	t.Add(d, d)
	c.x.Sub(f, t)

	c.z.Mul(&a.y, &a.z)
	c.z.Add(&c.z, &c.z)

	t.Add(C, C)
	t2.Add(t, t)
	t.Add(t2, t2)
	c.y.Sub(d, &c.x)
	t2.Mul(e, &c.y)
	c.y.Sub(t2, t)
}

func (c *twistPoint) Mul(a *twistPoint, scalar *big.Int) {
	sum, t := &twistPoint{}, &twistPoint{}

	for i := scalar.BitLen(); i >= 0; i-- {
		t.Double(sum)
		if scalar.Bit(i) != 0 {
			sum.Add(t, a)
		} else {
			sum.Set(t)
		}
	}

	c.Set(sum)
}

func (c *twistPoint) MakeAffine() {
	if c.z.IsOne() {
		return
	} else if c.z.IsZero() {
		c.x.SetZero()
		c.y.SetOne()
		c.t.SetZero()
		return
	}

	zInv := (&gfP2{}).Invert(&c.z)
	t := (&gfP2{}).Mul(&c.y, zInv)
	zInv2 := (&gfP2{}).Square(zInv)
	c.y.Mul(t, zInv2)
	t.Mul(&c.x, zInv2)
	c.x.Set(t)
	c.z.SetOne()
	c.t.SetOne()
}

func (c *twistPoint) Neg(a *twistPoint) {
	c.x.Set(&a.x)
	c.y.Neg(&a.y)
	c.z.Set(&a.z)
	c.t.SetZero()
}
