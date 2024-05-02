package g2

import (
	"math/big"

	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fp"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/algebra/emulated/fields_bn254"
	gadget "github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	"github.com/consensys/gnark/std/hash/mimc"
	"github.com/consensys/gnark/std/math/emulated"
)

const (
	MiMCBlockSize = 256
)

var (
	// Constant not public in gnark crypto...
	bCurveCoeff fp.Element
	// twist
	twist bn254.E2
	// bTwistCurveCoeff b coeff of the twist (defined over 𝔽p²) curve
	bTwistCurveCoeff bn254.E2
	B                fields_bn254.E2
)

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
// Return 1 if square, 0 otherwise
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
	e.ext2.AssertIsEqual(&p.P.X, &q.P.X)
	e.ext2.AssertIsEqual(&p.P.Y, &q.P.Y)
}

// https://datatracker.ietf.org/doc/html/rfc9380#name-the-sgn0-function
func (e *EmulatedAPI) g2Sgn0Circuit(z *fields_bn254.E2) frontend.Variable {
	a0b := e.field.ToBits(&z.A0)

	sign_0 := a0b[0]
	zero_0 := e.field.IsZero(&z.A0)

	a1b := e.field.ToBits(&z.A1)

	sign_1 := a1b[0]
	sign := e.api.Or(sign_0, e.api.And(zero_0, sign_1))

	return sign
}

// Union whitepaper: (2) svdw
//
// https://datatracker.ietf.org/doc/html/rfc9380#straightline-svdw
//
// WARNING: this function is partial, the legendre hint only check the root branch. If you are going to reuse this function, make sure you are aware of this edge case.
func (e *EmulatedAPI) MapToCurve(u *fields_bn254.E2) *gadget.G2Affine {
	// NOTE: up to the caller to call legendre if the root is not guarantee to exist
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
	point := gadget.G2Affine{}
	point.P.X = *x
	point.P.Y = *y
	return &point
}

func (e *EmulatedAPI) Neg(p *gadget.G2Affine) *gadget.G2Affine {
	point := gadget.G2Affine{}
	point.P.X = p.P.X
	point.P.Y = *e.ext2.Neg(&p.P.Y)
	return &point
}

func (e *EmulatedAPI) Select(b frontend.Variable, p, q *gadget.G2Affine) *gadget.G2Affine {
	x := e.ext2.Select(b, &p.P.X, &q.P.X)
	y := e.ext2.Select(b, &p.P.Y, &q.P.Y)
	point := gadget.G2Affine{}
	point.P.X = *x
	point.P.Y = *y
	return &point
}

func (e *EmulatedAPI) Add(p, q *gadget.G2Affine) *gadget.G2Affine {
	// compute λ = (q.y-p.y)/(q.x-p.x)
	qypy := e.ext2.Sub(&q.P.Y, &p.P.Y)
	qxpx := e.ext2.Sub(&q.P.X, &p.P.X)
	λ := e.ext2.DivUnchecked(qypy, qxpx)

	// xr = λ²-p.x-q.x
	λλ := e.ext2.Square(λ)
	qxpx = e.ext2.Add(&p.P.X, &q.P.X)
	xr := e.ext2.Sub(λλ, qxpx)

	// p.y = λ(p.x-r.x) - p.y
	pxrx := e.ext2.Sub(&p.P.X, xr)
	λpxrx := e.ext2.Mul(λ, pxrx)
	yr := e.ext2.Sub(λpxrx, &p.P.Y)

	point := gadget.G2Affine{}
	point.P.X = *xr
	point.P.Y = *yr
	return &point
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
	yqyp := e.ext2.Sub(&q.P.Y, &p.P.Y)
	xqxp := e.ext2.Sub(&q.P.X, &p.P.X)
	λ1 := e.ext2.DivUnchecked(yqyp, xqxp)

	// compute x2 = λ1²-p.x-q.x
	λ1λ1 := e.ext2.Square(λ1)
	xqxp = e.ext2.Add(&p.P.X, &q.P.X)
	x2 := e.ext2.Sub(λ1λ1, xqxp)

	// omit y2 computation
	// compute λ2 = -λ1-2*p.y/(x2-p.x)
	ypyp := e.ext2.Double(&p.P.Y)
	x2xp := e.ext2.Sub(x2, &p.P.X)
	λ2 := e.ext2.DivUnchecked(ypyp, x2xp)
	λ2 = e.ext2.Add(λ1, λ2)
	λ2 = e.ext2.Neg(λ2)

	// compute x3 =λ2²-p.x-x3
	λ2λ2 := e.ext2.Square(λ2)
	x3 := e.ext2.Sub(λ2λ2, &p.P.X)
	x3 = e.ext2.Sub(x3, x2)

	// compute y3 = λ2*(p.x - x3)-p.y
	y3 := e.ext2.Sub(&p.P.X, x3)
	y3 = e.ext2.Mul(λ2, y3)
	y3 = e.ext2.Sub(y3, &p.P.Y)

	point := gadget.G2Affine{}
	point.P.X = *x3
	point.P.Y = *y3
	return &point
}

// Double doubles p and return it. It doesn't modify p.
// It uses affine coordinates.
func (e *EmulatedAPI) Double(p *gadget.G2Affine) *gadget.G2Affine {
	// compute λ = (3p.x²+a)/2*p.y, here we assume a=0 (j invariant 0 curve)
	xx3a := e.ext2.Square(&p.P.X)
	xx3a = e.ext2.MulByConstElement(xx3a, big.NewInt(3))
	y2 := e.ext2.MulByConstElement(&p.P.Y, big.NewInt(2))
	λ := e.ext2.DivUnchecked(xx3a, y2)

	// xr = λ²-2p.x
	x2 := e.ext2.MulByConstElement(&p.P.X, big.NewInt(2))
	λλ := e.ext2.Square(λ)
	xr := e.ext2.Sub(λλ, x2)

	// yr = λ(p-xr) - p.y
	pxrx := e.ext2.Sub(&p.P.X, xr)
	λpxrx := e.ext2.Mul(λ, pxrx)
	yr := e.ext2.Sub(λpxrx, &p.P.Y)

	point := gadget.G2Affine{}
	point.P.X = *xr
	point.P.Y = *yr
	return &point
}

func (e *EmulatedAPI) DoubleN(p *gadget.G2Affine, n int) *gadget.G2Affine {
	pn := p
	for s := 0; s < n; s++ {
		pn = e.Double(pn)
	}
	return pn
}

func (e *EmulatedAPI) Psi(q *gadget.G2Affine) *gadget.G2Affine {
	x := e.ext2.Conjugate(&q.P.X)
	x = e.ext2.Mul(x, e.u)
	y := e.ext2.Conjugate(&q.P.Y)
	y = e.ext2.Mul(y, e.v)
	point := gadget.G2Affine{}
	point.P.X = *x
	point.P.Y = *y
	return &point
}

// Scalar multiplication by bn254 seed.
// The input point must not be the infinity point.

// NOTE: we only use this function in cofactor clearing in hashing to curve.
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

// Union whitepaper: (2) h
//
// http://cacr.uwaterloo.ca/techreports/2011/cacr2011-26.pdf, 6.1
func (e *EmulatedAPI) ClearCofactor(Q *gadget.G2Affine) *gadget.G2Affine {
	// xQ
	xQ := e.ScalarMulBySeed(Q)
	// psi(3xQ)
	psi_3xQ := e.Psi(e.Add(e.Double(xQ), xQ))
	// psi^2(xQ)
	psi2_xQ := e.Psi(e.Psi(xQ))
	// psi^3(Q)
	psi3_Q := e.Psi(e.Psi(e.Psi(Q)))
	// xQ + psi(3xQ) + psi^2(xQ) + psi^3(Q)
	return e.Add(e.Add(e.Add(xQ, psi_3xQ), psi2_xQ), psi3_Q)
}

// Union whitepaper: (1), (2) M ◦ H_{mimc^4}
//
// https://datatracker.ietf.org/doc/html/rfc9380#name-encoding-byte-strings-to-el

// WARNING: this function calls a partial MapToCurve, read it's documentation before using it.
func (e *EmulatedAPI) HashToG2(message frontend.Variable, dst frontend.Variable) (*gadget.G2Affine, error) {
	u, err := e.HashToField(message, dst)
	if err != nil {
		return nil, err
	}
	Q0 := e.MapToCurve(&fields_bn254.E2{
		A0: *u[0],
		A1: *u[1],
	})
	Q1 := e.MapToCurve(&fields_bn254.E2{
		A0: *u[2],
		A1: *u[3],
	})
	return e.ClearCofactor(e.Add(Q0, Q1)), nil
}

// Union whitepaper: (1), (2) M ◦ H_{mimc^4}
//
// https://datatracker.ietf.org/doc/html/rfc9380#name-hash_to_field-implementatio
// /
// WARNING: /!\ Tailored for 4 field elements (actually scalar field because of the underlying MiMC hash function).
// WARNING: this functions uses a 256bit block MiMC (which is in fact only 254bit), use it at your own risk.
// WARNING: we only support 254bit messages (usually MiMC hash) and domain separation tag (usually MiMC hash).
func (e *EmulatedAPI) HashToField(message frontend.Variable, dst frontend.Variable) ([]*emulated.Element[emulated.BN254Fp], error) {
	pseudoRandomBits, err := e.ExpandMsgXmd(message, dst)
	if err != nil {
		return nil, err
	}
	elements := make([]*emulated.Element[emulated.BN254Fp], 4)
	for i := 0; i < 4; i++ {
		elemBits := pseudoRandomBits[i*48*8 : (i+1)*48*8]
		// sadly this result is >limbs than expected and looks like gnark don't like it
		splitPoint := int64(17)
		l := e.field.FromBits(elemBits[:splitPoint*8]...)
		r := e.field.FromBits(elemBits[splitPoint*8:]...)
		c := emulated.ValueOf[emulated.BN254Fp](new(big.Int).Lsh(big.NewInt(1), uint(splitPoint*8)))
		elements[i] = e.field.Add(l, e.field.Mul(r, &c))
	}
	return elements, nil
}

// /!\ IMPORTANT /!\ : This is not a general implementation as the input/output length are fixed.
// It is a tailor-made for BN254G2_XMD:MiMC-256_SVDW hash_to_curve implementation.
//
// WARNING: this functions uses a 256bit block MiMC (effectively 254bit with modulus happening), use it at your own risk.
// WARNING: we only support ~254bit messages and domain separation tag (both bn254 F_r elements).
//
// https://datatracker.ietf.org/doc/html/rfc9380#name-expand_message_xmd
// https://datatracker.ietf.org/doc/html/rfc9380#name-utility-functions (I2OSP/O2ISP)
// https://eprint.iacr.org/2016/492.pdf
func (e *EmulatedAPI) ExpandMsgXmd(message frontend.Variable, dst frontend.Variable) ([]frontend.Variable, error) {
	h, err := mimc.NewMiMC(e.api)
	if err != nil {
		return nil, err
	}

	block := []frontend.Variable{}
	write := func(b ...frontend.Variable) {
		block = append(block, b...)
	}
	repeat := func(b frontend.Variable, count int) {
		bs := make([]frontend.Variable, count)
		for i := 0; i < count; i++ {
			bs[i] = b
		}
		write(bs...)
	}
	sum := func() []frontend.Variable {
		// fill block
		if len(block)%MiMCBlockSize != 0 {
			repeat(0, MiMCBlockSize-len(block)%MiMCBlockSize)
		}
		for i := 0; i < len(block); i += MiMCBlockSize {
			h.Write(e.api.FromBinary(block[i : i+MiMCBlockSize]...))
		}
		block = []frontend.Variable{}
		s := h.Sum()
		h.Reset()
		return e.api.ToBinary(s, 256)
	}

	writeU8 := func(x frontend.Variable) {
		write(e.api.ToBinary(x, 8)...)
	}

	write_message := func() {
		write(e.api.ToBinary(message, 256)...)
	}

	// Z_pad = I2OSP(0, r_in_bytes)
	write_Z_pad := func() {
		repeat(0, 256)
	}

	// l_i_b_str = I2OSP(len_in_bytes, 2)
	write_l_i_b_str := func() {
		writeU8(192 >> 8)
		writeU8(192)
	}

	// DST_prime =  DST ∥ I2OSP(len(DST), 1)
	write_DST_prime := func() {
		write(e.api.ToBinary(dst, 256)...)
		writeU8(32)
	}

	// Z_pad = I2OSP(0, r_in_bytes)
	// l_i_b_str = I2OSP(len_in_bytes, 2)
	// DST_prime =  DST ∥ I2OSP(len(DST), 1)
	// b₀ = H(Z_pad ∥ msg ∥ l_i_b_str ∥ I2OSP(0, 1) ∥ DST_prime)
	write_Z_pad()
	write_message()
	write_l_i_b_str()
	writeU8(0)
	write_DST_prime()
	b0 := sum()

	// b₁ = H(b₀ ∥ I2OSP(1, 1) ∥ DST_prime)
	write(b0...)
	writeU8(1)
	write_DST_prime()
	b1 := sum()

	res := make([]frontend.Variable, 192*8)
	for i := 0; i < len(res); i++ {
		res[i] = 0
	}
	copy(res, b1)

	for i := 1; i <= 5; i++ {
		strxor := make([]frontend.Variable, 256)
		for j := 0; j < 256; j++ {
			strxor[j] = e.api.Xor(b0[j], b1[j])
		}

		// b_i = H(strxor(b₀, b_(i - 1)) ∥ I2OSP(i, 1) ∥ DST_prime)
		write(strxor...)
		writeU8(i + 1)
		write_DST_prime()
		b1 = sum()

		copy(res[i*256:(i+1)*256], b1)
	}

	return res, nil
}
