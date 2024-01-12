// Copyright 2020 ConsenSys AG
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

package bls12381

import (
	"errors"

	"github.com/consensys/gnark-crypto/ecc/bls12-381/fp"
	"github.com/consensys/gnark-crypto/ecc/bls12-381/internal/fptower"
)

// GT target group of the pairing
type GT = fptower.E12

type lineEvaluation struct {
	r0 fptower.E2
	r1 fptower.E2
	r2 fptower.E2
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
// where d = (p¹²-1)/r = (p¹²-1)/Φ₁₂(p) ⋅ Φ₁₂(p)/r = (p⁶-1)(p²+1)(p⁴ - p² +1)/r
// we use instead d=s ⋅ (p⁶-1)(p²+1)(p⁴ - p² +1)/r
// where s is the cofactor 3 (Hayashida et al.)
func FinalExponentiation(z *GT, _z ...*GT) GT {
	var result GT
	result.Set(z)

	for _, e := range _z {
		result.Mul(&result, e)
	}

	var t [3]GT

	// Easy part
	// (p⁶-1)(p²+1)
	t[0].Conjugate(&result)
	result.Inverse(&result)
	t[0].Mul(&t[0], &result)
	result.FrobeniusSquare(&t[0]).
		Mul(&result, &t[0])

	var one GT
	one.SetOne()
	if result.Equal(&one) {
		return result
	}

	// Hard part (up to permutation)
	// Daiki Hayashida, Kenichiro Hayasaka and Tadanori Teruya
	// https://eprint.iacr.org/2020/875.pdf
	t[0].CyclotomicSquare(&result)
	t[1].ExptHalf(&t[0])
	t[2].InverseUnitary(&result)
	t[1].Mul(&t[1], &t[2])
	t[2].Expt(&t[1])
	t[1].InverseUnitary(&t[1])
	t[1].Mul(&t[1], &t[2])
	t[2].Expt(&t[1])
	t[1].Frobenius(&t[1])
	t[1].Mul(&t[1], &t[2])
	result.Mul(&result, &t[0])
	t[0].Expt(&t[1])
	t[2].Expt(&t[0])
	t[0].FrobeniusSquare(&t[1])
	t[1].InverseUnitary(&t[1])
	t[1].Mul(&t[1], &t[2])
	t[1].Mul(&t[1], &t[0])
	result.Mul(&result, &t[1])

	return result
}

// MillerLoop computes the multi-Miller loop
// ∏ᵢ MillerLoop(Pᵢ, Qᵢ) = ∏ᵢ { fᵢ_{x,Qᵢ}(Pᵢ) }
func MillerLoop(P []G1Affine, Q []G2Affine) (GT, error) {
	// check input size match
	n := len(P)
	if n == 0 || n != len(Q) {
		return GT{}, errors.New("invalid inputs sizes")
	}

	// filter infinity points
	p := make([]G1Affine, 0, n)
	q := make([]G2Affine, 0, n)

	for k := 0; k < n; k++ {
		if P[k].IsInfinity() || Q[k].IsInfinity() {
			continue
		}
		p = append(p, P[k])
		q = append(q, Q[k])
	}

	n = len(p)

	// projective points for Q
	qProj := make([]g2Proj, n)
	for k := 0; k < n; k++ {
		qProj[k].FromAffine(&q[k])
	}

	var result GT
	result.SetOne()
	var l1, l2 lineEvaluation
	var prodLines [5]E2

	// Compute ∏ᵢ { fᵢ_{x₀,Q}(P) }
	if n >= 1 {
		// i = 62, separately to avoid an E12 Square
		// (Square(res) = 1² = 1)
		// LoopCounter[62] = 1
		// k = 0, separately to avoid MulBy014 (res × ℓ)
		// (assign line to res)

		// qProj[0] ← 2qProj[0] and l1 the tangent ℓ passing 2qProj[0]
		qProj[0].doubleStep(&l1)
		// line evaluation at P[0] (assign)
		result.C0.B0.Set(&l1.r0)
		result.C0.B1.MulByElement(&l1.r1, &p[0].X)
		result.C1.B1.MulByElement(&l1.r2, &p[0].Y)

		// qProj[0] ← qProj[0]+Q[0] and
		// l2 the line ℓ passing qProj[0] and Q[0]
		qProj[0].addMixedStep(&l2, &q[0])
		// line evaluation at P[0] (assign)
		l2.r1.MulByElement(&l2.r1, &p[0].X)
		l2.r2.MulByElement(&l2.r2, &p[0].Y)
		// ℓ × res
		prodLines = fptower.Mul014By014(&l2.r0, &l2.r1, &l2.r2, &result.C0.B0, &result.C0.B1, &result.C1.B1)
		result.C0.B0 = prodLines[0]
		result.C0.B1 = prodLines[1]
		result.C0.B2 = prodLines[2]
		result.C1.B1 = prodLines[3]
		result.C1.B2 = prodLines[4]
	}

	// k >= 1
	for k := 1; k < n; k++ {
		// qProj[k] ← 2qProj[k] and l1 the tangent ℓ passing 2qProj[k]
		qProj[k].doubleStep(&l1)
		// line evaluation at P[k]
		l1.r1.MulByElement(&l1.r1, &p[k].X)
		l1.r2.MulByElement(&l1.r2, &p[k].Y)

		// qProj[k] ← qProj[k]+Q[k] and
		// l2 the line ℓ passing qProj[k] and Q[k]
		qProj[k].addMixedStep(&l2, &q[k])
		// line evaluation at P[k]
		l2.r1.MulByElement(&l2.r1, &p[k].X)
		l2.r2.MulByElement(&l2.r2, &p[k].Y)
		// ℓ × ℓ
		prodLines = fptower.Mul014By014(&l2.r0, &l2.r1, &l2.r2, &l1.r0, &l1.r1, &l1.r2)
		// (ℓ × ℓ) × result
		result.MulBy01245(&prodLines)
	}

	// i <= 61
	for i := len(LoopCounter) - 3; i >= 1; i-- {
		// mutualize the square among n Miller loops
		// (∏ᵢfᵢ)²
		result.Square(&result)

		for k := 0; k < n; k++ {
			// qProj[k] ← 2qProj[k] and l1 the tangent ℓ passing 2qProj[k]
			qProj[k].doubleStep(&l1)
			// line evaluation at P[k]
			l1.r1.MulByElement(&l1.r1, &p[k].X)
			l1.r2.MulByElement(&l1.r2, &p[k].Y)

			if LoopCounter[i] == 0 {
				// ℓ × res
				result.MulBy014(&l1.r0, &l1.r1, &l1.r2)
			} else {
				// qProj[k] ← qProj[k]+Q[k] and
				// l2 the line ℓ passing qProj[k] and Q[k]
				qProj[k].addMixedStep(&l2, &q[k])
				// line evaluation at P[k]
				l2.r1.MulByElement(&l2.r1, &p[k].X)
				l2.r2.MulByElement(&l2.r2, &p[k].Y)
				// ℓ × ℓ
				prodLines = fptower.Mul014By014(&l2.r0, &l2.r1, &l2.r2, &l1.r0, &l1.r1, &l1.r2)
				// (ℓ × ℓ) × result
				result.MulBy01245(&prodLines)
			}
		}
	}

	// i = 0, separately to avoid a point doubling
	// LoopCounter[0] = 0
	result.Square(&result)
	for k := 0; k < n; k++ {
		// l1 the tangent ℓ passing 2qProj[k]
		qProj[k].tangentLine(&l1)
		// line evaluation at P[k]
		l1.r1.MulByElement(&l1.r1, &p[k].X)
		l1.r2.MulByElement(&l1.r2, &p[k].Y)
		// ℓ × result
		result.MulBy014(&l1.r0, &l1.r1, &l1.r2)
	}

	// negative x₀
	result.Conjugate(&result)

	return result, nil
}

// doubleStep doubles a point in Homogenous projective coordinates, and evaluates the line in Miller loop
// https://eprint.iacr.org/2013/722.pdf (Section 4.3)
func (p *g2Proj) doubleStep(l *lineEvaluation) {

	// get some Element from our pool
	var t1, A, B, C, D, E, EE, F, G, H, I, J, K fptower.E2
	A.Mul(&p.x, &p.y)
	A.Halve()
	B.Square(&p.y)
	C.Square(&p.z)
	D.Double(&C).
		Add(&D, &C)
	E.MulBybTwistCurveCoeff(&D)
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
	l.r0.Set(&I)
	l.r1.Double(&J).
		Add(&l.r1, &J)
	l.r2.Neg(&H)

}

// addMixedStep point addition in Mixed Homogenous projective and Affine coordinates
// https://eprint.iacr.org/2013/722.pdf (Section 4.3)
func (p *g2Proj) addMixedStep(l *lineEvaluation, a *G2Affine) {

	// get some Element from our pool
	var Y2Z1, X2Z1, O, L, C, D, E, F, G, H, t0, t1, t2, J fptower.E2
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
	l.r0.Set(&J)
	l.r1.Neg(&O)
	l.r2.Set(&L)
}

// tangentCompute computes the tangent through [2]p in Homogenous projective coordinates.
// It does not compute the resulting point [2]p.
func (p *g2Proj) tangentLine(l *lineEvaluation) {

	// get some Element from our pool
	var t1, B, C, D, E, H, I, J fptower.E2
	B.Square(&p.y)
	C.Square(&p.z)
	D.Double(&C).
		Add(&D, &C)
	E.MulBybTwistCurveCoeff(&D)
	H.Add(&p.y, &p.z).
		Square(&H)
	t1.Add(&B, &C)
	H.Sub(&H, &t1)
	I.Sub(&E, &B)
	J.Square(&p.x)

	// Line evaluation
	l.r0.Set(&I)
	l.r1.Double(&J).
		Add(&l.r1, &J)
	l.r2.Neg(&H)
}

// ----------------------
// Fixed-argument pairing
// ----------------------

type LineEvaluationAff struct {
	R0 fptower.E2
	R1 fptower.E2
}

// PairFixedQ calculates the reduced pairing for a set of points
// ∏ᵢ e(Pᵢ, Qᵢ) where Q are fixed points in G2.
//
// This function doesn't check that the inputs are in the correct subgroup. See IsInSubGroup.
func PairFixedQ(P []G1Affine, lines [][2][len(LoopCounter) - 1]LineEvaluationAff) (GT, error) {
	f, err := MillerLoopFixedQ(P, lines)
	if err != nil {
		return GT{}, err
	}
	return FinalExponentiation(&f), nil
}

// PairingCheckFixedQ calculates the reduced pairing for a set of points and returns True if the result is One
// ∏ᵢ e(Pᵢ, Qᵢ) =? 1 where Q are fixed points in G2.
//
// This function doesn't check that the inputs are in the correct subgroup. See IsInSubGroup.
func PairingCheckFixedQ(P []G1Affine, lines [][2][len(LoopCounter) - 1]LineEvaluationAff) (bool, error) {
	f, err := PairFixedQ(P, lines)
	if err != nil {
		return false, err
	}
	var one GT
	one.SetOne()
	return f.Equal(&one), nil
}

// PrecomputeLines precomputes the lines for the fixed-argument Miller loop
func PrecomputeLines(Q G2Affine) (PrecomputedLines [2][len(LoopCounter) - 1]LineEvaluationAff) {
	var accQ G2Affine
	accQ.Set(&Q)

	n := len(LoopCounter)
	for i := n - 2; i >= 0; i-- {
		accQ.doubleStep(&PrecomputedLines[0][i])
		if LoopCounter[i] == 0 {
			continue
		} else {
			accQ.addStep(&PrecomputedLines[1][i], &Q)
		}
	}
	return PrecomputedLines
}

// MillerLoopFixedQ computes the multi-Miller loop as in MillerLoop
// but Qᵢ are fixed points in G2 known in advance.
func MillerLoopFixedQ(P []G1Affine, lines [][2][len(LoopCounter) - 1]LineEvaluationAff) (GT, error) {
	n := len(P)
	if n == 0 || n != len(lines) {
		return GT{}, errors.New("invalid inputs sizes")
	}

	// no need to filter infinity points:
	// 		1. if Pᵢ=(0,0) then -x/y=1/y=0 by gnark-crypto convention and so
	// 		lines R0 and R1 are 0. It happens that result will stay, through
	// 		the Miller loop, in 𝔽p⁶ because MulBy01(0,0,1),
	// 		Mul01By01(0,0,1,0,0,1) and MulBy01245 set result.C0 to 0. At the
	// 		end result will be in a proper subgroup of Fp¹² so it be reduced to
	// 		1 in FinalExponentiation.
	//
	//      and/or
	//
	// 		2. if Qᵢ=(0,0) then PrecomputeLines(Qᵢ) will return lines R0 and R1
	// 		that are 0 because of gnark-convention (*/0==0) in doubleStep and
	// 		addStep. Similarly to Pᵢ=(0,0) it happens that result be 1
	// 		after the FinalExponentiation.

	// precomputations
	yInv := make([]fp.Element, n)
	xNegOverY := make([]fp.Element, n)
	for k := 0; k < n; k++ {
		yInv[k].Set(&P[k].Y)
	}
	yInv = fp.BatchInvert(yInv)
	for k := 0; k < n; k++ {
		xNegOverY[k].Mul(&P[k].X, &yInv[k]).
			Neg(&xNegOverY[k])
	}

	var result GT
	result.SetOne()
	var prodLines [5]E2

	// Compute ∏ᵢ { fᵢ_{x₀,Q}(P) }
	for i := len(LoopCounter) - 2; i >= 0; i-- {
		// mutualize the square among n Miller loops
		// (∏ᵢfᵢ)²
		result.Square(&result)

		for k := 0; k < n; k++ {
			// line evaluation at P[k]
			lines[k][0][i].R1.
				MulByElement(
					&lines[k][0][i].R1,
					&yInv[k],
				)
			lines[k][0][i].R0.
				MulByElement(&lines[k][0][i].R0,
					&xNegOverY[k],
				)
			if LoopCounter[i] == 0 {
				// ℓ × res
				result.MulBy01(
					&lines[k][0][i].R1,
					&lines[k][0][i].R0,
				)

			} else {
				lines[k][1][i].R1.
					MulByElement(
						&lines[k][1][i].R1,
						&yInv[k],
					)
				lines[k][1][i].R0.
					MulByElement(
						&lines[k][1][i].R0,
						&xNegOverY[k],
					)
				prodLines = fptower.Mul01By01(
					&lines[k][0][i].R1, &lines[k][0][i].R0,
					&lines[k][1][i].R1, &lines[k][1][i].R0,
				)
				result.MulBy01245(&prodLines)
			}
		}
	}

	// negative x₀
	result.Conjugate(&result)

	return result, nil
}

func (p *G2Affine) doubleStep(evaluations *LineEvaluationAff) {

	var n, d, λ, xr, yr fptower.E2
	// λ = 3x²/2y
	n.Square(&p.X)
	λ.Double(&n).
		Add(&λ, &n)
	d.Double(&p.Y)
	λ.Div(&λ, &d)

	// xr = λ²-2x
	xr.Square(&λ).
		Sub(&xr, &p.X).
		Sub(&xr, &p.X)

	// yr = λ(x-xr)-y
	yr.Sub(&p.X, &xr).
		Mul(&yr, &λ).
		Sub(&yr, &p.Y)

	evaluations.R0.Set(&λ)
	evaluations.R1.Mul(&λ, &p.X).
		Sub(&evaluations.R1, &p.Y)

	p.X.Set(&xr)
	p.Y.Set(&yr)
}

func (p *G2Affine) addStep(evaluations *LineEvaluationAff, a *G2Affine) {
	var n, d, λ, λλ, xr, yr fptower.E2

	// compute λ = (y2-y1)/(x2-x1)
	n.Sub(&a.Y, &p.Y)
	d.Sub(&a.X, &p.X)
	λ.Div(&n, &d)

	// xr = λ²-x1-x2
	λλ.Square(&λ)
	n.Add(&p.X, &a.X)
	xr.Sub(&λλ, &n)

	// yr = λ(x1-xr) - y1
	yr.Sub(&p.X, &xr).
		Mul(&yr, &λ).
		Sub(&yr, &p.Y)

	evaluations.R0.Set(&λ)
	evaluations.R1.Mul(&λ, &p.X).
		Sub(&evaluations.R1, &p.Y)

	p.X.Set(&xr)
	p.Y.Set(&yr)
}
