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

package bls24317

import (
	"errors"

	"github.com/consensys/gnark-crypto/ecc/bls24-317/internal/fptower"
)

// GT target group of the pairing
type GT = fptower.E24

type lineEvaluation struct {
	r0 fptower.E4
	r1 fptower.E4
	r2 fptower.E4
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
// where d = (p²⁴-1)/r = (p²⁴-1)/Φ₂₄(p) ⋅ Φ₂₄(p)/r = (p¹²-1)(p⁴+1)(p⁸ - p⁴ +1)/r
// we use instead d=s ⋅ (p¹²-1)(p⁴+1)(p⁸ - p⁴ +1)/r
// where s is the cofactor 3 (Hayashida et al.)
func FinalExponentiation(z *GT, _z ...*GT) GT {

	var result GT
	result.Set(z)

	for _, e := range _z {
		result.Mul(&result, e)
	}

	var t [9]GT

	// Easy part
	// (p¹²-1)(p⁴+1)
	t[0].Conjugate(&result)
	result.Inverse(&result)
	t[0].Mul(&t[0], &result)
	result.FrobeniusQuad(&t[0]).
		Mul(&result, &t[0])

	// hard part (up to permutation)
	// Daiki Hayashida and Kenichiro Hayasaka
	// and Tadanori Teruya
	// https://eprint.iacr.org/2020/875.pdf
	// 3(p⁸ - p⁴ +1)/r = (x₀-1)² * (x₀+p) * (x₀²+p²) * (x₀⁴+p⁴-1) + 3
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
	t[2].Mul(&t[0], &t[2])
	t[1].ExptHalf(&t[2])
	t[1].ExptHalf(&t[1])
	t[1].ExptHalf(&t[1])
	t[1].ExptHalf(&t[1])
	for s := 0; s < 4; s++ {
		t[1].CyclotomicSquareCompressed(&t[1])
	}
	t[1].DecompressKarabina(&t[1])
	t[0].FrobeniusQuad(&t[2])
	t[0].Mul(&t[0], &t[1])
	t[2].InverseUnitary(&t[2])
	t[0].Mul(&t[0], &t[2])
	result.Mul(&result, &t[0])

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
	qNeg := make([]G2Affine, n)
	for k := 0; k < n; k++ {
		qProj[k].FromAffine(&q[k])
		qNeg[k].Neg(&q[k])
	}

	var result GT
	result.SetOne()
	var l1, l2 lineEvaluation
	var prodLines [5]fptower.E4

	// Compute ∏ᵢ { fᵢ_{x₀,Q}(P) }
	if n >= 1 {
		// i = 31, separately to avoid an E12 Square
		// (Square(res) = 1² = 1)
		// loopCounter[31] = 0
		// k = 0, separately to avoid MulBy014 (res × ℓ)
		// (assign line to res)

		// qProj[0] ← 2qProj[0] and l1 the tangent ℓ passing 2qProj[0]
		qProj[0].doubleStep(&l1)
		// line evaluation at P[0] (assign)
		result.D0.C0.Set(&l1.r0)
		result.D0.C1.MulByElement(&l1.r1, &p[0].X)
		result.D1.C1.MulByElement(&l1.r2, &p[0].Y)
	}

	if n >= 2 {
		// k = 1, separately to avoid MulBy014 (res × ℓ)
		// (res is also a line at this point, so we use Mul014By014 ℓ × ℓ)

		// qProj[1] ← 2qProj[1] and l1 the tangent ℓ passing 2qProj[1]
		qProj[1].doubleStep(&l1)
		// line evaluation at P[1]
		l1.r1.MulByElement(&l1.r1, &p[1].X)
		l1.r2.MulByElement(&l1.r2, &p[1].Y)
		// ℓ × res
		prodLines = fptower.Mul014By014(&l1.r0, &l1.r1, &l1.r2, &result.D0.C0, &result.D0.C1, &result.D1.C1)
		result.D0.C0 = prodLines[0]
		result.D0.C1 = prodLines[1]
		result.D0.C2 = prodLines[2]
		result.D1.C1 = prodLines[3]
		result.D1.C2 = prodLines[4]
	}

	// k >= 2
	for k := 2; k < n; k++ {
		// qProj[k] ← 2qProj[k] and l1 the tangent ℓ passing 2qProj[k]
		qProj[k].doubleStep(&l1)
		// line evaluation at P[k]
		l1.r1.MulByElement(&l1.r1, &p[k].X)
		l1.r2.MulByElement(&l1.r2, &p[k].Y)
		// ℓ × res
		result.MulBy014(&l1.r0, &l1.r1, &l1.r2)
	}

	// i <= 30
	for i := len(loopCounter) - 3; i >= 1; i-- {
		// mutualize the square among n Miller loops
		// (∏ᵢfᵢ)²
		result.Square(&result)

		for k := 0; k < n; k++ {
			// qProj[k] ← 2qProj[k] and l1 the tangent ℓ passing 2qProj[k]
			qProj[k].doubleStep(&l1)
			// line evaluation at P[k]
			l1.r1.MulByElement(&l1.r1, &p[k].X)
			l1.r2.MulByElement(&l1.r2, &p[k].Y)

			if loopCounter[i] == 1 {
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
			} else if loopCounter[i] == -1 {
				// qProj[k] ← qProj[k]-Q[k] and
				// l2 the line ℓ passing qProj[k] and -Q[k]
				qProj[k].addMixedStep(&l2, &qNeg[k])
				// line evaluation at P[k]
				l2.r1.MulByElement(&l2.r1, &p[k].X)
				l2.r2.MulByElement(&l2.r2, &p[k].Y)
				// ℓ × ℓ
				prodLines = fptower.Mul014By014(&l2.r0, &l2.r1, &l2.r2, &l1.r0, &l1.r1, &l1.r2)
				// (ℓ × ℓ) × result
				result.MulBy01245(&prodLines)
			} else {
				// ℓ × result
				result.MulBy014(&l1.r0, &l1.r1, &l1.r2)
			}
		}
	}

	// i = 0, separately to avoid a point doubling
	// loopCounter[0] = 0
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

	return result, nil
}

// doubleStep doubles a point in Homogenous projective coordinates, and evaluates the line in Miller loop
// https://eprint.iacr.org/2013/722.pdf (Section 4.3)
func (p *g2Proj) doubleStep(evaluations *lineEvaluation) {

	// get some Element from our pool
	var t1, A, B, C, D, E, EE, F, G, H, I, J, K fptower.E4
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
	evaluations.r0.Set(&I)
	evaluations.r1.Double(&J).
		Add(&evaluations.r1, &J)
	evaluations.r2.Neg(&H)
}

// addMixedStep point addition in Mixed Homogenous projective and Affine coordinates
// https://eprint.iacr.org/2013/722.pdf (Section 4.3)
func (p *g2Proj) addMixedStep(evaluations *lineEvaluation, a *G2Affine) {

	// get some Element from our pool
	var Y2Z1, X2Z1, O, L, C, D, E, F, G, H, t0, t1, t2, J fptower.E4
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
	evaluations.r0.Set(&J)
	evaluations.r1.Neg(&O)
	evaluations.r2.Set(&L)
}

// tangentCompute computes the tangent through [2]p in Homogenous projective coordinates.
// It does not compute the resulting point [2]p.
func (p *g2Proj) tangentLine(l *lineEvaluation) {

	// get some Element from our pool
	var t1, B, C, D, E, H, I, J fptower.E4
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
