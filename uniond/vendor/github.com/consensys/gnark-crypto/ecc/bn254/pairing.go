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

package bn254

import (
	"errors"

	"github.com/consensys/gnark-crypto/ecc/bn254/internal/fptower"
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
// where s is the cofactor 2x₀(6x₀²+3x₀+1) (Fuentes et al.)
func FinalExponentiation(z *GT, _z ...*GT) GT {

	var result GT
	result.Set(z)

	for _, e := range _z {
		result.Mul(&result, e)
	}

	var t [4]GT

	// Easy part
	// (p⁶-1)(p²+1)
	t[0].Conjugate(&result)
	result.Inverse(&result)
	t[0].Mul(&t[0], &result)
	result.FrobeniusSquare(&t[0]).
		Mul(&result, &t[0])

		// Hard part (up to permutation)
		// 2x₀(6x₀²+3x₀+1)(p⁴-p²+1)/r
	// Duquesne and Ghammam
	// https://eprint.iacr.org/2015/192.pdf
	// Fuentes et al. variant (alg. 10)
	t[0].Expt(&result).
		Conjugate(&t[0])
	t[0].CyclotomicSquare(&t[0])
	t[2].Expt(&t[0]).
		Conjugate(&t[2])
	t[1].CyclotomicSquare(&t[2])
	t[2].Mul(&t[2], &t[1])
	t[2].Mul(&t[2], &result)
	t[1].Expt(&t[2]).
		CyclotomicSquare(&t[1]).
		Mul(&t[1], &t[2]).
		Conjugate(&t[1])
	t[3].Conjugate(&t[1])
	t[1].CyclotomicSquare(&t[0])
	t[1].Mul(&t[1], &result)
	t[1].Conjugate(&t[1])
	t[1].Mul(&t[1], &t[3])
	t[0].Mul(&t[0], &t[1])
	t[2].Mul(&t[2], &t[1])
	t[3].FrobeniusSquare(&t[1])
	t[2].Mul(&t[2], &t[3])
	t[3].Conjugate(&result)
	t[3].Mul(&t[3], &t[0])
	t[1].FrobeniusCube(&t[3])
	t[2].Mul(&t[2], &t[1])
	t[1].Frobenius(&t[0])
	t[1].Mul(&t[1], &t[2])

	result.Set(&t[1])

	return result
}

// MillerLoop computes the multi-Miller loop
// ∏ᵢ MillerLoop(Pᵢ, Qᵢ)
func MillerLoop(P []G1Affine, Q []G2Affine) (GT, error) {
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

	var l, l0 lineEvaluation
	var tmp, result GT
	result.SetOne()

	// i == len(loopCounter) - 2
	for k := 0; k < n; k++ {
		qProj[k].DoubleStep(&l)
		// line evaluation
		l.r0.MulByElement(&l.r0, &p[k].Y)
		l.r1.MulByElement(&l.r1, &p[k].X)
		result.MulBy034(&l.r0, &l.r1, &l.r2)
	}

	for i := len(loopCounter) - 3; i >= 0; i-- {
		// (∏ᵢfᵢ)²
		result.Square(&result)

		for k := 0; k < n; k++ {
			qProj[k].DoubleStep(&l)
			// line evaluation
			l.r0.MulByElement(&l.r0, &p[k].Y)
			l.r1.MulByElement(&l.r1, &p[k].X)

			if loopCounter[i] == 1 {
				qProj[k].AddMixedStep(&l0, &q[k])
				// line evaluation
				l0.r0.MulByElement(&l0.r0, &p[k].Y)
				l0.r1.MulByElement(&l0.r1, &p[k].X)
				tmp.Mul034by034(&l.r0, &l.r1, &l.r2, &l0.r0, &l0.r1, &l0.r2)
				result.Mul(&result, &tmp)

			} else if loopCounter[i] == -1 {
				qProj[k].AddMixedStep(&l0, &qNeg[k])
				// line evaluation
				l0.r0.MulByElement(&l0.r0, &p[k].Y)
				l0.r1.MulByElement(&l0.r1, &p[k].X)
				tmp.Mul034by034(&l.r0, &l.r1, &l.r2, &l0.r0, &l0.r1, &l0.r2)
				result.Mul(&result, &tmp)
			} else {
				result.MulBy034(&l.r0, &l.r1, &l.r2)
			}
		}
	}

	var Q1, Q2 G2Affine
	for k := 0; k < n; k++ {
		//Q1 = π(Q)
		Q1.X.Conjugate(&q[k].X).MulByNonResidue1Power2(&Q1.X)
		Q1.Y.Conjugate(&q[k].Y).MulByNonResidue1Power3(&Q1.Y)

		// Q2 = -π²(Q)
		Q2.X.MulByNonResidue2Power2(&q[k].X)
		Q2.Y.MulByNonResidue2Power3(&q[k].Y).Neg(&Q2.Y)

		qProj[k].AddMixedStep(&l0, &Q1)
		l0.r0.MulByElement(&l0.r0, &p[k].Y)
		l0.r1.MulByElement(&l0.r1, &p[k].X)

		qProj[k].AddMixedStep(&l, &Q2)
		l.r0.MulByElement(&l.r0, &p[k].Y)
		l.r1.MulByElement(&l.r1, &p[k].X)
		tmp.Mul034by034(&l.r0, &l.r1, &l.r2, &l0.r0, &l0.r1, &l0.r2)
		result.Mul(&result, &tmp)
	}

	return result, nil
}

// DoubleStep doubles a point in Homogenous projective coordinates, and evaluates the line in Miller loop
// https://eprint.iacr.org/2013/722.pdf (Section 4.3)
func (p *g2Proj) DoubleStep(evaluations *lineEvaluation) {

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
	evaluations.r0.Neg(&H)
	evaluations.r1.Double(&J).
		Add(&evaluations.r1, &J)
	evaluations.r2.Set(&I)
}

// AddMixedStep point addition in Mixed Homogenous projective and Affine coordinates
// https://eprint.iacr.org/2013/722.pdf (Section 4.3)
func (p *g2Proj) AddMixedStep(evaluations *lineEvaluation, a *G2Affine) {

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
	evaluations.r0.Set(&L)
	evaluations.r1.Neg(&O)
	evaluations.r2.Set(&J)
}
