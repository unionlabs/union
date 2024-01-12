/*
Copyright © 2021 ConsenSys Software Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

package scs

import (
	"fmt"
	"math/big"

	"github.com/consensys/gnark/debug"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/internal/expr"
	"github.com/consensys/gnark/std/math/bits"
)

// AssertIsEqual fails if i1 != i2
func (builder *builder) AssertIsEqual(i1, i2 frontend.Variable) {

	c1, i1Constant := builder.constantValue(i1)
	c2, i2Constant := builder.constantValue(i2)

	if i1Constant && i2Constant {
		if c1 != c2 {
			panic("i1, i2 should be equal")
		}
		return
	}
	if i1Constant {
		i1, i2 = i2, i1
		i2Constant = i1Constant
		c2 = c1
	}
	if i2Constant {
		xa := i1.(expr.Term)
		c2 := builder.cs.Neg(c2)

		// xa - i2 == 0
		toAdd := sparseR1C{
			xa: xa.VID,
			qL: xa.Coeff,
			qC: c2,
		}

		if debug.Debug {
			debug := builder.newDebugInfo("assertIsEqual", xa, "==", i2)
			builder.addPlonkConstraint(toAdd, debug)
		} else {
			builder.addPlonkConstraint(toAdd)
		}
		return
	}
	xa := i1.(expr.Term)
	xb := i2.(expr.Term)

	xb.Coeff = builder.cs.Neg(xb.Coeff)
	// xa - xb == 0
	toAdd := sparseR1C{
		xa: xa.VID,
		xb: xb.VID,
		qL: xa.Coeff,
		qR: xb.Coeff,
	}

	if debug.Debug {
		xb.Coeff = builder.cs.Neg(xb.Coeff)
		debug := builder.newDebugInfo("assertIsEqual", xa, " == ", xb)
		builder.addPlonkConstraint(toAdd, debug)
	} else {
		builder.addPlonkConstraint(toAdd)
	}
}

// AssertIsDifferent fails if i1 == i2
func (builder *builder) AssertIsDifferent(i1, i2 frontend.Variable) {
	s := builder.Sub(i1, i2)
	if c, ok := builder.constantValue(s); ok && c.IsZero() {
		panic("AssertIsDifferent(x,x) will never be satisfied")
	} else if t := s.(expr.Term); t.Coeff.IsZero() {
		panic("AssertIsDifferent(x,x) will never be satisfied")
	}
	builder.Inverse(s)
}

// AssertIsBoolean fails if v != 0 ∥ v != 1
func (builder *builder) AssertIsBoolean(i1 frontend.Variable) {
	if c, ok := builder.constantValue(i1); ok {
		if !(c.IsZero() || builder.cs.IsOne(c)) {
			panic(fmt.Sprintf("assertIsBoolean failed: constant(%s)", builder.cs.String(c)))
		}
		return
	}

	v := i1.(expr.Term)
	if builder.IsBoolean(v) {
		return
	}
	builder.MarkBoolean(v)

	// ensure v * (1 - v) == 0
	// that is v + -v*v == 0
	// qM = -v.Coeff*v.Coeff
	qM := builder.cs.Neg(v.Coeff)
	qM = builder.cs.Mul(qM, v.Coeff)
	toAdd := sparseR1C{
		xa: v.VID,
		qL: v.Coeff,
		qM: qM,
	}
	if debug.Debug {
		debug := builder.newDebugInfo("assertIsBoolean", v, " == (0|1)")
		builder.addBoolGate(toAdd, debug)
	} else {
		builder.addBoolGate(toAdd)
	}

}

func (builder *builder) AssertIsCrumb(i1 frontend.Variable) {
	const errorMsg = "AssertIsCrumb: input is not a crumb"
	if c, ok := builder.constantValue(i1); ok {
		if i, ok := builder.cs.Uint64(c); ok && i < 4 {
			return
		}
		panic(errorMsg)
	}

	// i1 (i1-1) (i1-2) (i1-3) = (i1² - 3i1) (i1² - 3i1 + 2)
	// take X := i1² - 3i1 and we get X (X+2) = 0

	x := builder.MulAcc(builder.Mul(-3, i1), i1, i1).(expr.Term)

	// TODO @Tabaie Ideally this entire function would live in std/math/bits as it is quite specialized;
	// however using two generic MulAccs and an AssertIsEqual results in three constraints rather than two.
	builder.addPlonkConstraint(sparseR1C{
		xa: x.VID,
		xb: x.VID,
		qL: builder.cs.FromInterface(2),
		qM: builder.tOne,
	})
}

// AssertIsLessOrEqual fails if  v > bound
func (builder *builder) AssertIsLessOrEqual(v frontend.Variable, bound frontend.Variable) {
	cv, vConst := builder.constantValue(v)
	cb, bConst := builder.constantValue(bound)

	// both inputs are constants
	if vConst && bConst {
		bv, bb := builder.cs.ToBigInt(cv), builder.cs.ToBigInt(cb)
		if bv.Cmp(bb) == 1 {
			panic(fmt.Sprintf("AssertIsLessOrEqual: %s > %s", bv.String(), bb.String()))
		}
	}

	nbBits := builder.cs.FieldBitLen()
	vBits := bits.ToBinary(builder, v, bits.WithNbDigits(nbBits), bits.WithUnconstrainedOutputs())

	// bound is constant
	if bConst {
		builder.MustBeLessOrEqCst(vBits, builder.cs.ToBigInt(cb), v)
		return
	}

	if b, ok := bound.(expr.Term); ok {
		builder.mustBeLessOrEqVar(v, b)
	} else {
		panic(fmt.Sprintf("expected bound type expr.Term, got %T", bound))
	}
}

func (builder *builder) mustBeLessOrEqVar(a frontend.Variable, bound expr.Term) {

	debug := builder.newDebugInfo("mustBeLessOrEq", a, " <= ", bound)

	nbBits := builder.cs.FieldBitLen()

	aBits := bits.ToBinary(builder, a, bits.WithNbDigits(nbBits), bits.WithUnconstrainedOutputs(), bits.OmitModulusCheck())
	boundBits := bits.ToBinary(builder, bound, bits.WithNbDigits(nbBits)) // enforces range check against modulus

	p := make([]frontend.Variable, nbBits+1)
	p[nbBits] = 1

	for i := nbBits - 1; i >= 0; i-- {

		// if bound[i] == 0
		// 		p[i] = p[i+1]
		//		t = p[i+1]
		// else
		// 		p[i] = p[i+1] * a[i]
		//		t = 0
		v := builder.Mul(p[i+1], aBits[i])
		p[i] = builder.Select(boundBits[i], v, p[i+1])

		t := builder.Select(boundBits[i], 0, p[i+1])

		// (1 - t - ai) * ai == 0
		l := builder.Sub(1, t, aBits[i]).(expr.Term)

		// note if bound[i] == 1, this constraint is (1 - ai) * ai == 0
		// → this is a boolean constraint
		// if bound[i] == 0, t must be 0 or 1, thus ai must be 0 or 1 too

		if ai, ok := builder.constantValue(aBits[i]); ok {
			// a is constant; ensure l == 0
			l.Coeff = builder.cs.Mul(l.Coeff, ai)
			builder.addPlonkConstraint(sparseR1C{
				xa: l.VID,
				qL: l.Coeff,
			}, debug)
		} else {
			// l * a[i] == 0
			builder.addPlonkConstraint(sparseR1C{
				xa: l.VID,
				xb: aBits[i].(expr.Term).VID,
				qM: l.Coeff,
			}, debug)
		}

	}

}

// MustBeLessOrEqCst asserts that value represented using its bit decomposition
// aBits is less or equal than constant bound. The method boolean constraints
// the bits in aBits, so the caller can provide unconstrained bits.
func (builder *builder) MustBeLessOrEqCst(aBits []frontend.Variable, bound *big.Int, aForDebug frontend.Variable) {

	nbBits := builder.cs.FieldBitLen()
	if len(aBits) > nbBits {
		panic("more input bits than field bit length")
	}
	for i := len(aBits); i < nbBits; i++ {
		aBits = append(aBits, 0)
	}

	// ensure the bound is positive, it's bit-len doesn't matter
	if bound.Sign() == -1 {
		panic("AssertIsLessOrEqual: bound must be positive")
	}
	if bound.BitLen() > nbBits {
		panic("AssertIsLessOrEqual: bound is too large, constraint will never be satisfied")
	}

	// debug info
	debug := builder.newDebugInfo("mustBeLessOrEq", aForDebug, " <= ", bound)

	// t trailing bits in the bound
	t := 0
	for i := 0; i < nbBits; i++ {
		if bound.Bit(i) == 0 {
			break
		}
		t++
	}

	p := make([]frontend.Variable, nbBits+1)
	// p[i] == 1 → a[j] == c[j] for all j ⩾ i
	p[nbBits] = 1

	for i := nbBits - 1; i >= t; i-- {
		if bound.Bit(i) == 0 {
			p[i] = p[i+1]
		} else {
			p[i] = builder.Mul(p[i+1], aBits[i])
		}
	}

	for i := nbBits - 1; i >= 0; i-- {

		if bound.Bit(i) == 0 {
			// (1 - p(i+1) - ai) * ai == 0
			l := builder.Sub(1, p[i+1], aBits[i]).(expr.Term)
			//l = builder.Sub(l, ).(term)

			builder.addPlonkConstraint(sparseR1C{
				xa: l.VID,
				xb: aBits[i].(expr.Term).VID,
				qM: builder.tOne,
			}, debug)
		} else {
			builder.AssertIsBoolean(aBits[i])
		}
	}

}
