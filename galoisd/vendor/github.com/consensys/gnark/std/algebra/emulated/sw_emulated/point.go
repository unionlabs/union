package sw_emulated

import (
	"fmt"
	"math/big"

	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/algebra/algopts"
	"github.com/consensys/gnark/std/math/emulated"
	"golang.org/x/exp/slices"
)

// New returns a new [Curve] instance over the base field Base and scalar field
// Scalars defined by the curve parameters params. It returns an error if
// initialising the field emulation fails (for example, when the native field is
// too small) or when the curve parameters are incompatible with the fields.
func New[Base, Scalars emulated.FieldParams](api frontend.API, params CurveParams) (*Curve[Base, Scalars], error) {
	ba, err := emulated.NewField[Base](api)
	if err != nil {
		return nil, fmt.Errorf("new base api: %w", err)
	}
	sa, err := emulated.NewField[Scalars](api)
	if err != nil {
		return nil, fmt.Errorf("new scalar api: %w", err)
	}
	emuGm := make([]AffinePoint[Base], len(params.Gm))
	for i, v := range params.Gm {
		emuGm[i] = AffinePoint[Base]{emulated.ValueOf[Base](v[0]), emulated.ValueOf[Base](v[1])}
	}
	Gx := emulated.ValueOf[Base](params.Gx)
	Gy := emulated.ValueOf[Base](params.Gy)
	var eigenvalue *emulated.Element[Scalars]
	var thirdRootOne *emulated.Element[Base]
	if params.Eigenvalue != nil && params.ThirdRootOne != nil {
		eigenvalue = sa.NewElement(params.Eigenvalue)
		thirdRootOne = ba.NewElement(params.ThirdRootOne)
	}
	return &Curve[Base, Scalars]{
		params:    params,
		api:       api,
		baseApi:   ba,
		scalarApi: sa,
		g: AffinePoint[Base]{
			X: Gx,
			Y: Gy,
		},
		gm:           emuGm,
		a:            emulated.ValueOf[Base](params.A),
		b:            emulated.ValueOf[Base](params.B),
		addA:         params.A.Cmp(big.NewInt(0)) != 0,
		eigenvalue:   eigenvalue,
		thirdRootOne: thirdRootOne,
	}, nil
}

// Curve is an initialised curve which allows performing group operations.
type Curve[Base, Scalars emulated.FieldParams] struct {
	// params is the parameters of the curve
	params CurveParams
	// api is the native api, we construct it ourselves to be sure
	api frontend.API
	// baseApi is the api for point operations
	baseApi *emulated.Field[Base]
	// scalarApi is the api for scalar operations
	scalarApi *emulated.Field[Scalars]

	// g is the generator (base point) of the curve.
	g AffinePoint[Base]

	// gm are the pre-computed multiples the generator (base point) of the curve.
	gm []AffinePoint[Base]

	a            emulated.Element[Base]
	b            emulated.Element[Base]
	addA         bool
	eigenvalue   *emulated.Element[Scalars]
	thirdRootOne *emulated.Element[Base]
}

// Generator returns the base point of the curve. The method does not copy and
// modifying the returned element leads to undefined behaviour!
func (c *Curve[B, S]) Generator() *AffinePoint[B] {
	return &c.g
}

// GeneratorMultiples returns the pre-computed multiples of the base point of the curve. The method does not copy and
// modifying the returned element leads to undefined behaviour!
func (c *Curve[B, S]) GeneratorMultiples() []AffinePoint[B] {
	return c.gm
}

// AffinePoint represents a point on the elliptic curve. We do not check that
// the point is actually on the curve.
//
// Point (0,0) represents point at the infinity. This representation is
// compatible with the EVM representations of points at infinity.
type AffinePoint[Base emulated.FieldParams] struct {
	X, Y emulated.Element[Base]
}

// MarshalScalar marshals the scalar into bits. Compatible with scalar
// marshalling in gnark-crypto.
func (c *Curve[B, S]) MarshalScalar(s emulated.Element[S]) []frontend.Variable {
	var fr S
	nbBits := 8 * ((fr.Modulus().BitLen() + 7) / 8)
	sReduced := c.scalarApi.Reduce(&s)
	res := c.scalarApi.ToBits(sReduced)[:nbBits]
	for i, j := 0, nbBits-1; i < j; {
		res[i], res[j] = res[j], res[i]
		i++
		j--
	}
	return res
}

// MarshalG1 marshals the affine point into bits. The output is compatible with
// the point marshalling in gnark-crypto.
func (c *Curve[B, S]) MarshalG1(p AffinePoint[B]) []frontend.Variable {
	var fp B
	nbBits := 8 * ((fp.Modulus().BitLen() + 7) / 8)
	x := c.baseApi.Reduce(&p.X)
	y := c.baseApi.Reduce(&p.Y)
	bx := c.baseApi.ToBits(x)[:nbBits]
	by := c.baseApi.ToBits(y)[:nbBits]
	slices.Reverse(bx)
	slices.Reverse(by)
	res := make([]frontend.Variable, 2*nbBits)
	copy(res, bx)
	copy(res[len(bx):], by)
	xZ := c.baseApi.IsZero(x)
	yZ := c.baseApi.IsZero(y)
	res[1] = c.api.Mul(xZ, yZ)
	return res
}

// Neg returns an inverse of p. It doesn't modify p.
func (c *Curve[B, S]) Neg(p *AffinePoint[B]) *AffinePoint[B] {
	return &AffinePoint[B]{
		X: p.X,
		Y: *c.baseApi.Neg(&p.Y),
	}
}

// AssertIsEqual asserts that p and q are the same point.
func (c *Curve[B, S]) AssertIsEqual(p, q *AffinePoint[B]) {
	c.baseApi.AssertIsEqual(&p.X, &q.X)
	c.baseApi.AssertIsEqual(&p.Y, &q.Y)
}

// add adds p and q and returns it. It doesn't modify p nor q.
//
// ⚠️  p must be different than q and -q, and both nonzero.
//
// It uses incomplete formulas in affine coordinates.
func (c *Curve[B, S]) add(p, q *AffinePoint[B]) *AffinePoint[B] {
	// compute λ = (q.y-p.y)/(q.x-p.x)
	qypy := c.baseApi.Sub(&q.Y, &p.Y)
	qxpx := c.baseApi.Sub(&q.X, &p.X)
	λ := c.baseApi.Div(qypy, qxpx)

	// xr = λ²-p.x-q.x
	λλ := c.baseApi.MulMod(λ, λ)
	qxpx = c.baseApi.Add(&p.X, &q.X)
	xr := c.baseApi.Sub(λλ, qxpx)

	// p.y = λ(p.x-r.x) - p.y
	pxrx := c.baseApi.Sub(&p.X, xr)
	λpxrx := c.baseApi.MulMod(λ, pxrx)
	yr := c.baseApi.Sub(λpxrx, &p.Y)

	return &AffinePoint[B]{
		X: *c.baseApi.Reduce(xr),
		Y: *c.baseApi.Reduce(yr),
	}
}

// AssertIsOnCurve asserts if p belongs to the curve. It doesn't modify p.
func (c *Curve[B, S]) AssertIsOnCurve(p *AffinePoint[B]) {
	// (X,Y) ∈ {Y² == X³ + aX + b} U (0,0)

	// if p=(0,0) we assign b=0 and continue
	selector := c.api.And(c.baseApi.IsZero(&p.X), c.baseApi.IsZero(&p.Y))
	b := c.baseApi.Select(selector, c.baseApi.Zero(), &c.b)

	left := c.baseApi.Mul(&p.Y, &p.Y)
	right := c.baseApi.Mul(&p.X, c.baseApi.Mul(&p.X, &p.X))
	right = c.baseApi.Add(right, b)
	if c.addA {
		ax := c.baseApi.Mul(&c.a, &p.X)
		right = c.baseApi.Add(right, ax)
	}
	c.baseApi.AssertIsEqual(left, right)
}

// AddUnified adds p and q and returns it. It doesn't modify p nor q.
//
// ✅ p can be equal to q, and either or both can be (0,0).
// (0,0) is not on the curve but we conventionally take it as the
// neutral/infinity point as per the [EVM].
//
// It uses the unified formulas of Brier and Joye ([[BriJoy02]] (Corollary 1)).
//
// [BriJoy02]: https://link.springer.com/content/pdf/10.1007/3-540-45664-3_24.pdf
// [EVM]: https://ethereum.github.io/yellowpaper/paper.pdf
func (c *Curve[B, S]) AddUnified(p, q *AffinePoint[B]) *AffinePoint[B] {

	// selector1 = 1 when p is (0,0) and 0 otherwise
	selector1 := c.api.And(c.baseApi.IsZero(&p.X), c.baseApi.IsZero(&p.Y))
	// selector2 = 1 when q is (0,0) and 0 otherwise
	selector2 := c.api.And(c.baseApi.IsZero(&q.X), c.baseApi.IsZero(&q.Y))

	// λ = ((p.x+q.x)² - p.x*q.x + a)/(p.y + q.y)
	pxqx := c.baseApi.MulMod(&p.X, &q.X)
	pxplusqx := c.baseApi.Add(&p.X, &q.X)
	num := c.baseApi.MulMod(pxplusqx, pxplusqx)
	num = c.baseApi.Sub(num, pxqx)
	if c.addA {
		num = c.baseApi.Add(num, &c.a)
	}
	denum := c.baseApi.Add(&p.Y, &q.Y)
	// if p.y + q.y = 0, assign dummy 1 to denum and continue
	selector3 := c.baseApi.IsZero(denum)
	denum = c.baseApi.Select(selector3, c.baseApi.One(), denum)
	λ := c.baseApi.Div(num, denum)

	// x = λ^2 - p.x - q.x
	xr := c.baseApi.MulMod(λ, λ)
	xr = c.baseApi.Sub(xr, pxplusqx)

	// y = λ(p.x - xr) - p.y
	yr := c.baseApi.Sub(&p.X, xr)
	yr = c.baseApi.MulMod(yr, λ)
	yr = c.baseApi.Sub(yr, &p.Y)
	result := AffinePoint[B]{
		X: *c.baseApi.Reduce(xr),
		Y: *c.baseApi.Reduce(yr),
	}

	zero := c.baseApi.Zero()
	infinity := AffinePoint[B]{X: *zero, Y: *zero}
	// if p=(0,0) return q
	result = *c.Select(selector1, q, &result)
	// if q=(0,0) return p
	result = *c.Select(selector2, p, &result)
	// if p.y + q.y = 0, return (0, 0)
	result = *c.Select(selector3, &infinity, &result)

	return &result
}

// Add performs unsafe addition of points p and q. For safe addition use
// [Curve.AddUnified].
func (c *Curve[B, S]) Add(p, q *AffinePoint[B]) *AffinePoint[B] {
	return c.add(p, q)
}

// double doubles p and return it. It doesn't modify p.
//
// ⚠️  p.Y must be nonzero.
//
// It uses affine coordinates.
func (c *Curve[B, S]) double(p *AffinePoint[B]) *AffinePoint[B] {

	// compute λ = (3p.x²+a)/2*p.y, here we assume a=0 (j invariant 0 curve)
	xx3a := c.baseApi.MulMod(&p.X, &p.X)
	xx3a = c.baseApi.MulConst(xx3a, big.NewInt(3))
	if c.addA {
		xx3a = c.baseApi.Add(xx3a, &c.a)
	}
	y2 := c.baseApi.MulConst(&p.Y, big.NewInt(2))
	λ := c.baseApi.Div(xx3a, y2)

	// xr = λ²-2p.x
	x2 := c.baseApi.MulConst(&p.X, big.NewInt(2))
	λλ := c.baseApi.MulMod(λ, λ)
	xr := c.baseApi.Sub(λλ, x2)

	// yr = λ(p-xr) - p.y
	pxrx := c.baseApi.Sub(&p.X, xr)
	λpxrx := c.baseApi.MulMod(λ, pxrx)
	yr := c.baseApi.Sub(λpxrx, &p.Y)

	return &AffinePoint[B]{
		X: *c.baseApi.Reduce(xr),
		Y: *c.baseApi.Reduce(yr),
	}
}

// triple triples p and return it. It follows [ELM03] (Section 3.1).
// Saves the computation of the y coordinate of 2p as it is used only in the computation of λ2,
// which can be computed as
//
//	λ2 = -λ1-2*p.y/(x2-p.x)
//
// instead. It doesn't modify p.
//
// ⚠️  p.Y must be nonzero.
//
// [ELM03]: https://arxiv.org/pdf/math/0208038.pdf
func (c *Curve[B, S]) triple(p *AffinePoint[B]) *AffinePoint[B] {

	// compute λ1 = (3p.x²+a)/2p.y, here we assume a=0 (j invariant 0 curve)
	xx := c.baseApi.MulMod(&p.X, &p.X)
	xx = c.baseApi.MulConst(xx, big.NewInt(3))
	if c.addA {
		xx = c.baseApi.Add(xx, &c.a)
	}
	y2 := c.baseApi.MulConst(&p.Y, big.NewInt(2))
	λ1 := c.baseApi.Div(xx, y2)

	// xr = λ1²-2p.x
	x2 := c.baseApi.MulConst(&p.X, big.NewInt(2))
	λ1λ1 := c.baseApi.MulMod(λ1, λ1)
	x2 = c.baseApi.Sub(λ1λ1, x2)

	// ommit y2 computation, and
	// compute λ2 = 2p.y/(x2 − p.x) − λ1.
	x1x2 := c.baseApi.Sub(&p.X, x2)
	λ2 := c.baseApi.Div(y2, x1x2)
	λ2 = c.baseApi.Sub(λ2, λ1)

	// xr = λ²-p.x-x2
	λ2λ2 := c.baseApi.MulMod(λ2, λ2)
	qxrx := c.baseApi.Add(x2, &p.X)
	xr := c.baseApi.Sub(λ2λ2, qxrx)

	// yr = λ(p.x-xr) - p.y
	pxrx := c.baseApi.Sub(&p.X, xr)
	λ2pxrx := c.baseApi.MulMod(λ2, pxrx)
	yr := c.baseApi.Sub(λ2pxrx, &p.Y)

	return &AffinePoint[B]{
		X: *c.baseApi.Reduce(xr),
		Y: *c.baseApi.Reduce(yr),
	}
}

// doubleAndAdd computes 2p+q as (p+q)+p. It follows [ELM03] (Section 3.1)
// Saves the computation of the y coordinate of p+q as it is used only in the computation of λ2,
// which can be computed as
//
//	λ2 = -λ1-2*p.y/(x2-p.x)
//
// instead. It doesn't modify p nor q.
//
// ⚠️  p must be different than q and -q, and both nonzero.
//
// [ELM03]: https://arxiv.org/pdf/math/0208038.pdf
func (c *Curve[B, S]) doubleAndAdd(p, q *AffinePoint[B]) *AffinePoint[B] {

	// compute λ1 = (q.y-p.y)/(q.x-p.x)
	yqyp := c.baseApi.Sub(&q.Y, &p.Y)
	xqxp := c.baseApi.Sub(&q.X, &p.X)
	λ1 := c.baseApi.Div(yqyp, xqxp)

	// compute x2 = λ1²-p.x-q.x
	λ1λ1 := c.baseApi.MulMod(λ1, λ1)
	xqxp = c.baseApi.Add(&p.X, &q.X)
	x2 := c.baseApi.Sub(λ1λ1, xqxp)

	// ommit y2 computation
	// compute λ2 = λ1+2*p.y/(x2-p.x)
	ypyp := c.baseApi.Add(&p.Y, &p.Y)
	x2xp := c.baseApi.Sub(x2, &p.X)
	λ2 := c.baseApi.Div(ypyp, x2xp)
	λ2 = c.baseApi.Add(λ1, λ2)

	// compute x3 =λ2²-p.x-x2
	λ2λ2 := c.baseApi.MulMod(λ2, λ2)
	x3 := c.baseApi.Sub(λ2λ2, c.baseApi.Add(&p.X, x2))

	// compute y3 = λ2*(-p.x + x3)-p.y
	y3 := c.baseApi.Sub(x3, &p.X)
	y3 = c.baseApi.Mul(λ2, y3)
	y3 = c.baseApi.Sub(y3, &p.Y)

	return &AffinePoint[B]{
		X: *c.baseApi.Reduce(x3),
		Y: *c.baseApi.Reduce(y3),
	}

}

// doubleAndAddSelect is the same as doubleAndAdd but computes either:
//
//	2p+q if b=1 or
//	2q+p if b=0
//
// It first computes the x-coordinate of p+q via the slope(p,q)
// and then based on a Select adds either p or q.
func (c *Curve[B, S]) doubleAndAddSelect(b frontend.Variable, p, q *AffinePoint[B]) *AffinePoint[B] {

	// compute λ1 = (q.y-p.y)/(q.x-p.x)
	yqyp := c.baseApi.Sub(&q.Y, &p.Y)
	xqxp := c.baseApi.Sub(&q.X, &p.X)
	λ1 := c.baseApi.Div(yqyp, xqxp)

	// compute x2 = λ1²-p.x-q.x
	λ1λ1 := c.baseApi.MulMod(λ1, λ1)
	xqxp = c.baseApi.Add(&p.X, &q.X)
	x2 := c.baseApi.Sub(λ1λ1, xqxp)

	// ommit y2 computation

	// conditional second addition
	t := c.Select(b, p, q)

	// compute λ2 = λ1+2*t.y/(x2-t.x)
	ypyp := c.baseApi.Add(&t.Y, &t.Y)
	x2xp := c.baseApi.Sub(x2, &t.X)
	λ2 := c.baseApi.Div(ypyp, x2xp)
	λ2 = c.baseApi.Add(λ1, λ2)

	// compute x3 =λ2²-t.x-x2
	λ2λ2 := c.baseApi.MulMod(λ2, λ2)
	x3 := c.baseApi.Sub(λ2λ2, c.baseApi.Add(&t.X, x2))

	// compute y3 = -λ2*(t.x - x3)-t.y
	y3 := c.baseApi.Sub(x3, &t.X)
	y3 = c.baseApi.Mul(λ2, y3)
	y3 = c.baseApi.Sub(y3, &t.Y)

	return &AffinePoint[B]{
		X: *c.baseApi.Reduce(x3),
		Y: *c.baseApi.Reduce(y3),
	}

}

// Select selects between p and q given the selector b. If b == 1, then returns
// p and q otherwise.
func (c *Curve[B, S]) Select(b frontend.Variable, p, q *AffinePoint[B]) *AffinePoint[B] {
	x := c.baseApi.Select(b, &p.X, &q.X)
	y := c.baseApi.Select(b, &p.Y, &q.Y)
	return &AffinePoint[B]{
		X: *x,
		Y: *y,
	}
}

// Lookup2 performs a 2-bit lookup between i0, i1, i2, i3 based on bits b0
// and b1. Returns:
//   - i0 if b0=0 and b1=0,
//   - i1 if b0=1 and b1=0,
//   - i2 if b0=0 and b1=1,
//   - i3 if b0=1 and b1=1.
func (c *Curve[B, S]) Lookup2(b0, b1 frontend.Variable, i0, i1, i2, i3 *AffinePoint[B]) *AffinePoint[B] {
	x := c.baseApi.Lookup2(b0, b1, &i0.X, &i1.X, &i2.X, &i3.X)
	y := c.baseApi.Lookup2(b0, b1, &i0.Y, &i1.Y, &i2.Y, &i3.Y)
	return &AffinePoint[B]{
		X: *x,
		Y: *y,
	}
}

// ScalarMul computes s * p and returns it. It doesn't modify p nor s.
// This function doesn't check that the p is on the curve. See AssertIsOnCurve.
//
// ScalarMul calls ScalarMulGeneric or scalarMulGLV depending on whether an efficient endomorphism is available.
func (c *Curve[B, S]) ScalarMul(p *AffinePoint[B], s *emulated.Element[S], opts ...algopts.AlgebraOption) *AffinePoint[B] {
	if c.eigenvalue != nil && c.thirdRootOne != nil {
		return c.scalarMulGLV(p, s, opts...)

	} else {
		return c.ScalarMulGeneric(p, s, opts...)

	}
}

// scalarMulGLV computes s * Q using an efficient endomorphism and returns it. It doesn't modify Q nor s.
//
// ⚠️  The scalar s must be nonzero and the point Q different from (0,0).
func (c *Curve[B, S]) scalarMulGLV(Q *AffinePoint[B], s *emulated.Element[S], opts ...algopts.AlgebraOption) *AffinePoint[B] {
	var st S
	frModulus := c.scalarApi.Modulus()
	sd, err := c.scalarApi.NewHint(decomposeScalarG1, 5, s, c.eigenvalue, frModulus)
	if err != nil {
		panic(fmt.Sprintf("compute GLV decomposition: %v", err))
	}
	s1, s2 := sd[0], sd[1]
	c.scalarApi.AssertIsEqual(
		c.scalarApi.Add(s1, c.scalarApi.Mul(s2, c.eigenvalue)),
		c.scalarApi.Add(s, c.scalarApi.Mul(frModulus, sd[2])),
	)
	selector1 := c.scalarApi.IsZero(c.scalarApi.Sub(sd[3], s1))
	selector2 := c.scalarApi.IsZero(c.scalarApi.Sub(sd[4], s2))

	var Acc, B1 *AffinePoint[B]
	// precompute -Q, -Φ(Q), Φ(Q)
	var tableQ, tablePhiQ [2]*AffinePoint[B]
	tableQ[1] = &AffinePoint[B]{
		X: Q.X,
		Y: *c.baseApi.Select(selector1, &Q.Y, c.baseApi.Neg(&Q.Y)),
	}
	tableQ[0] = c.Neg(tableQ[1])
	tablePhiQ[1] = &AffinePoint[B]{
		X: *c.baseApi.Mul(&Q.X, c.thirdRootOne),
		Y: *c.baseApi.Select(selector2, &Q.Y, c.baseApi.Neg(&Q.Y)),
	}
	tablePhiQ[0] = c.Neg(tablePhiQ[1])

	// Acc = Q + Φ(Q)
	Acc = c.Add(tableQ[1], tablePhiQ[1])

	s1 = c.scalarApi.Select(selector1, s1, sd[3])
	s2 = c.scalarApi.Select(selector2, s2, sd[4])

	s1bits := c.scalarApi.ToBits(s1)
	s2bits := c.scalarApi.ToBits(s2)

	nbits := st.Modulus().BitLen()>>1 + 2

	for i := nbits - 1; i > 0; i-- {
		B1 = &AffinePoint[B]{
			X: tableQ[0].X,
			Y: *c.baseApi.Select(s1bits[i], &tableQ[1].Y, &tableQ[0].Y),
		}
		Acc = c.doubleAndAdd(Acc, B1)
		B1 = &AffinePoint[B]{
			X: tablePhiQ[0].X,
			Y: *c.baseApi.Select(s2bits[i], &tablePhiQ[1].Y, &tablePhiQ[0].Y),
		}
		Acc = c.Add(Acc, B1)

	}

	tableQ[0] = c.Add(tableQ[0], Acc)
	Acc = c.Select(s1bits[0], Acc, tableQ[0])
	tablePhiQ[0] = c.Add(tablePhiQ[0], Acc)
	Acc = c.Select(s2bits[0], Acc, tablePhiQ[0])

	return Acc
}

// ScalarMulGeneric computes s * p and returns it. It doesn't modify p nor s.
// This function doesn't check that the p is on the curve. See AssertIsOnCurve.
//
// ✅ p can can be (0,0) and s can be 0.
// (0,0) is not on the curve but we conventionally take it as the
// neutral/infinity point as per the [EVM].
//
// It computes the right-to-left variable-base double-and-add algorithm ([Joye07], Alg.1).
//
// Since we use incomplete formulas for the addition law, we need to start with
// a non-zero accumulator point (R0). To do this, we skip the LSB (bit at
// position 0) and proceed assuming it was 1. At the end, we conditionally
// subtract the initial value (p) if LSB is 1. We also handle the bits at
// positions 1 and n-1 outside of the loop to optimize the number of
// constraints using [ELM03] (Section 3.1)
//
// [ELM03]: https://arxiv.org/pdf/math/0208038.pdf
// [EVM]: https://ethereum.github.io/yellowpaper/paper.pdf
// [Joye07]: https://www.iacr.org/archive/ches2007/47270135/47270135.pdf
func (c *Curve[B, S]) ScalarMulGeneric(p *AffinePoint[B], s *emulated.Element[S], opts ...algopts.AlgebraOption) *AffinePoint[B] {
	cfg, err := algopts.NewConfig(opts...)
	if err != nil {
		panic(fmt.Sprintf("parse opts: %v", err))
	}

	// if p=(0,0) we assign a dummy (0,1) to p and continue
	selector := c.api.And(c.baseApi.IsZero(&p.X), c.baseApi.IsZero(&p.Y))
	one := c.baseApi.One()
	p = c.Select(selector, &AffinePoint[B]{X: *one, Y: *one}, p)

	var st S
	sr := c.scalarApi.Reduce(s)
	sBits := c.scalarApi.ToBits(sr)
	n := st.Modulus().BitLen()
	if cfg.NbScalarBits > 2 && cfg.NbScalarBits < n {
		n = cfg.NbScalarBits
	}

	// i = 1
	Rb := c.triple(p)
	R0 := c.Select(sBits[1], Rb, p)
	R1 := c.Select(sBits[1], p, Rb)

	for i := 2; i < n-1; i++ {
		Rb = c.doubleAndAddSelect(sBits[i], R0, R1)
		R0 = c.Select(sBits[i], Rb, R0)
		R1 = c.Select(sBits[i], R1, Rb)
	}

	// i = n-1
	Rb = c.doubleAndAddSelect(sBits[n-1], R0, R1)
	R0 = c.Select(sBits[n-1], Rb, R0)

	// i = 0
	// we use AddUnified here instead of add so that when s=0, res=(0,0)
	// because AddUnified(p, -p) = (0,0)
	R0 = c.Select(sBits[0], R0, c.AddUnified(R0, c.Neg(p)))

	// if p=(0,0), return (0,0)
	zero := c.baseApi.Zero()
	R0 = c.Select(selector, &AffinePoint[B]{X: *zero, Y: *zero}, R0)

	return R0
}

// jointScalarMul computes s1 * p1 + s2 * p2 and returns it. It doesn't modify the inputs.
// This function doesn't check that the p1 and p2 are on the curve. See AssertIsOnCurve.
//
// jointScalarMul calls jointScalarMulGeneric or jointScalarMulGLV depending on whether an efficient endomorphism is available.
func (c *Curve[B, S]) jointScalarMul(p1, p2 *AffinePoint[B], s1, s2 *emulated.Element[S], opts ...algopts.AlgebraOption) *AffinePoint[B] {
	if c.params.Eigenvalue != nil && c.params.ThirdRootOne != nil {
		return c.jointScalarMulGLV(p1, p2, s1, s2, opts...)

	} else {
		return c.jointScalarMulGeneric(p1, p2, s1, s2, opts...)

	}
}

// jointScalarMulGeneric computes s1 * p1 + s2 * p2 and returns it. It doesn't modify the inputs.
//
// ⚠️  The scalar s must be nonzero and the point Q different from (0,0).
func (c *Curve[B, S]) jointScalarMulGeneric(p1, p2 *AffinePoint[B], s1, s2 *emulated.Element[S], opts ...algopts.AlgebraOption) *AffinePoint[B] {
	s1r := c.scalarApi.Reduce(s1)
	s1Bits := c.scalarApi.ToBits(s1r)
	s2r := c.scalarApi.Reduce(s2)
	s2Bits := c.scalarApi.ToBits(s2r)

	res := c.scalarBitsMul(p1, s1Bits, opts...)
	tmp := c.scalarBitsMul(p2, s2Bits, opts...)

	res = c.Add(res, tmp)
	return res
}

// jointScalarMulGLV computes P = [s]Q + [t]R using Shamir's trick with an efficient endomorphism and returns it. It doesn't modify P, Q nor s.
//
// ⚠️  The scalar s must be nonzero and the point Q different from (0,0).
func (c *Curve[B, S]) jointScalarMulGLV(Q, R *AffinePoint[B], s, t *emulated.Element[S], opts ...algopts.AlgebraOption) *AffinePoint[B] {
	var st S
	frModulus := c.scalarApi.Modulus()
	sd, err := c.scalarApi.NewHint(decomposeScalarG1, 5, s, c.eigenvalue, frModulus)
	if err != nil {
		// err is non-nil only for invalid number of inputs
		panic(err)
	}
	s1, s2 := sd[0], sd[1]

	td, err := c.scalarApi.NewHint(decomposeScalarG1, 5, t, c.eigenvalue, frModulus)
	if err != nil {
		// err is non-nil only for invalid number of inputs
		panic(err)
	}
	t1, t2 := td[0], td[1]

	c.scalarApi.AssertIsEqual(
		c.scalarApi.Add(s1, c.scalarApi.Mul(s2, c.eigenvalue)),
		c.scalarApi.Add(s, c.scalarApi.Mul(frModulus, sd[2])),
	)
	c.scalarApi.AssertIsEqual(
		c.scalarApi.Add(t1, c.scalarApi.Mul(t2, c.eigenvalue)),
		c.scalarApi.Add(t, c.scalarApi.Mul(frModulus, td[2])),
	)
	selector1 := c.scalarApi.IsZero(c.scalarApi.Sub(sd[3], s1))
	selector2 := c.scalarApi.IsZero(c.scalarApi.Sub(sd[4], s2))
	selector3 := c.scalarApi.IsZero(c.scalarApi.Sub(td[3], t1))
	selector4 := c.scalarApi.IsZero(c.scalarApi.Sub(td[4], t2))

	// precompute -Q, -Φ(Q), Φ(Q)
	var tableQ, tablePhiQ [2]*AffinePoint[B]
	tableQ[1] = &AffinePoint[B]{
		X: Q.X,
		Y: *c.baseApi.Select(selector1, &Q.Y, c.baseApi.Neg(&Q.Y)),
	}
	tableQ[0] = c.Neg(tableQ[1])
	tablePhiQ[1] = &AffinePoint[B]{
		X: *c.baseApi.Mul(&Q.X, c.thirdRootOne),
		Y: *c.baseApi.Select(selector2, &Q.Y, c.baseApi.Neg(&Q.Y)),
	}
	tablePhiQ[0] = c.Neg(tablePhiQ[1])
	// precompute -R, -Φ(R), Φ(R)
	var tableR, tablePhiR [2]*AffinePoint[B]
	tableR[1] = &AffinePoint[B]{
		X: R.X,
		Y: *c.baseApi.Select(selector3, &R.Y, c.baseApi.Neg(&R.Y)),
	}
	tableR[0] = c.Neg(tableR[1])
	tablePhiR[1] = &AffinePoint[B]{
		X: *c.baseApi.Mul(&R.X, c.thirdRootOne),
		Y: *c.baseApi.Select(selector4, &R.Y, c.baseApi.Neg(&R.Y)),
	}
	tablePhiR[0] = c.Neg(tablePhiR[1])
	// precompute Q+R, -Q-R, Q-R, -Q+R, Φ(Q)+Φ(R), -Φ(Q)-Φ(R), Φ(Q)-Φ(R), -Φ(Q)+Φ(R)
	var tableS, tablePhiS [4]*AffinePoint[B]
	tableS[0] = tableQ[0]
	tableS[0] = c.Add(tableS[0], tableR[0])
	tableS[1] = c.Neg(tableS[0])
	tableS[2] = tableQ[1]
	tableS[2] = c.Add(tableS[2], tableR[0])
	tableS[3] = c.Neg(tableS[2])
	f0 := c.baseApi.Mul(&tableS[0].X, c.thirdRootOne)
	f1 := c.baseApi.Mul(&tableS[1].X, c.thirdRootOne)
	f2 := c.baseApi.Mul(&tableS[2].X, c.thirdRootOne)
	f3 := c.baseApi.Mul(&tableS[3].X, c.thirdRootOne)
	tablePhiS[0] = &AffinePoint[B]{
		X: *c.baseApi.Lookup2(selector2, selector4, f1, f3, f2, f0),
		Y: *c.baseApi.Lookup2(selector2, selector4, &tableS[1].Y, &tableS[3].Y, &tableS[2].Y, &tableS[0].Y),
	}
	tablePhiS[1] = c.Neg(tablePhiS[0])
	tablePhiS[2] = &AffinePoint[B]{
		X: *c.baseApi.Lookup2(selector2, selector4, f3, f1, f0, f2),
		Y: *c.baseApi.Lookup2(selector2, selector4, &tableS[3].Y, &tableS[1].Y, &tableS[0].Y, &tableS[2].Y),
	}
	tablePhiS[3] = c.Neg(tablePhiS[2])

	// suppose first bit is 1 and set:
	// Acc = Q + R + Φ(Q) + Φ(R)
	Acc := c.Add(tableS[1], tablePhiS[1])

	s1 = c.scalarApi.Select(selector1, s1, sd[3])
	s2 = c.scalarApi.Select(selector2, s2, sd[4])
	t1 = c.scalarApi.Select(selector3, t1, td[3])
	t2 = c.scalarApi.Select(selector4, t2, td[4])

	s1bits := c.scalarApi.ToBits(s1)
	s2bits := c.scalarApi.ToBits(s2)
	t1bits := c.scalarApi.ToBits(t1)
	t2bits := c.scalarApi.ToBits(t2)
	nbits := st.Modulus().BitLen()>>1 + 2

	// Acc = [2]Acc ± Q ± R ± Φ(Q) ± Φ(R)
	for i := nbits - 1; i > 0; i-- {
		B1 := &AffinePoint[B]{
			X: *c.baseApi.Select(c.api.Xor(s1bits[i], t1bits[i]), &tableS[2].X, &tableS[0].X),
			Y: *c.baseApi.Lookup2(s1bits[i], t1bits[i], &tableS[0].Y, &tableS[2].Y, &tableS[3].Y, &tableS[1].Y),
		}
		Acc = c.doubleAndAdd(Acc, B1)
		B1 = &AffinePoint[B]{
			X: *c.baseApi.Select(c.api.Xor(s2bits[i], t2bits[i]), &tablePhiS[2].X, &tablePhiS[0].X),
			Y: *c.baseApi.Lookup2(s2bits[i], t2bits[i], &tablePhiS[0].Y, &tablePhiS[2].Y, &tablePhiS[3].Y, &tablePhiS[1].Y),
		}
		Acc = c.Add(Acc, B1)
	}

	// i = 0
	// subtract the initial point from the accumulator when first bit was 0
	tableQ[0] = c.Add(tableQ[0], Acc)
	Acc = c.Select(s1bits[0], Acc, tableQ[0])
	tablePhiQ[0] = c.Add(tablePhiQ[0], Acc)
	Acc = c.Select(s2bits[0], Acc, tablePhiQ[0])
	tableR[0] = c.Add(tableR[0], Acc)
	Acc = c.Select(t1bits[0], Acc, tableR[0])
	tablePhiR[0] = c.Add(tablePhiR[0], Acc)
	Acc = c.Select(t2bits[0], Acc, tablePhiR[0])

	return Acc
}

// scalarBitsMul computes s * p and returns it where sBits is the bit decomposition of s. It doesn't modify p nor sBits.
// ⚠️  Point and scalar must be nonzero.
func (c *Curve[B, S]) scalarBitsMul(p *AffinePoint[B], sBits []frontend.Variable, opts ...algopts.AlgebraOption) *AffinePoint[B] {
	cfg, err := algopts.NewConfig(opts...)
	if err != nil {
		panic(fmt.Sprintf("parse opts: %v", err))
	}

	var st S
	n := st.Modulus().BitLen()
	if cfg.NbScalarBits > 2 && cfg.NbScalarBits < n {
		n = cfg.NbScalarBits
	}

	// i = 1
	Rb := c.triple(p)
	R0 := c.Select(sBits[1], Rb, p)
	R1 := c.Select(sBits[1], p, Rb)

	for i := 2; i < n-1; i++ {
		Rb = c.doubleAndAddSelect(sBits[i], R0, R1)
		R0 = c.Select(sBits[i], Rb, R0)
		R1 = c.Select(sBits[i], R1, Rb)
	}

	// i = n-1
	Rb = c.doubleAndAddSelect(sBits[n-1], R0, R1)
	R0 = c.Select(sBits[n-1], Rb, R0)

	// i = 0
	R0 = c.Select(sBits[0], R0, c.AddUnified(R0, c.Neg(p)))

	return R0
}

// ScalarMulBase computes s * g and returns it, where g is the fixed generator.
// It doesn't modify s.
//
// ✅ When s=0, it returns (0,0).
// (0,0) is not on the curve but we conventionally take it as the
// neutral/infinity point as per the [EVM].
//
// It computes the standard little-endian fixed-base double-and-add algorithm
// [HMV04] (Algorithm 3.26), with the points [2^i]g precomputed.  The bits at
// positions 1 and 2 are handled outside of the loop to optimize the number of
// constraints using a Lookup2 with pre-computed [3]g, [5]g and [7]g points.
//
// [HMV04]: https://link.springer.com/book/10.1007/b97644
// [EVM]: https://ethereum.github.io/yellowpaper/paper.pdf
func (c *Curve[B, S]) ScalarMulBase(s *emulated.Element[S], opts ...algopts.AlgebraOption) *AffinePoint[B] {
	cfg, err := algopts.NewConfig(opts...)
	if err != nil {
		panic(fmt.Sprintf("parse opts: %v", err))
	}

	var st S
	sr := c.scalarApi.Reduce(s)
	sBits := c.scalarApi.ToBits(sr)
	n := st.Modulus().BitLen()
	if cfg.NbScalarBits > 2 && cfg.NbScalarBits < n {
		n = cfg.NbScalarBits
	}
	g := c.Generator()
	gm := c.GeneratorMultiples()

	// i = 1, 2
	// gm[0] = 3g, gm[1] = 5g, gm[2] = 7g
	res := c.Lookup2(sBits[1], sBits[2], g, &gm[0], &gm[1], &gm[2])

	for i := 3; i < n; i++ {
		// gm[i] = [2^i]g
		tmp := c.add(res, &gm[i])
		res = c.Select(sBits[i], tmp, res)
	}

	// i = 0
	tmp := c.AddUnified(res, c.Neg(g))
	res = c.Select(sBits[0], res, tmp)

	return res
}

// JointScalarMulBase computes s2 * p + s1 * g and returns it, where g is the
// fixed generator. It doesn't modify p, s1 and s2.
//
// ⚠️   p must NOT be (0,0).
// ⚠️   s1 and s2 must NOT be 0.
//
// It uses the logic from ScalarMul() for s1 * g and the logic from ScalarMulBase() for s2 * g.
//
// JointScalarMulBase is used to verify an ECDSA signature (r,s) on the
// secp256k1 curve. In this case, p is a public key, s2=r/s and s1=hash/s.
//   - hash cannot be 0, because of pre-image resistance.
//   - r cannot be 0, because r is the x coordinate of a random point on
//     secp256k1 (y²=x³+7 mod p) and 7 is not a square mod p. For any other
//     curve, (_,0) is a point of order 2 which is not the prime subgroup.
//   - (0,0) is not a valid public key.
//
// The [EVM] specifies these checks, wich are performed on the zkEVM
// arithmetization side before calling the circuit that uses this method.
//
// This saves the Select logic related to (0,0) and the use of AddUnified to
// handle the 0-scalar edge case.
func (c *Curve[B, S]) JointScalarMulBase(p *AffinePoint[B], s2, s1 *emulated.Element[S]) *AffinePoint[B] {
	return c.jointScalarMul(c.Generator(), p, s1, s2)
}

// MultiScalarMul computes the multi scalar multiplication of the points P and
// scalars s. It returns an error if the length of the slices mismatch. If the
// input slices are empty, then returns point at infinity.
//
// ⚠️  Points and scalars must be nonzero.
func (c *Curve[B, S]) MultiScalarMul(p []*AffinePoint[B], s []*emulated.Element[S], opts ...algopts.AlgebraOption) (*AffinePoint[B], error) {

	if len(p) == 0 {
		return &AffinePoint[B]{
			X: *c.baseApi.Zero(),
			Y: *c.baseApi.Zero(),
		}, nil
	}
	cfg, err := algopts.NewConfig(opts...)
	if err != nil {
		return nil, fmt.Errorf("new config: %w", err)
	}
	if !cfg.FoldMulti {
		// the scalars are unique
		if len(p) != len(s) {
			return nil, fmt.Errorf("mismatching points and scalars slice lengths")
		}
		n := len(p)
		var res *AffinePoint[B]
		if n%2 == 1 {
			res = c.ScalarMul(p[n-1], s[n-1], opts...)
		} else {
			res = c.jointScalarMul(p[n-2], p[n-1], s[n-2], s[n-1], opts...)
		}
		for i := 1; i < n-1; i += 2 {
			q := c.jointScalarMul(p[i-1], p[i], s[i-1], s[i], opts...)
			res = c.Add(res, q)
		}
		return res, nil
	} else {
		// scalars are powers
		if len(s) == 0 {
			return nil, fmt.Errorf("need scalar for folding")
		}
		gamma := s[0]
		gamma = c.scalarApi.Reduce(gamma)
		gammaBits := c.scalarApi.ToBits(gamma)
		res := c.scalarBitsMul(p[len(p)-1], gammaBits, opts...)
		for i := len(p) - 2; i > 0; i-- {
			res = c.Add(p[i], res)
			res = c.scalarBitsMul(res, gammaBits, opts...)
		}
		res = c.Add(p[0], res)
		return res, nil
	}
}
