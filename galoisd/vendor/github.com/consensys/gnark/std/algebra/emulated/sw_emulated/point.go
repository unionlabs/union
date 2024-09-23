package sw_emulated

import (
	"fmt"
	"math/big"

	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/algebra/algopts"
	"github.com/consensys/gnark/std/math/emulated"
	"github.com/consensys/gnark/std/math/emulated/emparams"
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

	// gm are the pre-computed doubles the generator (base point) of the curve.
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

// GeneratorMultiples returns the pre-computed doubles of the base point of the curve. The method does not copy and
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
	switch any(fp).(type) {
	case emparams.Secp256k1Fp:
		// in gnark-crypto we do not store the infinity bit for secp256k1 points
		return res
	}
	xZ := c.baseApi.IsZero(x)
	yZ := c.baseApi.IsZero(y)
	isZero := c.api.Mul(xZ, yZ)
	// isZero = 0 -> res[1]=0
	// isZero = 1, infty bit 0 -> res[1]=0
	// isZero = 1, infty bit 1 -> res[1]=1
	res[1] = c.api.Mul(isZero, c.marshalZeroG1())
	return res
}

// different curves have different marshalling for zero point
func (c *Curve[B, S]) marshalZeroG1() frontend.Variable {
	var fp B
	unusedBits := 64 - (fp.Modulus().BitLen() % 64)
	if unusedBits >= 3 {
		return 1
	}
	return 0
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
	ypyp := c.baseApi.MulConst(&p.Y, big.NewInt(2))
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
	ypyp := c.baseApi.MulConst(&t.Y, big.NewInt(2))
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

// Mux performs a lookup from the inputs and returns inputs[sel]. It is most
// efficient for power of two lengths of the inputs, but works for any number of
// inputs.
func (c *Curve[B, S]) Mux(sel frontend.Variable, inputs ...*AffinePoint[B]) *AffinePoint[B] {
	xs := make([]*emulated.Element[B], len(inputs))
	ys := make([]*emulated.Element[B], len(inputs))
	for i := range inputs {
		xs[i] = &inputs[i].X
		ys[i] = &inputs[i].Y
	}
	return &AffinePoint[B]{
		X: *c.baseApi.Mux(sel, xs...),
		Y: *c.baseApi.Mux(sel, ys...),
	}
}

// ScalarMul computes [s]p and returns it. It doesn't modify p nor s.
// This function doesn't check that the p is on the curve. See AssertIsOnCurve.
//
// ScalarMul calls scalarMulGeneric or scalarMulGLV depending on whether an efficient endomorphism is available.
func (c *Curve[B, S]) ScalarMul(p *AffinePoint[B], s *emulated.Element[S], opts ...algopts.AlgebraOption) *AffinePoint[B] {
	if c.eigenvalue != nil && c.thirdRootOne != nil {
		return c.scalarMulGLV(p, s, opts...)

	} else {
		return c.scalarMulGeneric(p, s, opts...)

	}
}

// scalarMulGLV computes [s]Q using an efficient endomorphism and returns it. It doesn't modify Q nor s.
// It implements an optimized version based on algorithm 1 of [Halo] (see Section 6.2 and appendix C).
//
// ⚠️  The scalar s must be nonzero and the point Q different from (0,0) unless [algopts.WithCompleteArithmetic] is set.
// (0,0) is not on the curve but we conventionally take it as the
// neutral/infinity point as per the [EVM].
//
// [Halo]: https://eprint.iacr.org/2019/1021.pdf
// [EVM]: https://ethereum.github.io/yellowpaper/paper.pdf
func (c *Curve[B, S]) scalarMulGLV(Q *AffinePoint[B], s *emulated.Element[S], opts ...algopts.AlgebraOption) *AffinePoint[B] {
	cfg, err := algopts.NewConfig(opts...)
	if err != nil {
		panic(err)
	}
	addFn := c.Add
	var selector frontend.Variable
	if cfg.CompleteArithmetic {
		addFn = c.AddUnified
		// if Q=(0,0) we assign a dummy (1,1) to Q and continue
		selector = c.api.And(c.baseApi.IsZero(&Q.X), c.baseApi.IsZero(&Q.Y))
		one := c.baseApi.One()
		Q = c.Select(selector, &AffinePoint[B]{X: *one, Y: *one}, Q)
	}

	// We use the endomorphism à la GLV to compute [s]Q as
	// 		[s1]Q + [s2]Φ(Q)
	// the sub-scalars s1, s2 can be negative (bigints) in the hint. If so,
	// they will be reduced in-circuit modulo the SNARK scalar field and not
	// the emulated field. So we return in the hint |s1|, |s2| and boolean
	// flags sdBits to negate the points Q, Φ(Q) instead of the corresponding
	// sub-scalars.

	// decompose s into s1 and s2
	sd, err := c.scalarApi.NewHint(decomposeScalarG1Subscalars, 2, s, c.eigenvalue)
	if err != nil {
		panic(fmt.Sprintf("compute GLV decomposition: %v", err))
	}
	s1, s2 := sd[0], sd[1]
	sdBits, err := c.scalarApi.NewHintWithNativeOutput(decomposeScalarG1Signs, 2, s, c.eigenvalue)
	if err != nil {
		panic(fmt.Sprintf("compute GLV decomposition bits: %v", err))
	}
	selector1, selector2 := sdBits[0], sdBits[1]
	s3 := c.scalarApi.Select(selector1, c.scalarApi.Neg(s1), s1)
	s4 := c.scalarApi.Select(selector2, c.scalarApi.Neg(s2), s2)
	// s == s3 + [λ]s4
	c.scalarApi.AssertIsEqual(
		c.scalarApi.Add(s3, c.scalarApi.Mul(s4, c.eigenvalue)),
		s,
	)

	s1bits := c.scalarApi.ToBits(s1)
	s2bits := c.scalarApi.ToBits(s2)
	var st S
	nbits := st.Modulus().BitLen()>>1 + 2

	// precompute -Q, -Φ(Q), Φ(Q)
	var tableQ, tablePhiQ [3]*AffinePoint[B]
	negQY := c.baseApi.Neg(&Q.Y)
	tableQ[1] = &AffinePoint[B]{
		X: Q.X,
		Y: *c.baseApi.Select(selector1, negQY, &Q.Y),
	}
	tableQ[0] = c.Neg(tableQ[1])
	tablePhiQ[1] = &AffinePoint[B]{
		X: *c.baseApi.Mul(&Q.X, c.thirdRootOne),
		Y: *c.baseApi.Select(selector2, negQY, &Q.Y),
	}
	tablePhiQ[0] = c.Neg(tablePhiQ[1])
	tableQ[2] = c.triple(tableQ[1])
	tablePhiQ[2] = &AffinePoint[B]{
		X: *c.baseApi.Mul(&tableQ[2].X, c.thirdRootOne),
		Y: *c.baseApi.Select(selector2, c.baseApi.Neg(&tableQ[2].Y), &tableQ[2].Y),
	}

	// we suppose that the first bits of the sub-scalars are 1 and set:
	// 		Acc = Q + Φ(Q)
	Acc := c.Add(tableQ[1], tablePhiQ[1])

	// At each iteration we need to compute:
	// 		[2]Acc ± Q ± Φ(Q).
	// We can compute [2]Acc and look up the (precomputed) point P from:
	// 		B1 = Q+Φ(Q)
	// 		B2 = -Q-Φ(Q)
	// 		B3 = Q-Φ(Q)
	// 		B4 = -Q+Φ(Q)
	//
	// If we extend this by merging two iterations, we need to look up P and P'
	// both from {B1, B2, B3, B4} and compute:
	// 		[2]([2]Acc+P)+P' = [4]Acc + T
	// where T = [2]P+P'. So at each (merged) iteration, we can compute [4]Acc
	// and look up T from the precomputed list of points:
	//
	// T = [3](Q + Φ(Q))
	// P = B1 and P' = B1
	T1 := c.Add(tableQ[2], tablePhiQ[2])
	// T = Q + Φ(Q)
	// P = B1 and P' = B2
	T2 := Acc
	// T = [3]Q + Φ(Q)
	// P = B1 and P' = B3
	T3 := c.Add(tableQ[2], tablePhiQ[1])
	// T = Q + [3]Φ(Q)
	// P = B1 and P' = B4
	T4 := c.Add(tableQ[1], tablePhiQ[2])
	// T  = -Q - Φ(Q)
	// P = B2 and P' = B1
	T5 := c.Neg(T2)
	// T  = -[3](Q + Φ(Q))
	// P = B2 and P' = B2
	T6 := c.Neg(T1)
	// T = -Q - [3]Φ(Q)
	// P = B2 and P' = B3
	T7 := c.Neg(T4)
	// T = -[3]Q - Φ(Q)
	// P = B2 and P' = B4
	T8 := c.Neg(T3)
	// T = [3]Q - Φ(Q)
	// P = B3 and P' = B1
	T9 := c.Add(tableQ[2], tablePhiQ[0])
	// T = Q - [3]Φ(Q)
	// P = B3 and P' = B2
	T11 := c.Neg(tablePhiQ[2])
	T10 := c.Add(tableQ[1], T11)
	// T = [3](Q - Φ(Q))
	// P = B3 and P' = B3
	T11 = c.Add(tableQ[2], T11)
	// T = -Φ(Q) + Q
	// P = B3 and P' = B4
	T12 := c.Add(tablePhiQ[0], tableQ[1])
	// T = [3]Φ(Q) - Q
	// P = B4 and P' = B1
	T13 := c.Neg(T10)
	// T = Φ(Q) - [3]Q
	// P = B4 and P' = B2
	T14 := c.Neg(T9)
	// T = Φ(Q) - Q
	// P = B4 and P' = B3
	T15 := c.Neg(T12)
	// T = [3](Φ(Q) - Q)
	// P = B4 and P' = B4
	T16 := c.Neg(T11)
	// note that half the points are negatives of the other half,
	// hence have the same X coordinates.

	// when nbits is odd, we need to handle the first iteration separately
	if nbits%2 == 0 {
		// Acc = [2]Acc ± Q ± Φ(Q)
		T := &AffinePoint[B]{
			X: *c.baseApi.Select(c.api.Xor(s1bits[nbits-1], s2bits[nbits-1]), &T12.X, &T5.X),
			Y: *c.baseApi.Lookup2(s1bits[nbits-1], s2bits[nbits-1], &T5.Y, &T12.Y, &T15.Y, &T2.Y),
		}
		// We don't use doubleAndAdd here as it would involve edge cases
		// when bits are 00 (T==-Acc) or 11 (T==Acc).
		Acc = c.double(Acc)
		Acc = c.add(Acc, T)
	} else {
		// when nbits is even we start the main loop at normally nbits - 1
		nbits++
	}
	for i := nbits - 2; i > 0; i -= 2 {
		// selectorY takes values in [0,15]
		selectorY := c.api.Add(
			s1bits[i],
			c.api.Mul(s2bits[i], 2),
			c.api.Mul(s1bits[i-1], 4),
			c.api.Mul(s2bits[i-1], 8),
		)
		// selectorX takes values in [0,7] s.t.:
		// 		- when selectorY < 8: selectorX = selectorY
		// 		- when selectorY >= 8: selectorX = 15 - selectorY
		selectorX := c.api.Add(
			c.api.Mul(selectorY, c.api.Sub(1, c.api.Mul(s2bits[i-1], 2))),
			c.api.Mul(s2bits[i-1], 15),
		)
		// Bi.Y are distincts so we need a 16-to-1 multiplexer,
		// but only half of the Bi.X are distinct so we need a 8-to-1.
		T := &AffinePoint[B]{
			X: *c.baseApi.Mux(selectorX,
				&T6.X, &T10.X, &T14.X, &T2.X, &T7.X, &T11.X, &T15.X, &T3.X,
			),
			Y: *c.baseApi.Mux(selectorY,
				&T6.Y, &T10.Y, &T14.Y, &T2.Y, &T7.Y, &T11.Y, &T15.Y, &T3.Y,
				&T8.Y, &T12.Y, &T16.Y, &T4.Y, &T5.Y, &T9.Y, &T13.Y, &T1.Y,
			),
		}
		// Acc = [4]Acc + T
		Acc = c.double(Acc)
		Acc = c.doubleAndAdd(Acc, T)
	}

	// i = 0
	// subtract the Q, Φ(Q) if the first bits are 0.
	// When cfg.CompleteArithmetic is set, we use AddUnified instead of Add.
	// This means when s=0 then Acc=(0,0) because AddUnified(Q, -Q) = (0,0).
	tableQ[0] = addFn(tableQ[0], Acc)
	Acc = c.Select(s1bits[0], Acc, tableQ[0])
	tablePhiQ[0] = addFn(tablePhiQ[0], Acc)
	Acc = c.Select(s2bits[0], Acc, tablePhiQ[0])

	if cfg.CompleteArithmetic {
		zero := c.baseApi.Zero()
		Acc = c.Select(selector, &AffinePoint[B]{X: *zero, Y: *zero}, Acc)
	}

	return Acc
}

// scalarMulGeneric computes [s]p and returns it. It doesn't modify p nor s.
// This function doesn't check that the p is on the curve. See AssertIsOnCurve.
//
// ⚠️  p must not be (0,0) and s must not be 0, unless [algopts.WithCompleteArithmetic] option is set.
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
func (c *Curve[B, S]) scalarMulGeneric(p *AffinePoint[B], s *emulated.Element[S], opts ...algopts.AlgebraOption) *AffinePoint[B] {
	cfg, err := algopts.NewConfig(opts...)
	if err != nil {
		panic(fmt.Sprintf("parse opts: %v", err))
	}
	var selector frontend.Variable
	if cfg.CompleteArithmetic {
		// if p=(0,0) we assign a dummy (0,1) to p and continue
		selector = c.api.And(c.baseApi.IsZero(&p.X), c.baseApi.IsZero(&p.Y))
		one := c.baseApi.One()
		p = c.Select(selector, &AffinePoint[B]{X: *one, Y: *one}, p)
	}

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
	// we use AddUnified instead of Add. This is because:
	// 		- when s=0 then R0=P and AddUnified(P, -P) = (0,0). We return (0,0).
	// 		- when s=1 then R0=P AddUnified(Q, -Q) is well defined. We return R0=P.
	R0 = c.Select(sBits[0], R0, c.AddUnified(R0, c.Neg(p)))

	if cfg.CompleteArithmetic {
		// if p=(0,0), return (0,0)
		zero := c.baseApi.Zero()
		R0 = c.Select(selector, &AffinePoint[B]{X: *zero, Y: *zero}, R0)
	}

	return R0
}

// jointScalarMul computes [s1]p1 + [s2]p2 and returns it. It doesn't modify the inputs.
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

// jointScalarMulGeneric computes [s1]p1 + [s2]p2. It doesn't modify p1, p2 nor s1, s2.
//
// ⚠️  The scalars s1, s2 must be nonzero and the point p1, p2 different from (0,0), unless [algopts.WithCompleteArithmetic] option is set.
func (c *Curve[B, S]) jointScalarMulGeneric(p1, p2 *AffinePoint[B], s1, s2 *emulated.Element[S], opts ...algopts.AlgebraOption) *AffinePoint[B] {
	cfg, err := algopts.NewConfig(opts...)
	if err != nil {
		panic(fmt.Sprintf("parse opts: %v", err))
	}
	if cfg.CompleteArithmetic {
		res1 := c.scalarMulGeneric(p1, s1, opts...)
		res2 := c.scalarMulGeneric(p2, s2, opts...)
		return c.AddUnified(res1, res2)
	} else {
		return c.jointScalarMulGenericUnsafe(p1, p2, s1, s2)
	}
}

// jointScalarMulGenericUnsafe computes [s1]p1 + [s2]p2 using Shamir's trick and returns it. It doesn't modify p1, p2 nor s1, s2.
// ⚠️  The scalars must be nonzero and the points different from (0,0).
func (c *Curve[B, S]) jointScalarMulGenericUnsafe(p1, p2 *AffinePoint[B], s1, s2 *emulated.Element[S]) *AffinePoint[B] {
	var Acc, B1, p1Neg, p2Neg *AffinePoint[B]
	p1Neg = c.Neg(p1)
	p2Neg = c.Neg(p2)

	// Acc = P1 + P2
	Acc = c.Add(p1, p2)

	s1bits := c.scalarApi.ToBits(s1)
	s2bits := c.scalarApi.ToBits(s2)

	var st S
	nbits := st.Modulus().BitLen()

	for i := nbits - 1; i > 0; i-- {
		B1 = &AffinePoint[B]{
			X: p1Neg.X,
			Y: *c.baseApi.Select(s1bits[i], &p1.Y, &p1Neg.Y),
		}
		Acc = c.doubleAndAdd(Acc, B1)
		B1 = &AffinePoint[B]{
			X: p2Neg.X,
			Y: *c.baseApi.Select(s2bits[i], &p2.Y, &p2Neg.Y),
		}
		Acc = c.Add(Acc, B1)

	}

	// i = 0
	p1Neg = c.Add(p1Neg, Acc)
	Acc = c.Select(s1bits[0], Acc, p1Neg)
	p2Neg = c.Add(p2Neg, Acc)
	Acc = c.Select(s2bits[0], Acc, p2Neg)

	return Acc
}

// jointScalarMulGLV computes [s1]p1 + [s2]p2 using an endomorphism. It doesn't modify p1, p2 nor s1, s2.
//
// ⚠️  The scalars s1, s2 must be nonzero and the point p1, p2 different from (0,0), unless [algopts.WithCompleteArithmetic] option is set.
func (c *Curve[B, S]) jointScalarMulGLV(p1, p2 *AffinePoint[B], s1, s2 *emulated.Element[S], opts ...algopts.AlgebraOption) *AffinePoint[B] {
	cfg, err := algopts.NewConfig(opts...)
	if err != nil {
		panic(fmt.Sprintf("parse opts: %v", err))
	}
	if cfg.CompleteArithmetic {
		res1 := c.scalarMulGLV(p1, s1, opts...)
		res2 := c.scalarMulGLV(p2, s2, opts...)
		return c.AddUnified(res1, res2)
	} else {
		return c.jointScalarMulGLVUnsafe(p1, p2, s1, s2)
	}
}

// jointScalarMulGLVUnsafe computes [s]Q + [t]R using Shamir's trick with an efficient endomorphism and returns it. It doesn't modify Q, R nor s, t.
// ⚠️  The scalars must be nonzero and the points
//   - ≠ (0,0),
//   - P ≠ ±Q,
func (c *Curve[B, S]) jointScalarMulGLVUnsafe(Q, R *AffinePoint[B], s, t *emulated.Element[S]) *AffinePoint[B] {
	// We use the endomorphism à la GLV to compute [s]Q + [t]R as
	// 		[s1]Q + [s2]Φ(Q) + [t1]R + [t2]Φ(R)
	// the sub-scalars s1, s2, t1, t2 can be negative (bigints) in the hint. If
	// so, they will be reduced in-circuit modulo the SNARK scalar field and
	// not the emulated field. So we return in the hint |s1|, |s2|, |t1|, |t2|
	// and boolean flags sdBits and tdBits to negate the points Q, Φ(Q), R and
	// Φ(R) instead of the corresponding sub-scalars.

	// decompose s into s1 and s2
	sd, err := c.scalarApi.NewHint(decomposeScalarG1Subscalars, 2, s, c.eigenvalue)
	if err != nil {
		// err is non-nil only for invalid number of inputs
		panic(err)
	}
	s1, s2 := sd[0], sd[1]
	sdBits, err := c.scalarApi.NewHintWithNativeOutput(decomposeScalarG1Signs, 2, s, c.eigenvalue)
	if err != nil {
		panic(fmt.Sprintf("compute s GLV decomposition bits: %v", err))
	}
	selector1, selector2 := sdBits[0], sdBits[1]
	s3 := c.scalarApi.Select(selector1, c.scalarApi.Neg(s1), s1)
	s4 := c.scalarApi.Select(selector2, c.scalarApi.Neg(s2), s2)
	// s == s3 + [λ]s4
	c.scalarApi.AssertIsEqual(
		c.scalarApi.Add(s3, c.scalarApi.Mul(s4, c.eigenvalue)),
		s,
	)

	// decompose t into t1 and t2
	td, err := c.scalarApi.NewHint(decomposeScalarG1Subscalars, 2, t, c.eigenvalue)
	if err != nil {
		// err is non-nil only for invalid number of inputs
		panic(err)
	}
	t1, t2 := td[0], td[1]
	tdBits, err := c.scalarApi.NewHintWithNativeOutput(decomposeScalarG1Signs, 2, t, c.eigenvalue)
	if err != nil {
		panic(fmt.Sprintf("compute t GLV decomposition bits: %v", err))
	}
	selector3, selector4 := tdBits[0], tdBits[1]
	t3 := c.scalarApi.Select(selector3, c.scalarApi.Neg(t1), t1)
	t4 := c.scalarApi.Select(selector4, c.scalarApi.Neg(t2), t2)
	// t == t3 + [λ]t4
	c.scalarApi.AssertIsEqual(
		c.scalarApi.Add(t3, c.scalarApi.Mul(t4, c.eigenvalue)),
		t,
	)

	// precompute -Q, -Φ(Q), Φ(Q)
	var tableQ, tablePhiQ [2]*AffinePoint[B]
	negQY := c.baseApi.Neg(&Q.Y)
	tableQ[1] = &AffinePoint[B]{
		X: Q.X,
		Y: *c.baseApi.Select(selector1, negQY, &Q.Y),
	}
	tableQ[0] = c.Neg(tableQ[1])
	tablePhiQ[1] = &AffinePoint[B]{
		X: *c.baseApi.Mul(&Q.X, c.thirdRootOne),
		Y: *c.baseApi.Select(selector2, negQY, &Q.Y),
	}
	tablePhiQ[0] = c.Neg(tablePhiQ[1])

	// precompute -R, -Φ(R), Φ(R)
	var tableR, tablePhiR [2]*AffinePoint[B]
	negRY := c.baseApi.Neg(&R.Y)
	tableR[1] = &AffinePoint[B]{
		X: R.X,
		Y: *c.baseApi.Select(selector3, negRY, &R.Y),
	}
	tableR[0] = c.Neg(tableR[1])
	tablePhiR[1] = &AffinePoint[B]{
		X: *c.baseApi.Mul(&R.X, c.thirdRootOne),
		Y: *c.baseApi.Select(selector4, negRY, &R.Y),
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
	f2 := c.baseApi.Mul(&tableS[2].X, c.thirdRootOne)
	tablePhiS[0] = &AffinePoint[B]{
		X: *c.baseApi.Select(c.api.Xor(selector2, selector4), f2, f0),
		Y: *c.baseApi.Lookup2(selector2, selector4, &tableS[0].Y, &tableS[2].Y, &tableS[3].Y, &tableS[1].Y),
	}
	tablePhiS[1] = c.Neg(tablePhiS[0])
	tablePhiS[2] = &AffinePoint[B]{
		X: *c.baseApi.Select(c.api.Xor(selector2, selector4), f0, f2),
		Y: *c.baseApi.Lookup2(selector2, selector4, &tableS[2].Y, &tableS[0].Y, &tableS[1].Y, &tableS[3].Y),
	}
	tablePhiS[3] = c.Neg(tablePhiS[2])

	// we suppose that the first bits of the sub-scalars are 1 and set:
	// 		Acc = Q + R + Φ(Q) + Φ(R)
	Acc := c.Add(tableS[1], tablePhiS[1])
	B1 := Acc
	// then we conditionally add to Acc either G (the base point) or
	// conditionally Φ²(G) (if Acc==-G) to avoid incomplete additions in the
	// loop, because when doing doubleAndAdd(Acc, Bi) as (Acc+Bi)+Acc it might
	// happen that Acc==Bi or Acc==-Bi. But now we force Acc to be different
	// than the stored Bi.  However we need at the end to subtract [2^nbits]G
	// or conditionally [2^nbits]Φ²(G) from the result.
	//
	// g0 = G
	g0 := c.Generator()
	// g1 = Φ²(G)
	g1 := &AffinePoint[B]{
		X: *c.baseApi.Mul(
			c.baseApi.Mul(&g0.X, c.thirdRootOne), c.thirdRootOne),
		Y: g0.Y,
	}
	selector0 := c.baseApi.IsZero(
		c.baseApi.Add(&Acc.Y, &g0.Y),
	)
	g := c.Select(selector0, g1, g0)
	// Acc = Q + R + Φ(Q) + Φ(R) + G or
	// Q + R + Φ(Q) + Φ(R) + Φ²(G) ( = -G+Φ²(G) = -2G-Φ(G) )
	Acc = c.Add(Acc, g)

	s1bits := c.scalarApi.ToBits(s1)
	s2bits := c.scalarApi.ToBits(s2)
	t1bits := c.scalarApi.ToBits(t1)
	t2bits := c.scalarApi.ToBits(t2)
	var st S
	nbits := st.Modulus().BitLen()>>1 + 2

	// At each iteration we look up the point Bi from:
	// 		B1  = +Q + R + Φ(Q) + Φ(R)
	// 		B2  = +Q + R + Φ(Q) - Φ(R)
	B2 := c.Add(tableS[1], tablePhiS[2])
	// 		B3  = +Q + R - Φ(Q) + Φ(R)
	B3 := c.Add(tableS[1], tablePhiS[3])
	// 		B4  = +Q + R - Φ(Q) - Φ(R)
	B4 := c.Add(tableS[1], tablePhiS[0])
	// 		B5  = +Q - R + Φ(Q) + Φ(R)
	B5 := c.Add(tableS[2], tablePhiS[1])
	// 		B6  = +Q - R + Φ(Q) - Φ(R)
	B6 := c.Add(tableS[2], tablePhiS[2])
	// 		B7  = +Q - R - Φ(Q) + Φ(R)
	B7 := c.Add(tableS[2], tablePhiS[3])
	// 		B8  = +Q - R - Φ(Q) - Φ(R)
	B8 := c.Add(tableS[2], tablePhiS[0])
	// 		B9  = -Q + R + Φ(Q) + Φ(R)
	B9 := c.Neg(B8)
	// 		B10 = -Q + R + Φ(Q) - Φ(R)
	B10 := c.Neg(B7)
	// 		B11 = -Q + R - Φ(Q) + Φ(R)
	B11 := c.Neg(B6)
	// 		B12 = -Q + R - Φ(Q) - Φ(R)
	B12 := c.Neg(B5)
	// 		B13 = -Q - R + Φ(Q) + Φ(R)
	B13 := c.Neg(B4)
	// 		B14 = -Q - R + Φ(Q) - Φ(R)
	B14 := c.Neg(B3)
	// 		B15 = -Q - R - Φ(Q) + Φ(R)
	B15 := c.Neg(B2)
	// 		B16 = -Q - R - Φ(Q) - Φ(R)
	B16 := c.Neg(B1)
	// note that half the points are negatives of the other half,
	// hence have the same X coordinates.

	var Bi *AffinePoint[B]
	for i := nbits - 1; i > 0; i-- {
		// selectorY takes values in [0,15]
		selectorY := c.api.Add(
			s1bits[i],
			c.api.Mul(s2bits[i], 2),
			c.api.Mul(t1bits[i], 4),
			c.api.Mul(t2bits[i], 8),
		)
		// selectorX takes values in [0,7] s.t.:
		// 		- when selectorY < 8: selectorX = selectorY
		// 		- when selectorY >= 8: selectorX = 15 - selectorY
		selectorX := c.api.Add(
			c.api.Mul(selectorY, c.api.Sub(1, c.api.Mul(t2bits[i], 2))),
			c.api.Mul(t2bits[i], 15),
		)
		// Bi.Y are distincts so we need a 16-to-1 multiplexer,
		// but only half of the Bi.X are distinct so we need a 8-to-1.
		Bi = &AffinePoint[B]{
			X: *c.baseApi.Mux(selectorX,
				&B16.X, &B8.X, &B14.X, &B6.X, &B12.X, &B4.X, &B10.X, &B2.X,
			),
			Y: *c.baseApi.Mux(selectorY,
				&B16.Y, &B8.Y, &B14.Y, &B6.Y, &B12.Y, &B4.Y, &B10.Y, &B2.Y,
				&B15.Y, &B7.Y, &B13.Y, &B5.Y, &B11.Y, &B3.Y, &B9.Y, &B1.Y,
			),
		}
		// Acc = [2]Acc + Bi
		Acc = c.doubleAndAdd(Acc, Bi)
	}

	// i = 0
	// subtract the Q, R, Φ(Q), Φ(R) if the first bits are 0
	tableQ[0] = c.Add(tableQ[0], Acc)
	Acc = c.Select(s1bits[0], Acc, tableQ[0])
	tablePhiQ[0] = c.Add(tablePhiQ[0], Acc)
	Acc = c.Select(s2bits[0], Acc, tablePhiQ[0])
	tableR[0] = c.Add(tableR[0], Acc)
	Acc = c.Select(t1bits[0], Acc, tableR[0])
	tablePhiR[0] = c.Add(tablePhiR[0], Acc)
	Acc = c.Select(t2bits[0], Acc, tablePhiR[0])

	// subtract [2^nbits]G or conditionally [2^nbits]Φ²(G)
	gm := c.GeneratorMultiples()[nbits-1]
	g = c.Select(
		selector0,
		// [2^nbits]Φ²(G)
		&AffinePoint[B]{
			X: *c.baseApi.Mul(
				c.baseApi.Mul(&gm.X, c.thirdRootOne), c.thirdRootOne),
			Y: gm.Y,
		},
		// [2^nbits]G
		&gm,
	)
	Acc = c.Add(Acc, c.Neg(g))

	return Acc

}

// ScalarMulBase computes [s]g and returns it where g is the fixed curve generator. It doesn't modify p nor s.
//
// ScalarMul calls scalarMulBaseGeneric or scalarMulGLV depending on whether an efficient endomorphism is available.
func (c *Curve[B, S]) ScalarMulBase(s *emulated.Element[S], opts ...algopts.AlgebraOption) *AffinePoint[B] {
	if c.eigenvalue != nil && c.thirdRootOne != nil {
		return c.scalarMulGLV(c.Generator(), s, opts...)

	} else {
		return c.scalarMulBaseGeneric(s, opts...)

	}
}

// scalarMulBaseGeneric computes [s]g and returns it, where g is the fixed generator.
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
func (c *Curve[B, S]) scalarMulBaseGeneric(s *emulated.Element[S], opts ...algopts.AlgebraOption) *AffinePoint[B] {
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
	// When cfg.CompleteArithmetic is set, we use AddUnified instead of Add. This means
	// when s=0 then Acc=(0,0) because AddUnified(Q, -Q) = (0,0).
	addFn := c.Add
	if cfg.CompleteArithmetic {
		addFn = c.AddUnified
	}
	tmp := addFn(res, c.Neg(g))
	res = c.Select(sBits[0], res, tmp)

	return res
}

// JointScalarMulBase computes [s1]g + [s2]p and returns it, where g is the
// fixed generator. It doesn't modify p, s1 and s2.
//
// ⚠️   p must NOT be (0,0),
// ⚠️   p must NOT be ±g,
// ⚠️   s1 and s2 must NOT be 0.
//
// JointScalarMulBase is used to verify an ECDSA signature (r,s) for example on
// the secp256k1 curve. In this case, p is a public key, s2=r/s and s1=hash/s.
//   - hash cannot be 0, because of pre-image resistance.
//   - r cannot be 0, because r is the x coordinate of a random point on
//     secp256k1 (y²=x³+7 mod p) and 7 is not a square mod p. For any other
//     curve, (_,0) is a point of order 2 which is not the prime subgroup.
//   - (0,0) is not a valid public key.
//
// The [EVM] specifies these checks, wich are performed on the zkEVM
// arithmetization side before calling the circuit that uses this method.
func (c *Curve[B, S]) JointScalarMulBase(p *AffinePoint[B], s2, s1 *emulated.Element[S], opts ...algopts.AlgebraOption) *AffinePoint[B] {
	return c.jointScalarMul(c.Generator(), p, s1, s2, opts...)
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
	addFn := c.Add
	if cfg.CompleteArithmetic {
		addFn = c.AddUnified
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
			res = addFn(res, q)
		}
		return res, nil
	} else {
		// scalars are powers
		if len(s) == 0 {
			return nil, fmt.Errorf("need scalar for folding")
		}
		gamma := s[0]
		res := c.ScalarMul(p[len(p)-1], gamma, opts...)
		for i := len(p) - 2; i > 0; i-- {
			res = addFn(p[i], res)
			res = c.ScalarMul(res, gamma, opts...)
		}
		res = addFn(p[0], res)
		return res, nil
	}
}
