package bn256

import (
	"math/big"
)

// curvePoint implements the elliptic curve y²=x³+3. Points are kept in Jacobian
// form and t=z² when valid. G₁ is the set of points of this curve on GF(p).
type curvePoint struct {
	x, y, z, t gfP
}

var curveB = newGFp(3)

// curveGen is the generator of G₁.
var curveGen = &curvePoint{
	x: *newGFp(1),
	y: *newGFp(2),
	z: *newGFp(1),
	t: *newGFp(1),
}

func (c *curvePoint) String() string {
	c.MakeAffine()
	x, y := &gfP{}, &gfP{}
	montDecode(x, &c.x)
	montDecode(y, &c.y)
	return "(" + x.String() + ", " + y.String() + ")"
}

func (c *curvePoint) Set(a *curvePoint) {
	c.x.Set(&a.x)
	c.y.Set(&a.y)
	c.z.Set(&a.z)
	c.t.Set(&a.t)
}

// IsOnCurve returns true iff c is on the curve.
func (c *curvePoint) IsOnCurve() bool {
	c.MakeAffine()
	if c.IsInfinity() {
		return true
	}

	y2, x3 := &gfP{}, &gfP{}
	gfpMul(y2, &c.y, &c.y)
	gfpMul(x3, &c.x, &c.x)
	gfpMul(x3, x3, &c.x)
	gfpAdd(x3, x3, curveB)

	return *y2 == *x3
}

func (c *curvePoint) SetInfinity() {
	c.x = gfP{0}
	c.y = *newGFp(1)
	c.z = gfP{0}
	c.t = gfP{0}
}

func (c *curvePoint) IsInfinity() bool {
	return c.z == gfP{0}
}

func (c *curvePoint) Add(a, b *curvePoint) {
	if a.IsInfinity() {
		c.Set(b)
		return
	}
	if b.IsInfinity() {
		c.Set(a)
		return
	}

	// See http://hyperelliptic.org/EFD/g1p/auto-code/shortw/jacobian-0/addition/add-2007-bl.op3

	// Normalize the points by replacing a = [x1:y1:z1] and b = [x2:y2:z2]
	// by [u1:s1:z1·z2] and [u2:s2:z1·z2]
	// where u1 = x1·z2², s1 = y1·z2³ and u1 = x2·z1², s2 = y2·z1³
	z12, z22 := &gfP{}, &gfP{}
	gfpMul(z12, &a.z, &a.z)
	gfpMul(z22, &b.z, &b.z)

	u1, u2 := &gfP{}, &gfP{}
	gfpMul(u1, &a.x, z22)
	gfpMul(u2, &b.x, z12)

	t, s1 := &gfP{}, &gfP{}
	gfpMul(t, &b.z, z22)
	gfpMul(s1, &a.y, t)

	s2 := &gfP{}
	gfpMul(t, &a.z, z12)
	gfpMul(s2, &b.y, t)

	// Compute x = (2h)²(s²-u1-u2)
	// where s = (s2-s1)/(u2-u1) is the slope of the line through
	// (u1,s1) and (u2,s2). The extra factor 2h = 2(u2-u1) comes from the value of z below.
	// This is also:
	// 4(s2-s1)² - 4h²(u1+u2) = 4(s2-s1)² - 4h³ - 4h²(2u1)
	//                        = r² - j - 2v
	// with the notations below.
	h := &gfP{}
	gfpSub(h, u2, u1)
	xEqual := *h == gfP{0}

	gfpAdd(t, h, h)
	// i = 4h²
	i := &gfP{}
	gfpMul(i, t, t)
	// j = 4h³
	j := &gfP{}
	gfpMul(j, h, i)

	gfpSub(t, s2, s1)
	yEqual := *t == gfP{0}
	if xEqual && yEqual {
		c.Double(a)
		return
	}
	r := &gfP{}
	gfpAdd(r, t, t)

	v := &gfP{}
	gfpMul(v, u1, i)

	// t4 = 4(s2-s1)²
	t4, t6 := &gfP{}, &gfP{}
	gfpMul(t4, r, r)
	gfpAdd(t, v, v)
	gfpSub(t6, t4, j)

	gfpSub(&c.x, t6, t)

	// Set y = -(2h)³(s1 + s*(x/4h²-u1))
	// This is also
	// y = - 2·s1·j - (s2-s1)(2x - 2i·u1) = r(v-x) - 2·s1·j
	gfpSub(t, v, &c.x) // t7
	gfpMul(t4, s1, j)  // t8
	gfpAdd(t6, t4, t4) // t9
	gfpMul(t4, r, t)   // t10
	gfpSub(&c.y, t4, t6)

	// Set z = 2(u2-u1)·z1·z2 = 2h·z1·z2
	gfpAdd(t, &a.z, &b.z) // t11
	gfpMul(t4, t, t)      // t12
	gfpSub(t, t4, z12)    // t13
	gfpSub(t4, t, z22)    // t14
	gfpMul(&c.z, t4, h)
}

func (c *curvePoint) Double(a *curvePoint) {
	// See http://hyperelliptic.org/EFD/g1p/auto-code/shortw/jacobian-0/doubling/dbl-2009-l.op3
	A, B, C := &gfP{}, &gfP{}, &gfP{}
	gfpMul(A, &a.x, &a.x)
	gfpMul(B, &a.y, &a.y)
	gfpMul(C, B, B)

	t, t2 := &gfP{}, &gfP{}
	gfpAdd(t, &a.x, B)
	gfpMul(t2, t, t)
	gfpSub(t, t2, A)
	gfpSub(t2, t, C)

	d, e, f := &gfP{}, &gfP{}, &gfP{}
	gfpAdd(d, t2, t2)
	gfpAdd(t, A, A)
	gfpAdd(e, t, A)
	gfpMul(f, e, e)

	gfpAdd(t, d, d)
	gfpSub(&c.x, f, t)

	gfpMul(&c.z, &a.y, &a.z)
	gfpAdd(&c.z, &c.z, &c.z)

	gfpAdd(t, C, C)
	gfpAdd(t2, t, t)
	gfpAdd(t, t2, t2)
	gfpSub(&c.y, d, &c.x)
	gfpMul(t2, e, &c.y)
	gfpSub(&c.y, t2, t)
}

func (c *curvePoint) Mul(a *curvePoint, scalar *big.Int) {
	precomp := [1 << 2]*curvePoint{nil, {}, {}, {}}
	precomp[1].Set(a)
	precomp[2].Set(a)
	gfpMul(&precomp[2].x, &precomp[2].x, xiTo2PSquaredMinus2Over3)
	precomp[3].Add(precomp[1], precomp[2])

	multiScalar := curveLattice.Multi(scalar)

	sum := &curvePoint{}
	sum.SetInfinity()
	t := &curvePoint{}

	for i := len(multiScalar) - 1; i >= 0; i-- {
		t.Double(sum)
		if multiScalar[i] == 0 {
			sum.Set(t)
		} else {
			sum.Add(t, precomp[multiScalar[i]])
		}
	}
	c.Set(sum)
}

func (c *curvePoint) MakeAffine() {
	if c.z == *newGFp(1) {
		return
	} else if c.z == *newGFp(0) {
		c.x = gfP{0}
		c.y = *newGFp(1)
		c.t = gfP{0}
		return
	}

	zInv := &gfP{}
	zInv.Invert(&c.z)

	t, zInv2 := &gfP{}, &gfP{}
	gfpMul(t, &c.y, zInv)
	gfpMul(zInv2, zInv, zInv)

	gfpMul(&c.x, &c.x, zInv2)
	gfpMul(&c.y, t, zInv2)

	c.z = *newGFp(1)
	c.t = *newGFp(1)
}

func (c *curvePoint) Neg(a *curvePoint) {
	c.x.Set(&a.x)
	gfpNeg(&c.y, &a.y)
	c.z.Set(&a.z)
	c.t = gfP{0}
}
