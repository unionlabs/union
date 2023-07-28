//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package bw6761

import (
	"errors"

	"github.com/consensys/gnark-crypto/ecc/bw6-761/fp"
	"github.com/consensys/gnark-crypto/ecc/bw6-761/internal/fptower"
)

// GT target group of the pairing
type GT = fptower.E6

type lineEvaluation struct {
	r0 fp.Element
	r1 fp.Element
	r2 fp.Element
}

// Pair calculates the reduced pairing for a set of points
// ∏ᵢ e(Pᵢ, Qᵢ).
//
// This function doesn't check that the inputs are in the correct subgroup. See IsInSubGroup.
func Pair(P []G1Affine, Q []G2Affine) (GT, error) {
	f, err := MillerLoop(P, Q)
	if err != nil {
		return GT{}, err
	}
	return FinalExponentiation(&f), nil
}

// PairingCheck calculates the reduced pairing for a set of points and returns True if the result is One
// ∏ᵢ e(Pᵢ, Qᵢ) =? 1
//
// This function doesn't check that the inputs are in the correct subgroup. See IsInSubGroup.
func PairingCheck(P []G1Affine, Q []G2Affine) (bool, error) {
	f, err := Pair(P, Q)
	if err != nil {
		return false, err
	}
	var one GT
	one.SetOne()
	return f.Equal(&one), nil
}

// FinalExponentiation computes the exponentiation (∏ᵢ zᵢ)ᵈ
// where d = (p^6-1)/r = (p^6-1)/Φ_6(p) ⋅ Φ_6(p)/r = (p^3-1)(p+1)(p^2 - p +1)/r
// we use instead d=s ⋅ (p^3-1)(p+1)(p^2 - p +1)/r
// where s is the cofactor 12(x_0+1) (El Housni and Guillevic)
func FinalExponentiation(z *GT, _z ...*GT) GT {

	var result GT
	result.Set(z)

	for _, e := range _z {
		result.Mul(&result, e)
	}

	var buf GT

	// Easy part
	// (p^3-1)(p+1)
	buf.Conjugate(&result)
	result.Inverse(&result)
	buf.Mul(&buf, &result)
	result.Frobenius(&buf).
		Mul(&result, &buf)

	// Hard part (up to permutation)
	// El Housni and Guillevic
	// https://eprint.iacr.org/2020/351.pdf
	var m1, _m1, m2, _m2, m3, f0, f0_36, g0, g1, _g1, g2, g3, _g3, g4, _g4, g5, _g5, g6, gA, gB, g034, _g1g2, gC, h1, h2, h2g2C, h4 GT
	m1.Expt(&result)
	_m1.Conjugate(&m1)
	m2.Expt(&m1)
	_m2.Conjugate(&m2)
	m3.Expt(&m2)
	f0.Frobenius(&result).
		Mul(&f0, &result).
		Mul(&f0, &m2)
	m2.CyclotomicSquare(&_m1)
	f0.Mul(&f0, &m2)
	f0_36.CyclotomicSquare(&f0).
		CyclotomicSquare(&f0_36).
		CyclotomicSquare(&f0_36).
		Mul(&f0_36, &f0).
		CyclotomicSquare(&f0_36).
		CyclotomicSquare(&f0_36)
	g0.Mul(&result, &m1).
		Frobenius(&g0).
		Mul(&g0, &m3).
		Mul(&g0, &_m2).
		Mul(&g0, &_m1)
	g1.Expt(&g0)
	_g1.Conjugate(&g1)
	g2.Expt(&g1)
	g3.Expt(&g2)
	_g3.Conjugate(&g3)
	g4.Expt(&g3)
	_g4.Conjugate(&g4)
	g5.Expt(&g4)
	_g5.Conjugate(&g5)
	g6.Expt(&g5)
	gA.Mul(&g3, &_g5).
		CyclotomicSquare(&gA).
		Mul(&gA, &g6).
		Mul(&gA, &g1).
		Mul(&gA, &g0)
	g034.Mul(&g0, &g3).
		Mul(&g034, &_g4)
	gB.CyclotomicSquare(&g034).
		Mul(&gB, &g034).
		Mul(&gB, &g5).
		Mul(&gB, &_g1)
	_g1g2.Mul(&_g1, &g2)
	gC.Mul(&_g3, &_g1g2).
		CyclotomicSquare(&gC).
		Mul(&gC, &_g1g2).
		Mul(&gC, &g0).
		CyclotomicSquare(&gC).
		Mul(&gC, &g2).
		Mul(&gC, &g0).
		Mul(&gC, &g4)

		// ht, hy = 13, 9
		// c1 = ht**2+3*hy**2 = 412
	h1.Expc1(&gA)
	// c2 = ht+hy = 22
	h2.Expc2(&gB)
	h2g2C.CyclotomicSquare(&gC).
		Mul(&h2g2C, &h2)
	h4.CyclotomicSquare(&h2g2C).
		Mul(&h4, &h2g2C).
		CyclotomicSquare(&h4)
	result.Mul(&h1, &h4).
		Mul(&result, &f0_36)

	return result
}

// MillerLoop Optimal Tate alternative (or twisted ate or Eta revisited)
// computes the multi-Miller loop
// ∏ᵢ MillerLoop(Pᵢ, Qᵢ) =
// ∏ᵢ { fᵢ_{x₀+1+λ(x₀³-x₀²-x₀),Pᵢ}(Qᵢ) }
//
// Alg.2 in https://eprint.iacr.org/2021/1359.pdf
// Eq. (6) in https://hackmd.io/@gnark/BW6-761-changes
func MillerLoop(P []G1Affine, Q []G2Affine) (GT, error) {
	// check input size match
	n := len(P)
	if n == 0 || n != len(Q) {
		return GT{}, errors.New("invalid inputs sizes")
	}

	// filter infinity points
	p0 := make([]G1Affine, 0, n)
	q := make([]G2Affine, 0, n)

	for k := 0; k < n; k++ {
		if P[k].IsInfinity() || Q[k].IsInfinity() {
			continue
		}
		p0 = append(p0, P[k])
		q = append(q, Q[k])
	}

	n = len(q)

	// precomputations
	pProj1 := make([]g1Proj, n)
	p1 := make([]G1Affine, n)
	pProj01 := make([]g1Proj, n) // P0+P1
	pProj10 := make([]g1Proj, n) // P0-P1
	l01 := make([]lineEvaluation, n)
	l10 := make([]lineEvaluation, n)
	for k := 0; k < n; k++ {
		p1[k].Y.Neg(&p0[k].Y)
		p1[k].X.Mul(&p0[k].X, &thirdRootOneG2)
		pProj1[k].FromAffine(&p1[k])

		// l_{p0,p1}(q)
		pProj01[k].Set(&pProj1[k])
		pProj01[k].addMixedStep(&l01[k], &p0[k])
		l01[k].r1.Mul(&l01[k].r1, &q[k].X)
		l01[k].r0.Mul(&l01[k].r0, &q[k].Y)

		// l_{p0,-p1}(q)
		pProj10[k].Neg(&pProj1[k])
		pProj10[k].addMixedStep(&l10[k], &p0[k])
		l10[k].r1.Mul(&l10[k].r1, &q[k].X)
		l10[k].r0.Mul(&l10[k].r0, &q[k].Y)
	}
	p01 := BatchProjectiveToAffineG1(pProj01)
	p10 := BatchProjectiveToAffineG1(pProj10)

	// f_{a0+λ*a1,P}(Q)
	var result GT
	result.SetOne()
	var l, l0 lineEvaluation
	var prodLines [5]fp.Element

	var j int8

	if n >= 1 {
		// i = len(loopCounter0) - 2, separately to avoid an E12 Square
		// (Square(res) = 1² = 1)
		// j = 0
		// k = 0, separately to avoid MulBy034 (res × ℓ)
		// (assign line to res)

		// pProj1[0] ← 2pProj1[0] and l0 the tangent ℓ passing 2pProj1[0]
		pProj1[0].doubleStep(&l0)
		// line evaluation at Q[0] (assign)
		result.B1.A0.Mul(&l0.r1, &q[0].X)
		result.B0.A0.Mul(&l0.r0, &q[0].Y)
		result.B1.A1.Set(&l0.r2)
	}

	// k = 1
	if n >= 2 {
		// pProj1[1] ← 2pProj1[1] and l0 the tangent ℓ passing 2pProj1[1]
		pProj1[1].doubleStep(&l0)
		// line evaluation at Q[0]
		l0.r1.Mul(&l0.r1, &q[1].X)
		l0.r0.Mul(&l0.r0, &q[1].Y)
		// ℓ × res
		prodLines = fptower.Mul034By034(&l0.r0, &l0.r1, &l0.r2, &result.B0.A0, &result.B1.A0, &result.B1.A1)
		result.B0.A0 = prodLines[0]
		result.B0.A1 = prodLines[1]
		result.B0.A2 = prodLines[2]
		result.B1.A0 = prodLines[3]
		result.B1.A1 = prodLines[4]
	}

	// k >= 2
	for k := 2; k < n; k++ {
		// pProj1[1] ← 2pProj1[1] and l0 the tangent ℓ passing 2pProj1[1]
		pProj1[k].doubleStep(&l0)
		// line evaluation at Q[k]
		l0.r1.Mul(&l0.r1, &q[k].X)
		l0.r0.Mul(&l0.r0, &q[k].Y)
		// ℓ × res
		result.MulBy034(&l0.r0, &l0.r1, &l0.r2)
	}

	var tmp G1Affine
	for i := len(loopCounter0) - 3; i >= 1; i-- {
		// (∏ᵢfᵢ)²
		// mutualize the square among n Miller loops
		result.Square(&result)

		j = loopCounter1[i]*3 + loopCounter0[i]

		for k := 0; k < n; k++ {
			// pProj1[1] ← 2pProj1[1] and l0 the tangent ℓ passing 2pProj1[1]
			pProj1[k].doubleStep(&l0)
			// line evaluation at Q[k]
			l0.r1.Mul(&l0.r1, &q[k].X)
			l0.r0.Mul(&l0.r0, &q[k].Y)

			switch j {
			case -4:
				tmp.Neg(&p01[k])
				// pProj1[k] ← pProj1[k]-p01[k] and
				// l the line ℓ passing pProj1[k] and -p01[k]
				pProj1[k].addMixedStep(&l, &tmp)
				// line evaluation at Q[k]
				l.r1.Mul(&l.r1, &q[k].X)
				l.r0.Mul(&l.r0, &q[k].Y)
				// ℓ × ℓ
				prodLines = fptower.Mul034By034(&l.r0, &l.r1, &l.r2, &l01[k].r0, &l01[k].r1, &l01[k].r2)
				// ℓ × res
				result.MulBy034(&l0.r0, &l0.r1, &l0.r2)
				// (ℓ × ℓ) × res
				result.MulBy01234(&prodLines)
			case -3:
				tmp.Neg(&p1[k])
				// pProj1[k] ← pProj1[k]-p1[k] and
				// l the line ℓ passing pProj1[k] and -p1[k]
				pProj1[k].addMixedStep(&l, &tmp)
				// line evaluation at Q[k]
				l.r1.Mul(&l.r1, &q[k].X)
				l.r0.Mul(&l.r0, &q[k].Y)
				// ℓ × ℓ
				prodLines = fptower.Mul034By034(&l.r0, &l.r1, &l.r2, &l0.r0, &l0.r1, &l0.r2)
				// (ℓ × ℓ) × res
				result.MulBy01234(&prodLines)
			case -2:
				// pProj1[k] ← pProj1[k]+p10[k] and
				// l the line ℓ passing pProj1[k] and p10[k]
				pProj1[k].addMixedStep(&l, &p10[k])
				// line evaluation at Q[k]
				l.r1.Mul(&l.r1, &q[k].X)
				l.r0.Mul(&l.r0, &q[k].Y)
				// ℓ × ℓ
				prodLines = fptower.Mul034By034(&l.r0, &l.r1, &l.r2, &l01[k].r0, &l01[k].r1, &l01[k].r2)
				// ℓ × res
				result.MulBy034(&l0.r0, &l0.r1, &l0.r2)
				// (ℓ × ℓ) × res
				result.MulBy01234(&prodLines)
			case -1:
				tmp.Neg(&p0[k])
				// pProj1[k] ← pProj1[k]-p0[k] and
				// l the line ℓ passing pProj1[k] and -p0[k]
				pProj1[k].addMixedStep(&l, &tmp)
				// line evaluation at Q[k]
				l.r1.Mul(&l.r1, &q[k].X)
				l.r0.Mul(&l.r0, &q[k].Y)
				// ℓ × ℓ
				prodLines = fptower.Mul034By034(&l.r0, &l.r1, &l.r2, &l0.r0, &l0.r1, &l0.r2)
				// (ℓ × ℓ) × res
				result.MulBy01234(&prodLines)
			case 0:
				// ℓ × res
				result.MulBy034(&l0.r0, &l0.r1, &l0.r2)
			case 1:
				// pProj1[k] ← pProj1[k]+p0[k] and
				// l the line ℓ passing pProj1[k] and p0[k]
				pProj1[k].addMixedStep(&l, &p0[k])
				// line evaluation at Q[k]
				l.r1.Mul(&l.r1, &q[k].X)
				l.r0.Mul(&l.r0, &q[k].Y)
				// ℓ × ℓ
				prodLines = fptower.Mul034By034(&l.r0, &l.r1, &l.r2, &l0.r0, &l0.r1, &l0.r2)
				// (ℓ × ℓ) × res
				result.MulBy01234(&prodLines)
			case 2:
				tmp.Neg(&p10[k])
				// pProj1[k] ← pProj1[k]-p10[k] and
				// l the line ℓ passing pProj1[k] and -p10[k]
				pProj1[k].addMixedStep(&l, &tmp)
				// line evaluation at Q[k]
				l.r1.Mul(&l.r1, &q[k].X)
				l.r0.Mul(&l.r0, &q[k].Y)
				// ℓ × ℓ
				prodLines = fptower.Mul034By034(&l.r0, &l.r1, &l.r2, &l01[k].r0, &l01[k].r1, &l01[k].r2)
				// ℓ × res
				result.MulBy034(&l0.r0, &l0.r1, &l0.r2)
				// (ℓ × ℓ) × res
				result.MulBy01234(&prodLines)
			case 3:
				// pProj1[k] ← pProj1[k]+p1[k] and
				// l the line ℓ passing pProj1[k] and p1[k]
				pProj1[k].addMixedStep(&l, &p1[k])
				// line evaluation at Q[k]
				l.r1.Mul(&l.r1, &q[k].X)
				l.r0.Mul(&l.r0, &q[k].Y)
				// ℓ × ℓ
				prodLines = fptower.Mul034By034(&l.r0, &l.r1, &l.r2, &l0.r0, &l0.r1, &l0.r2)
				// (ℓ × ℓ) × res
				result.MulBy01234(&prodLines)
			case 4:
				// pProj1[k] ← pProj1[k]+p01[k] and
				// l the line ℓ passing pProj1[k] and p01[k]
				pProj1[k].addMixedStep(&l, &p01[k])
				// line evaluation at Q[k]
				l.r1.Mul(&l.r1, &q[k].X)
				l.r0.Mul(&l.r0, &q[k].Y)
				// ℓ × ℓ
				prodLines = fptower.Mul034By034(&l.r0, &l.r1, &l.r2, &l01[k].r0, &l01[k].r1, &l01[k].r2)
				// ℓ × res
				result.MulBy034(&l0.r0, &l0.r1, &l0.r2)
				// (ℓ × ℓ) × res
				result.MulBy01234(&prodLines)
			default:
				return GT{}, errors.New("invalid loopCounter")
			}
		}
	}

	// i = 0, separately to avoid a point addition
	// j = -3
	result.Square(&result)
	for k := 0; k < n; k++ {
		// pProj1[k] ← 2pProj1[k] and l0 the tangent ℓ passing 2pProj1[k]
		pProj1[k].doubleStep(&l0)
		// line evaluation at Q[k]
		l0.r1.Mul(&l0.r1, &q[k].X)
		l0.r0.Mul(&l0.r0, &q[k].Y)

		tmp.Neg(&p1[k])
		// l the line passing pProj1[k] and -p1
		pProj1[k].lineCompute(&l, &tmp)
		// line evaluation at Q[k]
		l.r1.Mul(&l.r1, &q[k].X)
		l.r0.Mul(&l.r0, &q[k].Y)
		// ℓ × ℓ
		prodLines = fptower.Mul034By034(&l.r0, &l.r1, &l.r2, &l0.r0, &l0.r1, &l0.r2)
		// (ℓ × ℓ) × res
		result.MulBy034(&l0.r0, &l0.r1, &l0.r2)
	}

	return result, nil
}

// doubleStep doubles a point in Homogenous projective coordinates, and evaluates the line in Miller loop
// https://eprint.iacr.org/2013/722.pdf (Section 4.3)
func (p *g1Proj) doubleStep(evaluations *lineEvaluation) {

	// get some Element from our pool
	var t1, A, B, C, D, E, EE, F, G, H, I, J, K fp.Element
	A.Mul(&p.x, &p.y)
	A.Halve()
	B.Square(&p.y)
	C.Square(&p.z)
	D.Double(&C).
		Add(&D, &C)

		// E.Mul(&D, &bCurveCoeff)
	E.Neg(&D)

	F.Double(&E).
		Add(&F, &E)
	G.Add(&B, &F)
	G.Halve()
	H.Add(&p.y, &p.z).
		Square(&H)
	t1.Add(&B, &C)
	H.Sub(&H, &t1)
	I.Sub(&E, &B)
	J.Square(&p.x)
	EE.Square(&E)
	K.Double(&EE).
		Add(&K, &EE)

	// X, Y, Z
	p.x.Sub(&B, &F).
		Mul(&p.x, &A)
	p.y.Square(&G).
		Sub(&p.y, &K)
	p.z.Mul(&B, &H)

	// Line evaluation
	evaluations.r0.Neg(&H)
	evaluations.r1.Double(&J).
		Add(&evaluations.r1, &J)
	evaluations.r2.Set(&I)
}

// addMixedStep point addition in Mixed Homogenous projective and Affine coordinates
// https://eprint.iacr.org/2013/722.pdf (Section 4.3)
func (p *g1Proj) addMixedStep(evaluations *lineEvaluation, a *G1Affine) {

	// get some Element from our pool
	var Y2Z1, X2Z1, O, L, C, D, E, F, G, H, t0, t1, t2, J fp.Element
	Y2Z1.Mul(&a.Y, &p.z)
	O.Sub(&p.y, &Y2Z1)
	X2Z1.Mul(&a.X, &p.z)
	L.Sub(&p.x, &X2Z1)
	C.Square(&O)
	D.Square(&L)
	E.Mul(&L, &D)
	F.Mul(&p.z, &C)
	G.Mul(&p.x, &D)
	t0.Double(&G)
	H.Add(&E, &F).
		Sub(&H, &t0)
	t1.Mul(&p.y, &E)

	// X, Y, Z
	p.x.Mul(&L, &H)
	p.y.Sub(&G, &H).
		Mul(&p.y, &O).
		Sub(&p.y, &t1)
	p.z.Mul(&E, &p.z)

	t2.Mul(&L, &a.Y)
	J.Mul(&a.X, &O).
		Sub(&J, &t2)

	// Line evaluation
	evaluations.r0.Set(&L)
	evaluations.r1.Neg(&O)
	evaluations.r2.Set(&J)
}

// lineCompute computes the line through p in Homogenous projective coordinates
// and a in affine coordinates. It does not compute the resulting point p+a.
func (p *g1Proj) lineCompute(evaluations *lineEvaluation, a *G1Affine) {

	// get some Element from our pool
	var Y2Z1, X2Z1, O, L, t2, J fp.Element
	Y2Z1.Mul(&a.Y, &p.z)
	O.Sub(&p.y, &Y2Z1)
	X2Z1.Mul(&a.X, &p.z)
	L.Sub(&p.x, &X2Z1)
	t2.Mul(&L, &a.Y)
	J.Mul(&a.X, &O).
		Sub(&J, &t2)

	// Line evaluation
	evaluations.r0.Set(&L)
	evaluations.r1.Neg(&O)
	evaluations.r2.Set(&J)
}
