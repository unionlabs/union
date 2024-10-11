// Copyright 2020 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package bls12381

type pair struct {
	g1 *PointG1
	g2 *PointG2
}

func newPair(g1 *PointG1, g2 *PointG2) pair {
	return pair{g1, g2}
}

// Engine is BLS12-381 elliptic curve pairing engine
type Engine struct {
	G1   *G1
	G2   *G2
	fp12 *fp12
	fp2  *fp2
	pairingEngineTemp
	pairs []pair
}

// NewPairingEngine creates new pairing engine instance.
func NewPairingEngine() *Engine {
	fp2 := newFp2()
	fp6 := newFp6(fp2)
	fp12 := newFp12(fp6)
	g1 := NewG1()
	g2 := newG2(fp2)
	return &Engine{
		fp2:               fp2,
		fp12:              fp12,
		G1:                g1,
		G2:                g2,
		pairingEngineTemp: newEngineTemp(),
	}
}

type pairingEngineTemp struct {
	t2  [10]*fe2
	t12 [9]fe12
}

func newEngineTemp() pairingEngineTemp {
	t2 := [10]*fe2{}
	for i := 0; i < 10; i++ {
		t2[i] = &fe2{}
	}
	t12 := [9]fe12{}
	return pairingEngineTemp{t2, t12}
}

// AddPair adds a g1, g2 point pair to pairing engine
func (e *Engine) AddPair(g1 *PointG1, g2 *PointG2) *Engine {
	p := newPair(g1, g2)
	if !e.isZero(p) {
		e.affine(p)
		e.pairs = append(e.pairs, p)
	}
	return e
}

// AddPairInv adds a G1, G2 point pair to pairing engine. G1 point is negated.
func (e *Engine) AddPairInv(g1 *PointG1, g2 *PointG2) *Engine {
	e.G1.Neg(g1, g1)
	e.AddPair(g1, g2)
	return e
}

// Reset deletes added pairs.
func (e *Engine) Reset() *Engine {
	e.pairs = []pair{}
	return e
}

func (e *Engine) isZero(p pair) bool {
	return e.G1.IsZero(p.g1) || e.G2.IsZero(p.g2)
}

func (e *Engine) affine(p pair) {
	e.G1.Affine(p.g1)
	e.G2.Affine(p.g2)
}

func (e *Engine) doublingStep(coeff *[3]fe2, r *PointG2) {
	// Adaptation of Formula 3 in https://eprint.iacr.org/2010/526.pdf
	fp2 := e.fp2
	t := e.t2
	fp2.mul(t[0], &r[0], &r[1])
	fp2.mulByFq(t[0], t[0], twoInv)
	fp2.square(t[1], &r[1])
	fp2.square(t[2], &r[2])
	fp2.double(t[7], t[2])
	fp2.add(t[7], t[7], t[2])
	fp2.mulByB(t[3], t[7])
	fp2.double(t[4], t[3])
	fp2.add(t[4], t[4], t[3])
	fp2.add(t[5], t[1], t[4])
	fp2.mulByFq(t[5], t[5], twoInv)
	fp2.add(t[6], &r[1], &r[2])
	fp2.square(t[6], t[6])
	fp2.add(t[7], t[2], t[1])
	fp2.sub(t[6], t[6], t[7])
	fp2.sub(&coeff[0], t[3], t[1])
	fp2.square(t[7], &r[0])
	fp2.sub(t[4], t[1], t[4])
	fp2.mul(&r[0], t[4], t[0])
	fp2.square(t[2], t[3])
	fp2.double(t[3], t[2])
	fp2.add(t[3], t[3], t[2])
	fp2.square(t[5], t[5])
	fp2.sub(&r[1], t[5], t[3])
	fp2.mul(&r[2], t[1], t[6])
	fp2.double(t[0], t[7])
	fp2.add(&coeff[1], t[0], t[7])
	fp2.neg(&coeff[2], t[6])
}

func (e *Engine) additionStep(coeff *[3]fe2, r, q *PointG2) {
	// Algorithm 12 in https://eprint.iacr.org/2010/526.pdf
	fp2 := e.fp2
	t := e.t2
	fp2.mul(t[0], &q[1], &r[2])
	fp2.neg(t[0], t[0])
	fp2.add(t[0], t[0], &r[1])
	fp2.mul(t[1], &q[0], &r[2])
	fp2.neg(t[1], t[1])
	fp2.add(t[1], t[1], &r[0])
	fp2.square(t[2], t[0])
	fp2.square(t[3], t[1])
	fp2.mul(t[4], t[1], t[3])
	fp2.mul(t[2], &r[2], t[2])
	fp2.mul(t[3], &r[0], t[3])
	fp2.double(t[5], t[3])
	fp2.sub(t[5], t[4], t[5])
	fp2.add(t[5], t[5], t[2])
	fp2.mul(&r[0], t[1], t[5])
	fp2.sub(t[2], t[3], t[5])
	fp2.mul(t[2], t[2], t[0])
	fp2.mul(t[3], &r[1], t[4])
	fp2.sub(&r[1], t[2], t[3])
	fp2.mul(&r[2], &r[2], t[4])
	fp2.mul(t[2], t[1], &q[1])
	fp2.mul(t[3], t[0], &q[0])
	fp2.sub(&coeff[0], t[3], t[2])
	fp2.neg(&coeff[1], t[0])
	coeff[2].set(t[1])
}

func (e *Engine) preCompute(ellCoeffs *[68][3]fe2, twistPoint *PointG2) {
	// Algorithm 5 in  https://eprint.iacr.org/2019/077.pdf
	if e.G2.IsZero(twistPoint) {
		return
	}
	r := new(PointG2).Set(twistPoint)
	j := 0
	for i := x.BitLen() - 2; i >= 0; i-- {
		e.doublingStep(&ellCoeffs[j], r)
		if x.Bit(i) != 0 {
			j++
			ellCoeffs[j] = fe6{}
			e.additionStep(&ellCoeffs[j], r, twistPoint)
		}
		j++
	}
}

func (e *Engine) millerLoop(f *fe12) {
	pairs := e.pairs
	ellCoeffs := make([][68][3]fe2, len(pairs))
	for i := 0; i < len(pairs); i++ {
		e.preCompute(&ellCoeffs[i], pairs[i].g2)
	}
	fp12, fp2 := e.fp12, e.fp2
	t := e.t2
	f.one()
	j := 0
	for i := 62; /* x.BitLen() - 2 */ i >= 0; i-- {
		if i != 62 {
			fp12.square(f, f)
		}
		for i := 0; i <= len(pairs)-1; i++ {
			fp2.mulByFq(t[0], &ellCoeffs[i][j][2], &pairs[i].g1[1])
			fp2.mulByFq(t[1], &ellCoeffs[i][j][1], &pairs[i].g1[0])
			fp12.mulBy014Assign(f, &ellCoeffs[i][j][0], t[1], t[0])
		}
		if x.Bit(i) != 0 {
			j++
			for i := 0; i <= len(pairs)-1; i++ {
				fp2.mulByFq(t[0], &ellCoeffs[i][j][2], &pairs[i].g1[1])
				fp2.mulByFq(t[1], &ellCoeffs[i][j][1], &pairs[i].g1[0])
				fp12.mulBy014Assign(f, &ellCoeffs[i][j][0], t[1], t[0])
			}
		}
		j++
	}
	fp12.conjugate(f, f)
}

func (e *Engine) exp(c, a *fe12) {
	fp12 := e.fp12
	fp12.cyclotomicExp(c, a, x)
	fp12.conjugate(c, c)
}

func (e *Engine) finalExp(f *fe12) {
	fp12 := e.fp12
	t := e.t12
	// easy part
	fp12.frobeniusMap(&t[0], f, 6)
	fp12.inverse(&t[1], f)
	fp12.mul(&t[2], &t[0], &t[1])
	t[1].set(&t[2])
	fp12.frobeniusMapAssign(&t[2], 2)
	fp12.mulAssign(&t[2], &t[1])
	fp12.cyclotomicSquare(&t[1], &t[2])
	fp12.conjugate(&t[1], &t[1])
	// hard part
	e.exp(&t[3], &t[2])
	fp12.cyclotomicSquare(&t[4], &t[3])
	fp12.mul(&t[5], &t[1], &t[3])
	e.exp(&t[1], &t[5])
	e.exp(&t[0], &t[1])
	e.exp(&t[6], &t[0])
	fp12.mulAssign(&t[6], &t[4])
	e.exp(&t[4], &t[6])
	fp12.conjugate(&t[5], &t[5])
	fp12.mulAssign(&t[4], &t[5])
	fp12.mulAssign(&t[4], &t[2])
	fp12.conjugate(&t[5], &t[2])
	fp12.mulAssign(&t[1], &t[2])
	fp12.frobeniusMapAssign(&t[1], 3)
	fp12.mulAssign(&t[6], &t[5])
	fp12.frobeniusMapAssign(&t[6], 1)
	fp12.mulAssign(&t[3], &t[0])
	fp12.frobeniusMapAssign(&t[3], 2)
	fp12.mulAssign(&t[3], &t[1])
	fp12.mulAssign(&t[3], &t[6])
	fp12.mul(f, &t[3], &t[4])
}

func (e *Engine) calculate() *fe12 {
	f := e.fp12.one()
	if len(e.pairs) == 0 {
		return f
	}
	e.millerLoop(f)
	e.finalExp(f)
	return f
}

// Check computes pairing and checks if result is equal to one
func (e *Engine) Check() bool {
	return e.calculate().isOne()
}

// Result computes pairing and returns target group element as result.
func (e *Engine) Result() *E {
	r := e.calculate()
	e.Reset()
	return r
}

// GT returns target group instance.
func (e *Engine) GT() *GT {
	return NewGT()
}
