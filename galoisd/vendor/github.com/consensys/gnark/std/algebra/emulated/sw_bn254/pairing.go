package sw_bn254

import (
	"errors"
	"fmt"
	"math/big"

	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/algebra/emulated/fields_bn254"
	"github.com/consensys/gnark/std/math/emulated"
)

type Pairing struct {
	api frontend.API
	*fields_bn254.Ext12
	curveF *emulated.Field[emulated.BN254Fp]
}

type GTEl = fields_bn254.E12

func NewGTEl(v bn254.GT) GTEl {
	return GTEl{
		C0: fields_bn254.E6{
			B0: fields_bn254.E2{
				A0: emulated.ValueOf[emulated.BN254Fp](v.C0.B0.A0),
				A1: emulated.ValueOf[emulated.BN254Fp](v.C0.B0.A1),
			},
			B1: fields_bn254.E2{
				A0: emulated.ValueOf[emulated.BN254Fp](v.C0.B1.A0),
				A1: emulated.ValueOf[emulated.BN254Fp](v.C0.B1.A1),
			},
			B2: fields_bn254.E2{
				A0: emulated.ValueOf[emulated.BN254Fp](v.C0.B2.A0),
				A1: emulated.ValueOf[emulated.BN254Fp](v.C0.B2.A1),
			},
		},
		C1: fields_bn254.E6{
			B0: fields_bn254.E2{
				A0: emulated.ValueOf[emulated.BN254Fp](v.C1.B0.A0),
				A1: emulated.ValueOf[emulated.BN254Fp](v.C1.B0.A1),
			},
			B1: fields_bn254.E2{
				A0: emulated.ValueOf[emulated.BN254Fp](v.C1.B1.A0),
				A1: emulated.ValueOf[emulated.BN254Fp](v.C1.B1.A1),
			},
			B2: fields_bn254.E2{
				A0: emulated.ValueOf[emulated.BN254Fp](v.C1.B2.A0),
				A1: emulated.ValueOf[emulated.BN254Fp](v.C1.B2.A1),
			},
		},
	}
}

func NewPairing(api frontend.API) (*Pairing, error) {
	ba, err := emulated.NewField[emulated.BN254Fp](api)
	if err != nil {
		return nil, fmt.Errorf("new base api: %w", err)
	}
	return &Pairing{
		api:    api,
		Ext12:  fields_bn254.NewExt12(api),
		curveF: ba,
	}, nil
}

// FinalExponentiation computes the exponentiation eᵈ where
//
//	d = (p¹²-1)/r = (p¹²-1)/Φ₁₂(p) ⋅ Φ₁₂(p)/r = (p⁶-1)(p²+1)(p⁴ - p² +1)/r.
//
// We use instead d'= s ⋅ d, where s is the cofactor
//
//	2x₀(6x₀²+3x₀+1)
//
// and r does NOT divide d'
//
// FinalExponentiation returns a decompressed element in E12.
//
// This is the safe version of the method where e may be {-1,1}. If it is known
// that e ≠ {-1,1} then using the unsafe version of the method saves
// considerable amount of constraints. When called with the result of
// [MillerLoop], then current method is applicable when length of the inputs to
// Miller loop is 1.
func (pr Pairing) FinalExponentiation(e *GTEl) *GTEl {
	return pr.finalExponentiation(e, false)
}

// FinalExponentiationUnsafe computes the exponentiation eᵈ where
//
//	d = (p¹²-1)/r = (p¹²-1)/Φ₁₂(p) ⋅ Φ₁₂(p)/r = (p⁶-1)(p²+1)(p⁴ - p² +1)/r.
//
// We use instead d'= s ⋅ d, where s is the cofactor
//
//	2x₀(6x₀²+3x₀+1)
//
// and r does NOT divide d'
//
// FinalExponentiationUnsafe returns a decompressed element in E12.
//
// This is the unsafe version of the method where e may NOT be {-1,1}. If e ∈
// {-1, 1}, then there exists no valid solution to the circuit. This method is
// applicable when called with the result of [MillerLoop] method when the length
// of the inputs to Miller loop is 1.
func (pr Pairing) FinalExponentiationUnsafe(e *GTEl) *GTEl {
	return pr.finalExponentiation(e, true)
}

// finalExponentiation computes the exponentiation eᵈ where
//
//	d = (p¹²-1)/r = (p¹²-1)/Φ₁₂(p) ⋅ Φ₁₂(p)/r = (p⁶-1)(p²+1)(p⁴ - p² +1)/r.
//
// We use instead d'= s ⋅ d, where s is the cofactor
//
//	2x₀(6x₀²+3x₀+1)
//
// and r does NOT divide d'
//
// finalExponentiation returns a decompressed element in E12
func (pr Pairing) finalExponentiation(e *GTEl, unsafe bool) *GTEl {

	// 1. Easy part
	// (p⁶-1)(p²+1)
	var selector1, selector2 frontend.Variable
	_dummy := pr.Ext6.One()

	if unsafe {
		// The Miller loop result is ≠ {-1,1}, otherwise this means P and Q are
		// linearly dependant and not from G1 and G2 respectively.
		// So e ∈ G_{q,2} \ {-1,1} and hence e.C1 ≠ 0.
		// Nothing to do.

	} else {
		// However, for a product of Miller loops (n>=2) this might happen.  If this is
		// the case, the result is 1 in the torus. We assign a dummy value (1) to e.C1
		// and proceed further.
		selector1 = pr.Ext6.IsZero(&e.C1)
		e.C1 = *pr.Ext6.Select(selector1, _dummy, &e.C1)
	}

	// Torus compression absorbed:
	// Raising e to (p⁶-1) is
	// e^(p⁶) / e = (e.C0 - w*e.C1) / (e.C0 + w*e.C1)
	//            = (-e.C0/e.C1 + w) / (-e.C0/e.C1 - w)
	// So the fraction -e.C0/e.C1 is already in the torus.
	// This absorbs the torus compression in the easy part.
	c := pr.Ext6.DivUnchecked(&e.C0, &e.C1)
	c = pr.Ext6.Neg(c)
	t0 := pr.FrobeniusSquareTorus(c)
	c = pr.MulTorus(t0, c)

	// 2. Hard part (up to permutation)
	// 2x₀(6x₀²+3x₀+1)(p⁴-p²+1)/r
	// Duquesne and Ghammam
	// https://eprint.iacr.org/2015/192.pdf
	// Fuentes et al. (alg. 6)
	// performed in torus compressed form
	t0 = pr.ExptTorus(c)
	t0 = pr.InverseTorus(t0)
	t0 = pr.SquareTorus(t0)
	t1 := pr.SquareTorus(t0)
	t1 = pr.MulTorus(t0, t1)
	t2 := pr.ExptTorus(t1)
	t2 = pr.InverseTorus(t2)
	t3 := pr.InverseTorus(t1)
	t1 = pr.MulTorus(t2, t3)
	t3 = pr.SquareTorus(t2)
	t4 := pr.ExptTorus(t3)
	t4 = pr.MulTorus(t1, t4)
	t3 = pr.MulTorus(t0, t4)
	t0 = pr.MulTorus(t2, t4)
	t0 = pr.MulTorus(c, t0)
	t2 = pr.FrobeniusTorus(t3)
	t0 = pr.MulTorus(t2, t0)
	t2 = pr.FrobeniusSquareTorus(t4)
	t0 = pr.MulTorus(t2, t0)
	t2 = pr.InverseTorus(c)
	t2 = pr.MulTorus(t2, t3)
	t2 = pr.FrobeniusCubeTorus(t2)

	var result GTEl
	// MulTorus(t0, t2) requires t0 ≠ -t2. When t0 = -t2, it means the
	// product is 1 in the torus.
	if unsafe {
		// For a single pairing, this does not happen because the pairing is non-degenerate.
		result = *pr.DecompressTorus(pr.MulTorus(t2, t0))
	} else {
		// For a product of pairings this might happen when the result is expected to be 1.
		// We assign a dummy value (1) to t0 and proceed furhter.
		// Finally we do a select on both edge cases:
		//   - Only if seletor1=0 and selector2=0, we return MulTorus(t2, t0) decompressed.
		//   - Otherwise, we return 1.
		_sum := pr.Ext6.Add(t0, t2)
		selector2 = pr.Ext6.IsZero(_sum)
		t0 = pr.Ext6.Select(selector2, _dummy, t0)
		selector := pr.api.Mul(pr.api.Sub(1, selector1), pr.api.Sub(1, selector2))
		result = *pr.Select(selector, pr.DecompressTorus(pr.MulTorus(t2, t0)), pr.One())
	}

	return &result
}

// Pair calculates the reduced pairing for a set of points
// ∏ᵢ e(Pᵢ, Qᵢ).
//
// This function doesn't check that the inputs are in the correct subgroups.
func (pr Pairing) Pair(P []*G1Affine, Q []*G2Affine) (*GTEl, error) {
	res, err := pr.MillerLoop(P, Q)
	if err != nil {
		return nil, fmt.Errorf("miller loop: %w", err)
	}
	res = pr.finalExponentiation(res, len(P) == 1)
	return res, nil
}

// PairingCheck calculates the reduced pairing for a set of points and asserts if the result is One
// ∏ᵢ e(Pᵢ, Qᵢ) =? 1
//
// This function doesn't check that the inputs are in the correct subgroups.
func (pr Pairing) PairingCheck(P []*G1Affine, Q []*G2Affine) error {
	f, err := pr.Pair(P, Q)
	if err != nil {
		return err

	}
	one := pr.One()
	pr.AssertIsEqual(f, one)

	return nil
}

func (pr Pairing) AssertIsEqual(x, y *GTEl) {
	pr.Ext12.AssertIsEqual(x, y)
}

// loopCounter = 6x₀+2 = 29793968203157093288
//
// in 2-NAF
var loopCounter = [66]int8{
	0, 0, 0, 1, 0, 1, 0, -1, 0, 0, -1,
	0, 0, 0, 1, 0, 0, -1, 0, -1, 0, 0,
	0, 1, 0, -1, 0, 0, 0, 0, -1, 0, 0,
	1, 0, -1, 0, 0, 1, 0, 0, 0, 0, 0,
	-1, 0, 0, -1, 0, 1, 0, -1, 0, 0, 0,
	-1, 0, -1, 0, 0, 0, 1, 0, -1, 0, 1,
}

// lineEvaluation represents a sparse Fp12 Elmt (result of the line evaluation)
// line: 1 + R0(x/y) + R1(1/y) = 0 instead of R0'*y + R1'*x + R2' = 0 This
// makes the multiplication by lines (MulBy034) and between lines (Mul034By034)
// circuit-efficient.
type lineEvaluation struct {
	R0, R1 fields_bn254.E2
}

// MillerLoop computes the multi-Miller loop
// ∏ᵢ { fᵢ_{6x₀+2,Q}(P) · ℓᵢ_{[6x₀+2]Q,π(Q)}(P) · ℓᵢ_{[6x₀+2]Q+π(Q),-π²(Q)}(P) }
func (pr Pairing) MillerLoop(P []*G1Affine, Q []*G2Affine) (*GTEl, error) {
	// check input size match
	n := len(P)
	if n == 0 || n != len(Q) {
		return nil, errors.New("invalid inputs sizes")
	}

	res := pr.Ext12.One()
	var prodLines [5]fields_bn254.E2

	var l1, l2 *lineEvaluation
	Qacc := make([]*G2Affine, n)
	QNeg := make([]*G2Affine, n)
	yInv := make([]*emulated.Element[emulated.BN254Fp], n)
	xOverY := make([]*emulated.Element[emulated.BN254Fp], n)

	for k := 0; k < n; k++ {
		Qacc[k] = Q[k]
		QNeg[k] = &G2Affine{X: Q[k].X, Y: *pr.Ext2.Neg(&Q[k].Y)}
		// P and Q are supposed to be on G1 and G2 respectively of prime order r.
		// The point (x,0) is of order 2. But this function does not check
		// subgroup membership.
		// Anyway (x,0) cannot be on BN254 because -3 is a cubic non-residue in Fp.
		// So, 1/y is well defined for all points P's.
		yInv[k] = pr.curveF.Inverse(&P[k].Y)
		xOverY[k] = pr.curveF.MulMod(&P[k].X, yInv[k])
	}

	// Compute ∏ᵢ { fᵢ_{6x₀+2,Q}(P) }
	// i = 64, separately to avoid an E12 Square
	// (Square(res) = 1² = 1)

	// k = 0, separately to avoid MulBy034 (res × ℓ)
	// (assign line to res)
	Qacc[0], l1 = pr.doubleStep(Qacc[0])
	// line evaluation at P[0]
	res.C1.B0 = *pr.MulByElement(&l1.R0, xOverY[0])
	res.C1.B1 = *pr.MulByElement(&l1.R1, yInv[0])

	if n >= 2 {
		// k = 1, separately to avoid MulBy034 (res × ℓ)
		// (res is also a line at this point, so we use Mul034By034 ℓ × ℓ)
		Qacc[1], l1 = pr.doubleStep(Qacc[1])

		// line evaluation at P[1]
		l1.R0 = *pr.MulByElement(&l1.R0, xOverY[1])
		l1.R1 = *pr.MulByElement(&l1.R1, yInv[1])

		// ℓ × res
		prodLines = *pr.Mul034By034(&l1.R0, &l1.R1, &res.C1.B0, &res.C1.B1)
		res.C0.B0 = prodLines[0]
		res.C0.B1 = prodLines[1]
		res.C0.B2 = prodLines[2]
		res.C1.B0 = prodLines[3]
		res.C1.B1 = prodLines[4]
	}

	if n >= 3 {
		// k = 2, separately to avoid MulBy034 (res × ℓ)
		// (res has a zero E2 element, so we use Mul01234By034)
		Qacc[2], l1 = pr.doubleStep(Qacc[2])

		// line evaluation at P[1]
		l1.R0 = *pr.MulByElement(&l1.R0, xOverY[2])
		l1.R1 = *pr.MulByElement(&l1.R1, yInv[2])

		// ℓ × res
		res = pr.Mul01234By034(&prodLines, &l1.R0, &l1.R1)

		// k >= 3
		for k := 3; k < n; k++ {
			// Qacc[k] ← 2Qacc[k] and l1 the tangent ℓ passing 2Qacc[k]
			Qacc[k], l1 = pr.doubleStep(Qacc[k])

			// line evaluation at P[k]
			l1.R0 = *pr.MulByElement(&l1.R0, xOverY[k])
			l1.R1 = *pr.MulByElement(&l1.R1, yInv[k])

			// ℓ × res
			res = pr.MulBy034(res, &l1.R0, &l1.R1)
		}
	}

	// i = 63, separately to avoid a doubleStep
	// (at this point Qacc = 2Q, so 2Qacc-Q=3Q is equivalent to Qacc+Q=3Q
	// this means doubleAndAddStep is equivalent to addStep here)
	res = pr.Square(res)
	for k := 0; k < n; k++ {
		// l2 the line passing Qacc[k] and -Q
		l2 = pr.lineCompute(Qacc[k], QNeg[k])

		// line evaluation at P[k]
		l2.R0 = *pr.MulByElement(&l2.R0, xOverY[k])
		l2.R1 = *pr.MulByElement(&l2.R1, yInv[k])

		// Qacc[k] ← Qacc[k]+Q[k] and
		// l1 the line ℓ passing Qacc[k] and Q[k]
		Qacc[k], l1 = pr.addStep(Qacc[k], Q[k])

		// line evaluation at P[k]
		l1.R0 = *pr.MulByElement(&l1.R0, xOverY[k])
		l1.R1 = *pr.MulByElement(&l1.R1, yInv[k])

		// ℓ × ℓ
		prodLines = *pr.Mul034By034(&l1.R0, &l1.R1, &l2.R0, &l2.R1)
		// (ℓ × ℓ) × res
		res = pr.MulBy01234(res, &prodLines)
	}

	for i := 62; i >= 0; i-- {
		// mutualize the square among n Miller loops
		// (∏ᵢfᵢ)²
		res = pr.Square(res)

		switch loopCounter[i] {

		case 0:
			for k := 0; k < n; k++ {
				// Qacc[k] ← 2Qacc[k] and l1 the tangent ℓ passing 2Qacc[k]
				Qacc[k], l1 = pr.doubleStep(Qacc[k])

				// line evaluation at P[k]
				l1.R0 = *pr.MulByElement(&l1.R0, xOverY[k])
				l1.R1 = *pr.MulByElement(&l1.R1, yInv[k])

				// ℓ × res
				res = pr.MulBy034(res, &l1.R0, &l1.R1)
			}

		case 1:
			for k := 0; k < n; k++ {
				// Qacc[k] ← 2Qacc[k]+Q[k],
				// l1 the line ℓ passing Qacc[k] and Q[k]
				// l2 the line ℓ passing (Qacc[k]+Q[k]) and Qacc[k]
				Qacc[k], l1, l2 = pr.doubleAndAddStep(Qacc[k], Q[k])

				// line evaluation at P[k]
				l1.R0 = *pr.MulByElement(&l1.R0, xOverY[k])
				l1.R1 = *pr.MulByElement(&l1.R1, yInv[k])

				// line evaluation at P[k]
				l2.R0 = *pr.MulByElement(&l2.R0, xOverY[k])
				l2.R1 = *pr.MulByElement(&l2.R1, yInv[k])

				// ℓ × ℓ
				prodLines = *pr.Mul034By034(&l1.R0, &l1.R1, &l2.R0, &l2.R1)
				// (ℓ × ℓ) × res
				res = pr.MulBy01234(res, &prodLines)

			}

		case -1:
			for k := 0; k < n; k++ {
				// Qacc[k] ← 2Qacc[k]-Q[k],
				// l1 the line ℓ passing Qacc[k] and -Q[k]
				// l2 the line ℓ passing (Qacc[k]-Q[k]) and Qacc[k]
				Qacc[k], l1, l2 = pr.doubleAndAddStep(Qacc[k], QNeg[k])

				// line evaluation at P[k]
				l1.R0 = *pr.MulByElement(&l1.R0, xOverY[k])
				l1.R1 = *pr.MulByElement(&l1.R1, yInv[k])

				// line evaluation at P[k]
				l2.R0 = *pr.MulByElement(&l2.R0, xOverY[k])
				l2.R1 = *pr.MulByElement(&l2.R1, yInv[k])

				// ℓ × ℓ
				prodLines = *pr.Mul034By034(&l1.R0, &l1.R1, &l2.R0, &l2.R1)
				// (ℓ × ℓ) × res
				res = pr.MulBy01234(res, &prodLines)

			}

		default:
			return nil, errors.New("invalid loopCounter")
		}
	}

	// Compute  ∏ᵢ { ℓᵢ_{[6x₀+2]Q,π(Q)}(P) · ℓᵢ_{[6x₀+2]Q+π(Q),-π²(Q)}(P) }
	Q1, Q2 := new(G2Affine), new(G2Affine)
	for k := 0; k < n; k++ {
		//Q1 = π(Q)
		Q1.X = *pr.Ext12.Ext2.Conjugate(&Q[k].X)
		Q1.X = *pr.Ext12.Ext2.MulByNonResidue1Power2(&Q1.X)
		Q1.Y = *pr.Ext12.Ext2.Conjugate(&Q[k].Y)
		Q1.Y = *pr.Ext12.Ext2.MulByNonResidue1Power3(&Q1.Y)

		// Q2 = -π²(Q)
		Q2.X = *pr.Ext12.Ext2.MulByNonResidue2Power2(&Q[k].X)
		Q2.Y = *pr.Ext12.Ext2.MulByNonResidue2Power3(&Q[k].Y)
		Q2.Y = *pr.Ext12.Ext2.Neg(&Q2.Y)

		// Qacc[k] ← Qacc[k]+π(Q) and
		// l1 the line passing Qacc[k] and π(Q)
		Qacc[k], l1 = pr.addStep(Qacc[k], Q1)

		// line evaluation at P[k]
		l1.R0 = *pr.Ext2.MulByElement(&l1.R0, xOverY[k])
		l1.R1 = *pr.Ext2.MulByElement(&l1.R1, yInv[k])

		// l2 the line passing Qacc[k] and -π²(Q)
		l2 = pr.lineCompute(Qacc[k], Q2)
		// line evaluation at P[k]
		l2.R0 = *pr.MulByElement(&l2.R0, xOverY[k])
		l2.R1 = *pr.MulByElement(&l2.R1, yInv[k])

		// ℓ × ℓ
		prodLines = *pr.Mul034By034(&l1.R0, &l1.R1, &l2.R0, &l2.R1)
		// (ℓ × ℓ) × res
		res = pr.MulBy01234(res, &prodLines)

	}

	return res, nil
}

// doubleAndAddStep doubles p1 and adds p2 to the result in affine coordinates, and evaluates the line in Miller loop
// https://eprint.iacr.org/2022/1162 (Section 6.1)
func (pr Pairing) doubleAndAddStep(p1, p2 *G2Affine) (*G2Affine, *lineEvaluation, *lineEvaluation) {

	var line1, line2 lineEvaluation
	var p G2Affine

	// compute λ1 = (y2-y1)/(x2-x1)
	n := pr.Ext2.Sub(&p1.Y, &p2.Y)
	d := pr.Ext2.Sub(&p1.X, &p2.X)
	l1 := pr.Ext2.DivUnchecked(n, d)

	// compute x3 =λ1²-x1-x2
	x3 := pr.Ext2.Square(l1)
	x3 = pr.Ext2.Sub(x3, &p1.X)
	x3 = pr.Ext2.Sub(x3, &p2.X)

	// omit y3 computation

	// compute line1
	line1.R0 = *pr.Ext2.Neg(l1)
	line1.R1 = *pr.Ext2.Mul(l1, &p1.X)
	line1.R1 = *pr.Ext2.Sub(&line1.R1, &p1.Y)

	// compute λ2 = -λ1-2y1/(x3-x1)
	n = pr.Ext2.Double(&p1.Y)
	d = pr.Ext2.Sub(x3, &p1.X)
	l2 := pr.Ext2.DivUnchecked(n, d)
	l2 = pr.Ext2.Add(l2, l1)
	l2 = pr.Ext2.Neg(l2)

	// compute x4 = λ2²-x1-x3
	x4 := pr.Ext2.Square(l2)
	x4 = pr.Ext2.Sub(x4, &p1.X)
	x4 = pr.Ext2.Sub(x4, x3)

	// compute y4 = λ2(x1 - x4)-y1
	y4 := pr.Ext2.Sub(&p1.X, x4)
	y4 = pr.Ext2.Mul(l2, y4)
	y4 = pr.Ext2.Sub(y4, &p1.Y)

	p.X = *x4
	p.Y = *y4

	// compute line2
	line2.R0 = *pr.Ext2.Neg(l2)
	line2.R1 = *pr.Ext2.Mul(l2, &p1.X)
	line2.R1 = *pr.Ext2.Sub(&line2.R1, &p1.Y)

	return &p, &line1, &line2
}

// doubleStep doubles a point in affine coordinates, and evaluates the line in Miller loop
// https://eprint.iacr.org/2022/1162 (Section 6.1)
func (pr Pairing) doubleStep(p1 *G2Affine) (*G2Affine, *lineEvaluation) {

	var p G2Affine
	var line lineEvaluation

	// λ = 3x²/2y
	n := pr.Ext2.Square(&p1.X)
	three := big.NewInt(3)
	n = pr.Ext2.MulByConstElement(n, three)
	d := pr.Ext2.Double(&p1.Y)
	λ := pr.Ext2.DivUnchecked(n, d)

	// xr = λ²-2x
	xr := pr.Ext2.Square(λ)
	xr = pr.Ext2.Sub(xr, &p1.X)
	xr = pr.Ext2.Sub(xr, &p1.X)

	// yr = λ(x-xr)-y
	yr := pr.Ext2.Sub(&p1.X, xr)
	yr = pr.Ext2.Mul(λ, yr)
	yr = pr.Ext2.Sub(yr, &p1.Y)

	p.X = *xr
	p.Y = *yr

	line.R0 = *pr.Ext2.Neg(λ)
	line.R1 = *pr.Ext2.Mul(λ, &p1.X)
	line.R1 = *pr.Ext2.Sub(&line.R1, &p1.Y)

	return &p, &line

}

// addStep adds two points in affine coordinates, and evaluates the line in Miller loop
// https://eprint.iacr.org/2022/1162 (Section 6.1)
func (pr Pairing) addStep(p1, p2 *G2Affine) (*G2Affine, *lineEvaluation) {

	// compute λ = (y2-y1)/(x2-x1)
	p2ypy := pr.Ext2.Sub(&p2.Y, &p1.Y)
	p2xpx := pr.Ext2.Sub(&p2.X, &p1.X)
	λ := pr.Ext2.DivUnchecked(p2ypy, p2xpx)

	// xr = λ²-x1-x2
	λλ := pr.Ext2.Square(λ)
	p2xpx = pr.Ext2.Add(&p1.X, &p2.X)
	xr := pr.Ext2.Sub(λλ, p2xpx)

	// yr = λ(x1-xr) - y1
	pxrx := pr.Ext2.Sub(&p1.X, xr)
	λpxrx := pr.Ext2.Mul(λ, pxrx)
	yr := pr.Ext2.Sub(λpxrx, &p1.Y)

	var res G2Affine
	res.X = *xr
	res.Y = *yr

	var line lineEvaluation
	line.R0 = *pr.Ext2.Neg(λ)
	line.R1 = *pr.Ext2.Mul(λ, &p1.X)
	line.R1 = *pr.Ext2.Sub(&line.R1, &p1.Y)

	return &res, &line

}

// lineCompute computes the line that goes through p1 and p2 but does not compute p1+p2
func (pr Pairing) lineCompute(p1, p2 *G2Affine) *lineEvaluation {

	// compute λ = (y2-y1)/(x2-x1)
	qypy := pr.Ext2.Sub(&p2.Y, &p1.Y)
	qxpx := pr.Ext2.Sub(&p2.X, &p1.X)
	λ := pr.Ext2.DivUnchecked(qypy, qxpx)

	var line lineEvaluation
	line.R0 = *pr.Ext2.Neg(λ)
	line.R1 = *pr.Ext2.Mul(λ, &p1.X)
	line.R1 = *pr.Ext2.Sub(&line.R1, &p1.Y)

	return &line

}
