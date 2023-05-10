package g2

import (
	"fmt"
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
	solver.RegisterHint(hintDebug)
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

func hintDebug(nativeMod *big.Int, nativeInputs, nativeOutputs []*big.Int) error {
	return emulated.UnwrapHint(nativeInputs, nativeOutputs,
		func(mod *big.Int, inputs, outputs []*big.Int) error {
			var a bn254.E2

			a.A0.SetBigInt(inputs[0])
			a.A1.SetBigInt(inputs[1])

			fmt.Println("P = ", a)

			return nil
		})
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
func g2Sgn0Circuit(api frontend.API, z *fields_bn254.E2) frontend.Variable {
	field, err := emulated.NewField[emulated.BN254Fp](api)
	if err != nil {
		panic(err)
	}

	a0b := field.ToBits(&z.A0)

	sign_0 := a0b[0]
	zero_0 := field.IsZero(&z.A0)

	a1b := field.ToBits(&z.A1)

	sign_1 := a1b[0]
	sign := api.Or(sign_0, api.And(zero_0, sign_1))

	return sign
}

// Shallue-van de Woestijne method
// https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#name-shallue-van-de-woestijne-met
func MapToCurve(api frontend.API, u *fields_bn254.E2) *gadget.G2Affine {
	field, err := emulated.NewField[emulated.BN254Fp](api)
	if err != nil {
		panic(err)
	}

	e := fields_bn254.NewExt2(api)

	// Legendre must be called before calling sqrt
	sqrt := func(x *fields_bn254.E2) *fields_bn254.E2 {
		roots, err := field.NewHint(hintSqrt, 2, &x.A0, &x.A1)
		if err != nil {
			panic(err)
		}
		root := &fields_bn254.E2{
			A0: *roots[0],
			A1: *roots[1],
		}
		// Ensure valid root
		e.AssertIsEqual(x, e.Square(root))
		return root
	}

	legendre := func(x *fields_bn254.E2) (frontend.Variable, *fields_bn254.E2) {
		legendres, err := field.NewHint(hintLegendre, 1, &x.A0, &x.A1)
		if err != nil {
			panic(err)
		}
		legendre := legendres[0].Limbs[0]
		roots, err := field.NewHint(hintSqrt, 2, &x.A0, &x.A1)
		if err != nil {
			panic(err)
		}
		root := &fields_bn254.E2{
			A0: *roots[0],
			A1: *roots[1],
		}
		// Ensure valid legendre
		api.AssertIsBoolean(legendre)
		// Ensure valid branch
		e.AssertIsEqual(x, e.Select(legendre, e.Square(root), x))
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

	one = e.One()

	// 1.  tv1 = u^2
	tv1 = e.Square(u)
	// 2.  tv1 = tv1 * c1
	tv1 = e.Mul(tv1, &c1)
	// 3.  tv2 = 1 + tv1
	tv2 = e.Add(one, tv1)
	// 4.  tv1 = 1 - tv1
	tv1 = e.Sub(one, tv1)
	// 5.  tv3 = tv1 * tv2
	tv3 = e.Mul(tv1, tv2)
	// 6.  tv3 = inv0(tv3)
	tv3 = e.Inverse(tv3)
	// 7.  tv4 = u * tv1
	tv4 = e.Mul(u, tv1)
	// 8.  tv4 = tv4 * tv3
	tv4 = e.Mul(tv4, tv3)
	// 9.  tv4 = tv4 * c3
	tv4 = e.Mul(tv4, &c3)
	// 10.  x1 = c2 - tv4
	x1 = e.Sub(&c2, tv4)
	// 11. gx1 = x1^2
	gx1 = e.Square(x1)
	// 12. gx1 = gx1 + A
	// !!! NOOP !!!
	// 13. gx1 = gx1 * x1
	gx1 = e.Mul(gx1, x1)
	// 14. gx1 = gx1 + B
	gx1 = e.Add(gx1, &B)
	// 15.  e1 = is_square(gx1)
	e1, _ := legendre(gx1)
	// 16.  x2 = c2 + tv4
	x2 = e.Add(&c2, tv4)
	// 17. gx2 = x2^2
	gx2 = e.Square(x2)
	// 18. gx2 = gx2 + A
	// !!! NOOP !!!
	// 19. gx2 = gx2 * x2
	gx2 = e.Mul(gx2, x2)
	// 20. gx2 = gx2 + B
	gx2 = e.Add(gx2, &B)
	// 21.  e2 = is_square(gx2) AND NOT e1   # Avoid short-circuit logic ops
	gx2Square, _ := legendre(gx2)
	e2 := api.And(gx2Square, api.Select(e1, 0, 1))
	// 22.  x3 = tv2^2
	x3 = e.Square(tv2)
	// 23.  x3 = x3 * tv3
	x3 = e.Mul(x3, tv3)
	// 24.  x3 = x3^2
	x3 = e.Square(x3)
	// 25.  x3 = x3 * c4
	x3 = e.Mul(x3, &c4)
	// 26.  x3 = x3 + Z
	x3 = e.Add(x3, &Z)
	// 27.   x = CMOV(x3, x1, e1)   # x = x1 if gx1 is square, else x = x3
	x = e.Select(e1, x1, x3)
	// 28.   x = CMOV(x, x2, e2)    # x = x2 if gx2 is square and gx1 is not
	x = e.Select(e2, x2, x)
	// 29.  gx = x^2
	gx = e.Square(x)
	// 30.  gx = gx + A
	// !!! NOOP !!!
	// 31.  gx = gx * x
	gx = e.Mul(gx, x)
	// 32.  gx = gx + B
	gx = e.Add(gx, &B)
	// 33.   y = sqrt(gx)
	y = sqrt(gx)
	// 34.  e3 = sgn0(u) == sgn0(y)
	e3 := api.IsZero(api.Xor(g2Sgn0Circuit(api, u), g2Sgn0Circuit(api, y)))
	// 35.   y = CMOV(-y, y, e3)       # Select correct sign of y
	y = e.Select(e3, y, e.Neg(y))
	// 36. return (x, y)
	return &gadget.G2Affine{X: *x, Y: *y}
}

func Neg(ba *fields_bn254.Ext2, p *gadget.G2Affine) *gadget.G2Affine {
	return &gadget.G2Affine{
		X: p.X,
		Y: *ba.Neg(&p.Y),
	}
}

func Select(ba *fields_bn254.Ext2, b frontend.Variable, p, q *gadget.G2Affine) *gadget.G2Affine {
	x := ba.Select(b, &p.X, &q.X)
	y := ba.Select(b, &p.Y, &q.Y)
	return &gadget.G2Affine{
		X: *x,
		Y: *y,
	}
}

// Add adds p and q and returns it. It doesn't modify p nor q.
// It uses incomplete formulas in affine coordinates.
// The points p and q should be different and nonzero (neutral element).
func Add(ba *fields_bn254.Ext2, p, q *gadget.G2Affine) *gadget.G2Affine {
	// compute λ = (q.y-p.y)/(q.x-p.x)
	qypy := ba.Sub(&q.Y, &p.Y)
	qxpx := ba.Sub(&q.X, &p.X)
	λ := ba.DivUnchecked(qypy, qxpx)

	// xr = λ²-p.x-q.x
	λλ := ba.Square(λ)
	qxpx = ba.Add(&p.X, &q.X)
	xr := ba.Sub(λλ, qxpx)

	// p.y = λ(p.x-r.x) - p.y
	pxrx := ba.Sub(&p.X, xr)
	λpxrx := ba.Mul(λ, pxrx)
	yr := ba.Sub(λpxrx, &p.Y)

	return &gadget.G2Affine{
		X: *xr,
		Y: *yr,
	}
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
func DoubleAndAdd(ba *fields_bn254.Ext2, p, q *gadget.G2Affine) *gadget.G2Affine {

	// compute λ1 = (q.y-p.y)/(q.x-p.x)
	yqyp := ba.Sub(&q.Y, &p.Y)
	xqxp := ba.Sub(&q.X, &p.X)
	λ1 := ba.DivUnchecked(yqyp, xqxp)

	// compute x2 = λ1²-p.x-q.x
	λ1λ1 := ba.Square(λ1)
	xqxp = ba.Add(&p.X, &q.X)
	x2 := ba.Sub(λ1λ1, xqxp)

	// ommit y2 computation
	// compute λ2 = -λ1-2*p.y/(x2-p.x)
	ypyp := ba.Double(&p.Y)
	x2xp := ba.Sub(x2, &p.X)
	λ2 := ba.DivUnchecked(ypyp, x2xp)
	λ2 = ba.Add(λ1, λ2)
	λ2 = ba.Neg(λ2)

	// compute x3 =λ2²-p.x-x3
	λ2λ2 := ba.Square(λ2)
	x3 := ba.Sub(λ2λ2, &p.X)
	x3 = ba.Sub(x3, x2)

	// compute y3 = λ2*(p.x - x3)-p.y
	y3 := ba.Sub(&p.X, x3)
	y3 = ba.Mul(λ2, y3)
	y3 = ba.Sub(y3, &p.Y)

	return &gadget.G2Affine{
		X: *x3,
		Y: *y3,
	}

}

// Double doubles p and return it. It doesn't modify p.
// It uses affine coordinates.
func Double(ba *fields_bn254.Ext2, p *gadget.G2Affine) *gadget.G2Affine {
	// compute λ = (3p.x²+a)/2*p.y, here we assume a=0 (j invariant 0 curve)
	xx3a := ba.Square(&p.X)
	xx3a = ba.MulByConstElement(xx3a, big.NewInt(3))
	y2 := ba.MulByConstElement(&p.Y, big.NewInt(2))
	λ := ba.DivUnchecked(xx3a, y2)

	// xr = λ²-2p.x
	x2 := ba.MulByConstElement(&p.X, big.NewInt(2))
	λλ := ba.Square(λ)
	xr := ba.Sub(λλ, x2)

	// yr = λ(p-xr) - p.y
	pxrx := ba.Sub(&p.X, xr)
	λpxrx := ba.Mul(λ, pxrx)
	yr := ba.Sub(λpxrx, &p.Y)

	return &gadget.G2Affine{
		X: *xr,
		Y: *yr,
	}
}

func Triple(ba *fields_bn254.Ext2, p *gadget.G2Affine) *gadget.G2Affine {

	// compute λ1 = (3p.x²+a)/2p.y, here we assume a=0 (j invariant 0 curve)
	xx := ba.Square(&p.X)
	xx = ba.MulByConstElement(xx, big.NewInt(3))
	y2 := ba.Double(&p.Y)
	λ1 := ba.DivUnchecked(xx, y2)

	// xr = λ1²-2p.x
	x2 := ba.Double(&p.X)
	λ1λ1 := ba.Mul(λ1, λ1)
	x2 = ba.Sub(λ1λ1, x2)

	// ommit y2 computation, and
	// compute λ2 = 2p.y/(x2 − p.x) − λ1.
	x1x2 := ba.Sub(&p.X, x2)
	λ2 := ba.DivUnchecked(y2, x1x2)
	λ2 = ba.Sub(λ2, λ1)

	// xr = λ²-p.x-x2
	λ2λ2 := ba.Mul(λ2, λ2)
	qxrx := ba.Add(x2, &p.X)
	xr := ba.Sub(λ2λ2, qxrx)

	// yr = λ(p.x-xr) - p.y
	pxrx := ba.Sub(&p.X, xr)
	λ2pxrx := ba.Mul(λ2, pxrx)
	yr := ba.Sub(λ2pxrx, &p.Y)

	return &gadget.G2Affine{
		X: *xr,
		Y: *yr,
	}
}

func ScalarMul(sa *emulated.Field[emulated.BN254Fr], ba *fields_bn254.Ext2, p *gadget.G2Affine, s *emulated.Element[emulated.BN254Fr]) *gadget.G2Affine {
	var st emulated.BN254Fr
	sr := sa.Reduce(s)
	sBits := sa.ToBits(sr)
	n := st.Modulus().BitLen()

	// i = 1
	tmp := Triple(ba, p)
	res := Select(ba, sBits[1], tmp, p)
	acc := Add(ba, tmp, p)

	for i := 2; i <= n-3; i++ {
		tmp := Add(ba, res, acc)
		res = Select(ba, sBits[i], tmp, res)
		acc = Double(ba, acc)
	}

	// i = n-2
	tmp = Add(ba, res, acc)
	res = Select(ba, sBits[n-2], tmp, res)

	// i = n-1
	tmp = DoubleAndAdd(ba, acc, res)
	res = Select(ba, sBits[n-1], tmp, res)

	// i = 0
	tmp = Add(ba, res, Neg(ba, p))
	res = Select(ba, sBits[0], res, tmp)

	return res
}

func ClearCofactor(api frontend.API, p *gadget.G2Affine) *gadget.G2Affine {
	ba := fields_bn254.NewExt2(api)
	sa, err := emulated.NewField[emulated.BN254Fr](api)
	if err != nil {
		panic(err)
	}
	// BN254 G2 Cofactor, too big to fit in a single element
	bigH, _ := new(big.Int).SetString("30644e72e131a029b85045b68181585e06ceecda572a2489345f2299c0f9fa8d", 16)
	leftH := new(big.Int).Div(bigH, big.NewInt(2))
	rightH := new(big.Int).Sub(bigH, leftH)

	lh := emulated.ValueOf[emulated.BN254Fr](leftH)
	rh := emulated.ValueOf[emulated.BN254Fr](rightH)

	// Please find a way to optimize
	l := ScalarMul(sa, ba, p, &lh)
	r := ScalarMul(sa, ba, p, &rh)
	return Add(ba, l, r)

}

// AssertIsEqual asserts that p and q are the same point.
func AssertIsEqual(ba *fields_bn254.Ext2, p, q *gadget.G2Affine) {
	ba.AssertIsEqual(&p.X, &q.X)
	ba.AssertIsEqual(&p.Y, &q.Y)
}

func MapToG2(api frontend.API, u *fields_bn254.E2) *gadget.G2Affine {
	return ClearCofactor(api, MapToCurve(api, u))
}
