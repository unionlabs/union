package g2

import (
	"math/big"

	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fp"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/algebra/emulated/fields_bn254"
	gadget "github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	"github.com/consensys/gnark/std/math/emulated"
)

// Constant not public in gnark crypto...
var bCurveCoeff fp.Element

// twist
var twist bn254.E2

// bTwistCurveCoeff b coeff of the twist (defined over 𝔽p²) curve
var bTwistCurveCoeff bn254.E2
var B fields_bn254.E2

func init() {
	bCurveCoeff.SetUint64(3)
	// D-twist
	twist.A0.SetUint64(9)
	twist.A1.SetUint64(1)
	bTwistCurveCoeff.Inverse(&twist).MulByElement(&bTwistCurveCoeff, &bCurveCoeff)

	B = fields_bn254.FromE2(&bTwistCurveCoeff)

	solver.RegisterHint(hintSqrt)
	solver.RegisterHint(hintLegendre)

}

// Caller must ensure the root exists by calling legendre
func hintSqrt(nativeMod *big.Int, nativeInputs, nativeOutputs []*big.Int) error {
	return emulated.UnwrapHint(nativeInputs, nativeOutputs,
		func(mod *big.Int, inputs, outputs []*big.Int) error {
			var a, c bn254.E2

			a.A0.SetBigInt(inputs[0])
			a.A1.SetBigInt(inputs[1])

			c.Sqrt(&a)

			c.A0.BigInt(outputs[0])
			c.A1.BigInt(outputs[1])

			return nil
		})
}

// Hint legendre, caller must check that the result is valid, i.e. sqrt and verify root or no root
func hintLegendre(nativeMod *big.Int, nativeInputs, nativeOutputs []*big.Int) error {
	return emulated.UnwrapHint(nativeInputs, nativeOutputs,
		func(mod *big.Int, inputs, outputs []*big.Int) error {
			var a bn254.E2

			a.A0.SetBigInt(inputs[0])
			a.A1.SetBigInt(inputs[1])

			x := a.Legendre()

			if x == -1 {
				x = 0
			}

			outputs[0] = big.NewInt(int64(x))

			return nil
		})
}

type EmulatedAPI struct {
	api    frontend.API
	field  *emulated.Field[emulated.BN254Fp]
	fieldR *emulated.Field[emulated.BN254Fr]
	ext2   *fields_bn254.Ext2
	u, v   *fields_bn254.E2
}

func NewEmulatedAPI(api frontend.API) (*EmulatedAPI, error) {
	field, err := emulated.NewField[emulated.BN254Fp](api)
	if err != nil {
		return nil, err
	}
	fieldR, err := emulated.NewField[emulated.BN254Fr](api)
	if err != nil {
		return nil, err
	}
	u := fields_bn254.E2{
		A0: emulated.ValueOf[emulated.BN254Fp]("21575463638280843010398324269430826099269044274347216827212613867836435027261"),
		A1: emulated.ValueOf[emulated.BN254Fp]("10307601595873709700152284273816112264069230130616436755625194854815875713954"),
	}
	v := fields_bn254.E2{
		A0: emulated.ValueOf[emulated.BN254Fp]("2821565182194536844548159561693502659359617185244120367078079554186484126554"),
		A1: emulated.ValueOf[emulated.BN254Fp]("3505843767911556378687030309984248845540243509899259641013678093033130930403"),
	}
	ext2 := fields_bn254.NewExt2(api)
	return &EmulatedAPI{
		api:    api,
		field:  field,
		fieldR: fieldR,
		ext2:   ext2,
		u:      &u,
		v:      &v,
	}, nil
}

// AssertIsEqual asserts that p and q are the same point.
func (e *EmulatedAPI) AssertIsEqual(p, q *gadget.G2Affine) {
	e.ext2.AssertIsEqual(&p.X, &q.X)
	e.ext2.AssertIsEqual(&p.Y, &q.Y)
}

// https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#name-the-sgn0-function
// 	sgn0_m_eq_2(x)

// 	Input: x, an element of GF(p^2).
// 		Output: 0 or 1.

// Steps:
//  1. sign_0 = x_0 mod 2
//  2. zero_0 = x_0 == 0
//  3. sign_1 = x_1 mod 2
//  4. s = sign_0 OR (zero_0 AND sign_1) # Avoid short-circuit logic ops
//  5. return s
func (e *EmulatedAPI) g2Sgn0Circuit(z *fields_bn254.E2) frontend.Variable {
	a0b := e.field.ToBits(&z.A0)

	sign_0 := a0b[0]
	zero_0 := e.field.IsZero(&z.A0)

	a1b := e.field.ToBits(&z.A1)

	sign_1 := a1b[0]
	sign := e.api.Or(sign_0, e.api.And(zero_0, sign_1))

	return sign
}

// Shallue-van de Woestijne method
// https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#name-shallue-van-de-woestijne-met
func (e *EmulatedAPI) MapToCurve(u *fields_bn254.E2) *gadget.G2Affine {
	// Legendre must be called before calling sqrt
	sqrt := func(x *fields_bn254.E2) *fields_bn254.E2 {
		roots, err := e.field.NewHint(hintSqrt, 2, &x.A0, &x.A1)
		if err != nil {
			panic(err)
		}
		root := &fields_bn254.E2{
			A0: *roots[0],
			A1: *roots[1],
		}
		// Ensure valid root
		e.ext2.AssertIsEqual(x, e.ext2.Square(root))
		return root
	}

	legendre := func(x *fields_bn254.E2) (frontend.Variable, *fields_bn254.E2) {
		legendres, err := e.field.NewHint(hintLegendre, 1, &x.A0, &x.A1)
		if err != nil {
			panic(err)
		}
		legendre := legendres[0].Limbs[0]
		roots, err := e.field.NewHint(hintSqrt, 2, &x.A0, &x.A1)
		if err != nil {
			panic(err)
		}
		root := &fields_bn254.E2{
			A0: *roots[0],
			A1: *roots[1],
		}
		// Ensure valid legendre
		e.api.AssertIsBoolean(legendre)
		// Ensure valid branch
		e.ext2.AssertIsEqual(x, e.ext2.Select(legendre, e.ext2.Square(root), x))
		// TODO assert root^2 != x if legendre == 0
		return legendre, root
	}

	var tv1, tv2, tv3, tv4 *fields_bn254.E2
	var x1, x2, x3, gx1, gx2, gx, x, y *fields_bn254.E2
	var one *fields_bn254.E2

	//constants
	//c1 = g(Z)
	//c2 = -Z / 2
	//c3 = sqrt(-g(Z) * (3 * Z² + 4 * A))     # sgn0(c3) MUST equal 0
	//c4 = -4 * g(Z) / (3 * Z² + 4 * A)

	Z := fields_bn254.FromE2(&bn254.E2{
		A0: fp.Element{15230403791020821917, 754611498739239741, 7381016538464732716, 1011752739694698287},
		A1: fp.Element{0},
	})
	c1 := fields_bn254.FromE2(&bn254.E2{
		A0: fp.Element{15219334786797146878, 8431472696017589261, 15336528771359260718, 196732871012706162},
		A1: fp.Element{4100506350182530919, 7345568344173317438, 15513160039642431658, 90557763186888013},
	})
	c2 := fields_bn254.FromE2(&bn254.E2{
		A0: fp.Element{12997850613838968789, 14304628359724097447, 2950087706404981016, 1237622763554136189},
		A1: fp.Element{0},
	})
	c3 := fields_bn254.FromE2(&bn254.E2{
		A0: fp.Element{12298500088583694207, 17447120171744064890, 14097510924717921191, 2278398337453771183},
		A1: fp.Element{4693446565795584099, 18320164443970680666, 6792758484113206563, 2989688171181581768},
	})
	c4 := fields_bn254.FromE2(&bn254.E2{
		A0: fp.Element{7191623630069643826, 8333948550768170742, 13001081703983517696, 2062355016518372226},
		A1: fp.Element{11163104453509316115, 7271947710149976975, 4894807947557820282, 3366254582553786647},
	})

	one = e.ext2.One()

	// 1.  tv1 = u^2
	tv1 = e.ext2.Square(u)
	// 2.  tv1 = tv1 * c1
	tv1 = e.ext2.Mul(tv1, &c1)
	// 3.  tv2 = 1 + tv1
	tv2 = e.ext2.Add(one, tv1)
	// 4.  tv1 = 1 - tv1
	tv1 = e.ext2.Sub(one, tv1)
	// 5.  tv3 = tv1 * tv2
	tv3 = e.ext2.Mul(tv1, tv2)
	// 6.  tv3 = inv0(tv3)
	tv3 = e.ext2.Inverse(tv3)
	// 7.  tv4 = u * tv1
	tv4 = e.ext2.Mul(u, tv1)
	// 8.  tv4 = tv4 * tv3
	tv4 = e.ext2.Mul(tv4, tv3)
	// 9.  tv4 = tv4 * c3
	tv4 = e.ext2.Mul(tv4, &c3)
	// 10.  x1 = c2 - tv4
	x1 = e.ext2.Sub(&c2, tv4)
	// 11. gx1 = x1^2
	gx1 = e.ext2.Square(x1)
	// 12. gx1 = gx1 + A
	// !!! NOOP !!!
	// 13. gx1 = gx1 * x1
	gx1 = e.ext2.Mul(gx1, x1)
	// 14. gx1 = gx1 + B
	gx1 = e.ext2.Add(gx1, &B)
	// 15.  e1 = is_square(gx1)
	e1, _ := legendre(gx1)
	// 16.  x2 = c2 + tv4
	x2 = e.ext2.Add(&c2, tv4)
	// 17. gx2 = x2^2
	gx2 = e.ext2.Square(x2)
	// 18. gx2 = gx2 + A
	// !!! NOOP !!!
	// 19. gx2 = gx2 * x2
	gx2 = e.ext2.Mul(gx2, x2)
	// 20. gx2 = gx2 + B
	gx2 = e.ext2.Add(gx2, &B)
	// 21.  e2 = is_square(gx2) AND NOT e1   # Avoid short-circuit logic ops
	gx2Square, _ := legendre(gx2)
	e2 := e.api.And(gx2Square, e.api.Select(e1, 0, 1))
	// 22.  x3 = tv2^2
	x3 = e.ext2.Square(tv2)
	// 23.  x3 = x3 * tv3
	x3 = e.ext2.Mul(x3, tv3)
	// 24.  x3 = x3^2
	x3 = e.ext2.Square(x3)
	// 25.  x3 = x3 * c4
	x3 = e.ext2.Mul(x3, &c4)
	// 26.  x3 = x3 + Z
	x3 = e.ext2.Add(x3, &Z)
	// 27.   x = CMOV(x3, x1, e1)   # x = x1 if gx1 is square, else x = x3
	x = e.ext2.Select(e1, x1, x3)
	// 28.   x = CMOV(x, x2, e2)    # x = x2 if gx2 is square and gx1 is not
	x = e.ext2.Select(e2, x2, x)
	// 29.  gx = x^2
	gx = e.ext2.Square(x)
	// 30.  gx = gx + A
	// !!! NOOP !!!
	// 31.  gx = gx * x
	gx = e.ext2.Mul(gx, x)
	// 32.  gx = gx + B
	gx = e.ext2.Add(gx, &B)
	// 33.   y = sqrt(gx)
	y = sqrt(gx)
	// 34.  e3 = sgn0(u) == sgn0(y)
	e3 := e.api.IsZero(e.api.Xor(e.g2Sgn0Circuit(u), e.g2Sgn0Circuit(y)))
	// 35.   y = CMOV(-y, y, e3)       # Select correct sign of y
	y = e.ext2.Select(e3, y, e.ext2.Neg(y))
	// 36. return (x, y)
	return &gadget.G2Affine{X: *x, Y: *y}
}

func (e *EmulatedAPI) Neg(p *gadget.G2Affine) *gadget.G2Affine {
	return &gadget.G2Affine{
		X: p.X,
		Y: *e.ext2.Neg(&p.Y),
	}
}

func (e *EmulatedAPI) Select(b frontend.Variable, p, q *gadget.G2Affine) *gadget.G2Affine {
	x := e.ext2.Select(b, &p.X, &q.X)
	y := e.ext2.Select(b, &p.Y, &q.Y)
	return &gadget.G2Affine{
		X: *x,
		Y: *y,
	}
}

func (e *EmulatedAPI) Add(p, q *gadget.G2Affine) *gadget.G2Affine {
	// compute λ = (q.y-p.y)/(q.x-p.x)
	qypy := e.ext2.Sub(&q.Y, &p.Y)
	qxpx := e.ext2.Sub(&q.X, &p.X)
	λ := e.ext2.DivUnchecked(qypy, qxpx)

	// xr = λ²-p.x-q.x
	λλ := e.ext2.Square(λ)
	qxpx = e.ext2.Add(&p.X, &q.X)
	xr := e.ext2.Sub(λλ, qxpx)

	// p.y = λ(p.x-r.x) - p.y
	pxrx := e.ext2.Sub(&p.X, xr)
	λpxrx := e.ext2.Mul(λ, pxrx)
	yr := e.ext2.Sub(λpxrx, &p.Y)

	return &gadget.G2Affine{
		X: *xr,
		Y: *yr,
	}
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
func (e *EmulatedAPI) AddUnified(p, q *gadget.G2Affine) *gadget.G2Affine {

	// selector1 = 1 when p is (0,0) and 0 otherwise
	selector1 := e.api.And(e.ext2.IsZero(&p.X), e.ext2.IsZero(&p.Y))
	// selector2 = 1 when q is (0,0) and 0 otherwise
	selector2 := e.api.And(e.ext2.IsZero(&q.X), e.ext2.IsZero(&q.Y))

	// λ = ((p.x+q.x)² - p.x*q.x + a)/(p.y + q.y)
	pxqx := e.ext2.Mul(&p.X, &q.X)
	pxplusqx := e.ext2.Add(&p.X, &q.X)
	num := e.ext2.Mul(pxplusqx, pxplusqx)
	num = e.ext2.Sub(num, pxqx)

	// BN254 specialization
	// if c.addA {
	// 	num = e.ext2.Add(num, &c.a)
	// }

	denum := e.ext2.Add(&p.Y, &q.Y)
	// if p.y + q.y = 0, assign dummy 1 to denum and continue
	selector3 := e.ext2.IsZero(denum)
	denum = e.ext2.Select(selector3, e.ext2.One(), denum)
	λ := e.ext2.DivUnchecked(num, denum)

	// x = λ^2 - p.x - q.x
	xr := e.ext2.Mul(λ, λ)
	xr = e.ext2.Sub(xr, pxplusqx)

	// y = λ(p.x - xr) - p.y
	yr := e.ext2.Sub(&p.X, xr)
	yr = e.ext2.Mul(yr, λ)
	yr = e.ext2.Sub(yr, &p.Y)
	result := gadget.G2Affine{
		X: *xr,
		Y: *yr,
	}

	zero := e.ext2.Zero()
	infinity := gadget.G2Affine{X: *zero, Y: *zero}
	// if p=(0,0)
	result = *e.Select(selector1, q, &result)
	// if q=(0,0) return p
	result = *e.Select(selector2, p, &result)
	// if p.y + q.y = 0, return (0, 0)
	result = *e.Select(selector3, &infinity, &result)

	return &result
}

// DoubleAndAdd computes 2p+q as (p+q)+p. It follows [ELM03] (Section 3.1)
// Saves the computation of the y coordinate of p+q as it is used only in the computation of λ2,
// which can be computed as
//
//	λ2 = -λ1-2*p.y/(x2-p.x)
//
// instead. It doesn't modify p nor q.
//
// [ELM03]: https://arxiv.org/pdf/math/0208038.pdf
func (e *EmulatedAPI) DoubleAndAdd(p, q *gadget.G2Affine) *gadget.G2Affine {

	// compute λ1 = (q.y-p.y)/(q.x-p.x)
	yqyp := e.ext2.Sub(&q.Y, &p.Y)
	xqxp := e.ext2.Sub(&q.X, &p.X)
	λ1 := e.ext2.DivUnchecked(yqyp, xqxp)

	// compute x2 = λ1²-p.x-q.x
	λ1λ1 := e.ext2.Square(λ1)
	xqxp = e.ext2.Add(&p.X, &q.X)
	x2 := e.ext2.Sub(λ1λ1, xqxp)

	// omit y2 computation
	// compute λ2 = -λ1-2*p.y/(x2-p.x)
	ypyp := e.ext2.Double(&p.Y)
	x2xp := e.ext2.Sub(x2, &p.X)
	λ2 := e.ext2.DivUnchecked(ypyp, x2xp)
	λ2 = e.ext2.Add(λ1, λ2)
	λ2 = e.ext2.Neg(λ2)

	// compute x3 =λ2²-p.x-x3
	λ2λ2 := e.ext2.Square(λ2)
	x3 := e.ext2.Sub(λ2λ2, &p.X)
	x3 = e.ext2.Sub(x3, x2)

	// compute y3 = λ2*(p.x - x3)-p.y
	y3 := e.ext2.Sub(&p.X, x3)
	y3 = e.ext2.Mul(λ2, y3)
	y3 = e.ext2.Sub(y3, &p.Y)

	return &gadget.G2Affine{
		X: *x3,
		Y: *y3,
	}

}

// Double doubles p and return it. It doesn't modify p.
// It uses affine coordinates.
func (e *EmulatedAPI) Double(p *gadget.G2Affine) *gadget.G2Affine {
	// compute λ = (3p.x²+a)/2*p.y, here we assume a=0 (j invariant 0 curve)
	xx3a := e.ext2.Square(&p.X)
	xx3a = e.ext2.MulByConstElement(xx3a, big.NewInt(3))
	y2 := e.ext2.MulByConstElement(&p.Y, big.NewInt(2))
	λ := e.ext2.DivUnchecked(xx3a, y2)

	// xr = λ²-2p.x
	x2 := e.ext2.MulByConstElement(&p.X, big.NewInt(2))
	λλ := e.ext2.Square(λ)
	xr := e.ext2.Sub(λλ, x2)

	// yr = λ(p-xr) - p.y
	pxrx := e.ext2.Sub(&p.X, xr)
	λpxrx := e.ext2.Mul(λ, pxrx)
	yr := e.ext2.Sub(λpxrx, &p.Y)

	return &gadget.G2Affine{
		X: *xr,
		Y: *yr,
	}
}

func (e *EmulatedAPI) Triple(p *gadget.G2Affine) *gadget.G2Affine {

	// compute λ1 = (3p.x²+a)/2p.y, here we assume a=0 (j invariant 0 curve)
	xx := e.ext2.Square(&p.X)
	xx = e.ext2.MulByConstElement(xx, big.NewInt(3))
	y2 := e.ext2.Double(&p.Y)
	λ1 := e.ext2.DivUnchecked(xx, y2)

	// xr = λ1²-2p.x
	x2 := e.ext2.Double(&p.X)
	λ1λ1 := e.ext2.Mul(λ1, λ1)
	x2 = e.ext2.Sub(λ1λ1, x2)

	// omit y2 computation, and
	// compute λ2 = 2p.y/(x2 − p.x) − λ1.
	x1x2 := e.ext2.Sub(&p.X, x2)
	λ2 := e.ext2.DivUnchecked(y2, x1x2)
	λ2 = e.ext2.Sub(λ2, λ1)

	// xr = λ²-p.x-x2
	λ2λ2 := e.ext2.Mul(λ2, λ2)
	qxrx := e.ext2.Add(x2, &p.X)
	xr := e.ext2.Sub(λ2λ2, qxrx)

	// yr = λ(p.x-xr) - p.y
	pxrx := e.ext2.Sub(&p.X, xr)
	λ2pxrx := e.ext2.Mul(λ2, pxrx)
	yr := e.ext2.Sub(λ2pxrx, &p.Y)

	return &gadget.G2Affine{
		X: *xr,
		Y: *yr,
	}
}

func (e *EmulatedAPI) DoubleN(p *gadget.G2Affine, n int) *gadget.G2Affine {
	pn := p
	for s := 0; s < n; s++ {
		pn = e.Double(pn)
	}
	return pn
}

func (e *EmulatedAPI) Psi(q *gadget.G2Affine) *gadget.G2Affine {
	x := e.ext2.Conjugate(&q.X)
	x = e.ext2.Mul(x, e.u)
	y := e.ext2.Conjugate(&q.Y)
	y = e.ext2.Mul(y, e.v)
	return &gadget.G2Affine{
		X: *x,
		Y: *y,
	}
}

func (e *EmulatedAPI) ScalarMulBySeed(q *gadget.G2Affine) *gadget.G2Affine {
	z := e.Double(q)
	t0 := e.Add(q, z)
	t2 := e.Add(q, t0)
	t1 := e.Add(z, t2)
	z = e.DoubleAndAdd(t1, t0)
	t0 = e.Add(t0, z)
	t2 = e.Add(t2, t0)
	t1 = e.Add(t1, t2)
	t0 = e.Add(t0, t1)
	t1 = e.Add(t1, t0)
	t0 = e.Add(t0, t1)
	t2 = e.Add(t2, t0)
	t1 = e.DoubleAndAdd(t2, t1)
	t2 = e.Add(t2, t1)
	z = e.Add(z, t2)
	t2 = e.Add(t2, z)
	z = e.DoubleAndAdd(t2, z)
	t0 = e.Add(t0, z)
	t1 = e.Add(t1, t0)
	t3 := e.Double(t1)
	t3 = e.DoubleAndAdd(t3, t1)
	t2 = e.Add(t2, t3)
	t1 = e.Add(t1, t2)
	t2 = e.Add(t2, t1)
	t2 = e.DoubleN(t2, 16)
	t1 = e.DoubleAndAdd(t2, t1)
	t1 = e.DoubleN(t1, 13)
	t0 = e.DoubleAndAdd(t1, t0)
	t0 = e.DoubleN(t0, 15)
	z = e.DoubleAndAdd(t0, z)

	return z
}

func (e *EmulatedAPI) ClearCofactor(p *gadget.G2Affine) *gadget.G2Affine {
	p0 := e.ScalarMulBySeed(p)
	p1 := e.Psi(e.Add(e.Double(p0), p0))
	p2 := e.Psi(e.Psi(p0))
	p3 := e.Psi(e.Psi(e.Psi(p)))
	return e.Add(e.Add(e.Add(p0, p1), p2), p3)
}

func (e *EmulatedAPI) HashToG2(u *fields_bn254.E2) *gadget.G2Affine {
	return e.ClearCofactor(e.MapToCurve(u))
}
