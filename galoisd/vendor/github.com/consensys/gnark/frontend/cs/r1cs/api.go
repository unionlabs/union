/*
Copyright © 2020 ConsenSys

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

package r1cs

import (
	"errors"
	"fmt"
	"github.com/consensys/gnark/frontend/cs"
	"path/filepath"
	"reflect"
	"runtime"
	"strings"

	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/internal/expr"
	"github.com/consensys/gnark/frontend/schema"
	"github.com/consensys/gnark/std/math/bits"
)

// ---------------------------------------------------------------------------------------------
// Arithmetic

// Add returns res = i1+i2+...in
func (builder *builder) Add(i1, i2 frontend.Variable, in ...frontend.Variable) frontend.Variable {
	// extract frontend.Variables from input
	vars, s := builder.toVariables(append([]frontend.Variable{i1, i2}, in...)...)
	return builder.add(vars, false, s, nil)
}

func (builder *builder) MulAcc(a, b, c frontend.Variable) frontend.Variable {
	// do the multiplication into builder.mbuf1
	mulBC := func() {
		// reset the buffer
		builder.mbuf1 = builder.mbuf1[:0]

		n1, v1Constant := builder.constantValue(b)
		n2, v2Constant := builder.constantValue(c)

		// v1 and v2 are both unknown, this is the only case we add a constraint
		if !v1Constant && !v2Constant {
			res := builder.newInternalVariable()
			builder.cs.AddConstraint(builder.newR1C(b, c, res))
			builder.mbuf1 = append(builder.mbuf1, res...)
			return
		}

		// v1 and v2 are constants, we multiply big.Int values and return resulting constant
		if v1Constant && v2Constant {
			builder.cs.Mul(&n1, &n2)
			builder.mbuf1 = append(builder.mbuf1, expr.NewTerm(0, n1))
			return
		}

		if v1Constant {
			builder.mbuf1 = append(builder.mbuf1, builder.toVariable(c)...)
			builder.mulConstant(builder.mbuf1, n1, true)
			return
		}
		builder.mbuf1 = append(builder.mbuf1, builder.toVariable(b)...)
		builder.mulConstant(builder.mbuf1, n2, true)
	}
	mulBC()

	_a := builder.toVariable(a)
	// copy _a in buffer, use _a as result; so if _a was already a linear expression and
	// results fits, _a is mutated without performing a new memalloc
	builder.mbuf2 = builder.mbuf2[:0]
	builder.add([]expr.LinearExpression{_a, builder.mbuf1}, false, 0, &builder.mbuf2)
	_a = _a[:0]
	if len(builder.mbuf2) <= cap(_a) {
		// it fits, no mem alloc
		_a = append(_a, builder.mbuf2...)
	} else {
		// allocate a expression linear with extended capacity
		_a = make(expr.LinearExpression, len(builder.mbuf2), len(builder.mbuf2)*3)
		copy(_a, builder.mbuf2)
	}
	return _a
}

// Sub returns res = i1 - i2
func (builder *builder) Sub(i1, i2 frontend.Variable, in ...frontend.Variable) frontend.Variable {
	// extract frontend.Variables from input
	vars, s := builder.toVariables(append([]frontend.Variable{i1, i2}, in...)...)
	return builder.add(vars, true, s, nil)
}

// returns res = Σ(vars) or res = vars[0] - Σ(vars[1:]) if sub == true.
func (builder *builder) add(vars []expr.LinearExpression, sub bool, capacity int, res *expr.LinearExpression) frontend.Variable {
	// we want to merge all terms from input linear expressions
	// if they are duplicate, we reduce; that is, if multiple terms in different vars have the
	// same variable id.

	// the frontend/ only builds linear expression that are sorted.
	// we build a sorted output by iterating all the lists in order and dealing
	// with the edge cases (same variable ID, coeff == 0, etc.)

	// initialize the min-heap

	for lID, v := range vars {
		builder.heap = append(builder.heap, linMeta{val: v[0].VID, lID: lID})
	}
	builder.heap.heapify()

	if res == nil {
		t := make(expr.LinearExpression, 0, capacity)
		res = &t
	}
	curr := -1

	// process all the terms from all the inputs, in sorted order
	for len(builder.heap) > 0 {
		lID, tID := builder.heap[0].lID, builder.heap[0].tID
		if tID == len(vars[lID])-1 {
			// last element, we remove it from the heap.
			builder.heap.popHead()
		} else {
			// increment and fix the heap
			builder.heap[0].tID++
			builder.heap[0].val = vars[lID][tID+1].VID
			builder.heap.fix(0)
		}
		t := &vars[lID][tID]
		if t.Coeff.IsZero() {
			continue // is this really needed?
		}
		if curr != -1 && t.VID == (*res)[curr].VID {
			// accumulate, it's the same variable ID
			if sub && lID != 0 {
				builder.cs.Sub(&(*res)[curr].Coeff, &t.Coeff)
			} else {
				builder.cs.Add(&(*res)[curr].Coeff, &t.Coeff)
			}
			if (*res)[curr].Coeff.IsZero() {
				// remove self.
				(*res) = (*res)[:curr]
				curr--
			}
		} else {
			// append, it's a new variable ID
			(*res) = append((*res), *t)
			curr++
			if sub && lID != 0 {
				builder.cs.Neg(&(*res)[curr].Coeff)
			}
		}
	}

	if len((*res)) == 0 {
		// keep the linear expression valid (assertIsSet)
		(*res) = append((*res), expr.NewTerm(0, constraint.Coeff{}))
	}
	// if the linear expression LE is too long then record an equality
	// constraint LE * 1 = t and return short linear expression instead.
	compressed := builder.compress((*res))
	if len(compressed) != len(*res) {
		// we compressed, but don't want to override buffer
		*res = (*res)[:0]
		*res = append(*res, compressed...)
	}

	return *res
}

// Neg returns -i
func (builder *builder) Neg(i frontend.Variable) frontend.Variable {
	v := builder.toVariable(i)

	if n, ok := builder.constantValue(v); ok {
		builder.cs.Neg(&n)
		return expr.NewLinearExpression(0, n)
	}

	return builder.negateLinExp(v)
}

// Mul returns res = i1 * i2 * ... in
func (builder *builder) Mul(i1, i2 frontend.Variable, in ...frontend.Variable) frontend.Variable {
	vars, _ := builder.toVariables(append([]frontend.Variable{i1, i2}, in...)...)

	mul := func(v1, v2 expr.LinearExpression, first bool) expr.LinearExpression {

		n1, v1Constant := builder.constantValue(v1)
		n2, v2Constant := builder.constantValue(v2)

		// v1 and v2 are both unknown, this is the only case we add a constraint
		if !v1Constant && !v2Constant {
			res := builder.newInternalVariable()
			builder.cs.AddConstraint(builder.newR1C(v1, v2, res))
			return res
		}

		// v1 and v2 are constants, we multiply big.Int values and return resulting constant
		if v1Constant && v2Constant {
			builder.cs.Mul(&n1, &n2)
			return expr.NewLinearExpression(0, n1)
		}

		if v1Constant {
			return builder.mulConstant(v2, n1, false)
		}
		return builder.mulConstant(v1, n2, !first)
	}

	res := mul(vars[0], vars[1], true)

	for i := 2; i < len(vars); i++ {
		res = mul(res, vars[i], false)
	}

	return res
}

func (builder *builder) mulConstant(v1 expr.LinearExpression, lambda constraint.Coeff, inPlace bool) expr.LinearExpression {
	// multiplying a frontend.Variable by a constant -> we updated the coefficients in the linear expression
	// leading to that frontend.Variable
	var res expr.LinearExpression
	if inPlace {
		res = v1
	} else {
		res = v1.Clone()
	}

	for i := 0; i < len(res); i++ {
		builder.cs.Mul(&res[i].Coeff, &lambda)
	}
	return res
}

func (builder *builder) DivUnchecked(i1, i2 frontend.Variable) frontend.Variable {
	vars, _ := builder.toVariables(i1, i2)

	v1 := vars[0]
	v2 := vars[1]

	n1, v1Constant := builder.constantValue(v1)
	n2, v2Constant := builder.constantValue(v2)

	if !v2Constant {
		res := builder.newInternalVariable()
		debug := builder.newDebugInfo("div", v1, "/", v2, " == ", res)
		// note that here we don't ensure that divisor is != 0
		builder.cs.AddConstraint(builder.newR1C(v2, res, v1), debug)
		return res
	}

	// v2 is constant
	if n2.IsZero() {
		panic("div by constant(0)")
	}
	// q := builder.q
	builder.cs.Inverse(&n2)
	// n2.ModInverse(n2, q)

	if v1Constant {
		builder.cs.Mul(&n2, &n1)
		return expr.NewLinearExpression(0, n2)
	}

	// v1 is not constant
	return builder.mulConstant(v1, n2, false)
}

// Div returns res = i1 / i2
func (builder *builder) Div(i1, i2 frontend.Variable) frontend.Variable {
	vars, _ := builder.toVariables(i1, i2)

	v1 := vars[0]
	v2 := vars[1]

	n1, v1Constant := builder.constantValue(v1)
	n2, v2Constant := builder.constantValue(v2)

	if !v2Constant {
		res := builder.newInternalVariable()
		debug := builder.newDebugInfo("div", v1, "/", v2, " == ", res)
		v2Inv := builder.newInternalVariable()
		// note that here we ensure that v2 can't be 0, but it costs us one extra constraint
		c1 := builder.cs.AddConstraint(builder.newR1C(v2, v2Inv, builder.cstOne()))
		c2 := builder.cs.AddConstraint(builder.newR1C(v1, v2Inv, res))
		builder.cs.AttachDebugInfo(debug, []int{c1, c2})
		return res
	}

	// v2 is constant
	if n2.IsZero() {
		panic("div by constant(0)")
	}
	builder.cs.Inverse(&n2)

	if v1Constant {
		builder.cs.Mul(&n2, &n1)
		return expr.NewLinearExpression(0, n2)
	}

	// v1 is not constant
	return builder.mulConstant(v1, n2, false)
}

// Inverse returns res = inverse(v)
func (builder *builder) Inverse(i1 frontend.Variable) frontend.Variable {
	vars, _ := builder.toVariables(i1)

	if c, ok := builder.constantValue(vars[0]); ok {
		if c.IsZero() {
			panic("inverse by constant(0)")
		}

		builder.cs.Inverse(&c)
		return expr.NewLinearExpression(0, c)
	}

	// allocate resulting frontend.Variable
	res := builder.newInternalVariable()

	debug := builder.newDebugInfo("inverse", vars[0], "*", res, " == 1")
	builder.cs.AddConstraint(builder.newR1C(res, vars[0], builder.cstOne()), debug)

	return res
}

// ---------------------------------------------------------------------------------------------
// Bit operations

// ToBinary unpacks a frontend.Variable in binary,
// n is the number of bits to select (starting from lsb)
// n default value is fr.Bits the number of bits needed to represent a field element
//
// The result in in little endian (first bit= lsb)
func (builder *builder) ToBinary(i1 frontend.Variable, n ...int) []frontend.Variable {
	// nbBits
	nbBits := builder.cs.FieldBitLen()
	if len(n) == 1 {
		nbBits = n[0]
		if nbBits < 0 {
			panic("invalid n")
		}
	}

	return bits.ToBinary(builder, i1, bits.WithNbDigits(nbBits))
}

// FromBinary packs b, seen as a fr.Element in little endian
func (builder *builder) FromBinary(_b ...frontend.Variable) frontend.Variable {
	return bits.FromBinary(builder, _b)
}

// Xor compute the XOR between two frontend.Variables
func (builder *builder) Xor(_a, _b frontend.Variable) frontend.Variable {

	vars, _ := builder.toVariables(_a, _b)

	a := vars[0]
	b := vars[1]

	builder.AssertIsBoolean(a)
	builder.AssertIsBoolean(b)

	// instead of writing a + b - 2ab
	// we do a * (1 - 2b) + b
	// to limit large linear expressions

	// moreover, we ensure than b is as small as possible, so that the result
	// is bounded by len(min(a, b)) + 1
	if len(b) > len(a) {
		a, b = b, a
	}
	t := builder.Sub(builder.cstOne(), builder.Mul(b, 2))
	t = builder.Add(builder.Mul(a, t), b)

	builder.MarkBoolean(t)

	return t
}

// Or compute the OR between two frontend.Variables
func (builder *builder) Or(_a, _b frontend.Variable) frontend.Variable {
	vars, _ := builder.toVariables(_a, _b)

	a := vars[0]
	b := vars[1]

	builder.AssertIsBoolean(a)
	builder.AssertIsBoolean(b)

	// the formulation used is for easing up the conversion to sparse r1cs
	res := builder.newInternalVariable()
	builder.MarkBoolean(res)
	c := builder.Neg(res).(expr.LinearExpression)

	c = append(c, a...)
	c = append(c, b...)
	builder.cs.AddConstraint(builder.newR1C(a, b, c))

	return res
}

// And compute the AND between two frontend.Variables
func (builder *builder) And(_a, _b frontend.Variable) frontend.Variable {
	vars, _ := builder.toVariables(_a, _b)

	a := vars[0]
	b := vars[1]

	builder.AssertIsBoolean(a)
	builder.AssertIsBoolean(b)

	res := builder.Mul(a, b)
	builder.MarkBoolean(res)

	return res
}

// ---------------------------------------------------------------------------------------------
// Conditionals

// Select if i0 is true, yields i1 else yields i2
func (builder *builder) Select(i0, i1, i2 frontend.Variable) frontend.Variable {

	vars, _ := builder.toVariables(i0, i1, i2)
	cond := vars[0]

	// ensures that cond is boolean
	builder.AssertIsBoolean(cond)

	if c, ok := builder.constantValue(cond); ok {
		// condition is a constant return i1 if true, i2 if false
		if builder.isCstOne(&c) {
			return vars[1]
		}
		return vars[2]
	}

	n1, ok1 := builder.constantValue(vars[1])
	n2, ok2 := builder.constantValue(vars[2])

	if ok1 && ok2 {
		builder.cs.Sub(&n1, &n2)
		res := builder.Mul(cond, n1)    // no constraint is recorded
		res = builder.Add(res, vars[2]) // no constraint is recorded
		return res
	}

	// special case appearing in AssertIsLessOrEq
	if ok1 {
		if n1.IsZero() {
			v := builder.Sub(builder.cstOne(), vars[0])
			return builder.Mul(v, vars[2])
		}
	}

	v := builder.Sub(vars[1], vars[2]) // no constraint is recorded
	w := builder.Mul(cond, v)
	return builder.Add(w, vars[2])

}

// Lookup2 performs a 2-bit lookup between i1, i2, i3, i4 based on bits b0
// and b1. Returns i0 if b0=b1=0, i1 if b0=1 and b1=0, i2 if b0=0 and b1=1
// and i3 if b0=b1=1.
func (builder *builder) Lookup2(b0, b1 frontend.Variable, i0, i1, i2, i3 frontend.Variable) frontend.Variable {
	vars, _ := builder.toVariables(b0, b1, i0, i1, i2, i3)
	s0, s1 := vars[0], vars[1]
	in0, in1, in2, in3 := vars[2], vars[3], vars[4], vars[5]

	// ensure that bits are actually bits. Adds no constraints if the variables
	// are already constrained.
	builder.AssertIsBoolean(s0)
	builder.AssertIsBoolean(s1)

	c0, b0IsConstant := builder.constantValue(s0)
	c1, b1IsConstant := builder.constantValue(s1)

	if b0IsConstant && b1IsConstant {
		b0 := builder.isCstOne(&c0)
		b1 := builder.isCstOne(&c1)

		if !b0 && !b1 {
			return in0
		}
		if b0 && !b1 {
			return in1
		}
		if b0 && b1 {
			return in3
		}
		return in2
	}

	// two-bit lookup for the general case can be done with three constraints as
	// following:
	//    (1) (in3 - in2 - in1 + in0) * s1 = tmp1 - in1 + in0
	//    (2) tmp1 * s0 = tmp2
	//    (3) (in2 - in0) * s1 = RES - tmp2 - in0
	// the variables tmp1 and tmp2 are new internal variables and the variables
	// RES will be the returned result

	tmp1 := builder.Add(in3, in0)
	tmp1 = builder.Sub(tmp1, in2, in1)
	tmp1 = builder.Mul(tmp1, s1)
	tmp1 = builder.Add(tmp1, in1)
	tmp1 = builder.Sub(tmp1, in0) // (1) tmp1 = s1 * (in3 - in2 - in1 + in0) + in1 - in0
	tmp2 := builder.Mul(tmp1, s0) // (2) tmp2 = tmp1 * s0
	res := builder.Sub(in2, in0)
	res = builder.Mul(res, s1)
	res = builder.Add(res, tmp2, in0) // (3) res = (v2 - v0) * s1 + tmp2 + in0
	return res
}

// IsZero returns 1 if i1 is zero, 0 otherwise
func (builder *builder) IsZero(i1 frontend.Variable) frontend.Variable {
	vars, _ := builder.toVariables(i1)
	a := vars[0]
	if c, ok := builder.constantValue(a); ok {
		if c.IsZero() {
			return builder.cstOne()
		}
		return builder.cstZero()
	}

	debug := builder.newDebugInfo("isZero", a)

	// x = 1/a 				// in a hint (x == 0 if a == 0)
	// m = -a*x + 1         // constrain m to be 1 if a == 0
	// a * m = 0            // constrain m to be 0 if a != 0

	m := builder.newInternalVariable()

	// x = 1/a 				// in a hint (x == 0 if a == 0)
	x, err := builder.NewHint(solver.InvZeroHint, 1, a)
	if err != nil {
		// the function errs only if the number of inputs is invalid.
		panic(err)
	}

	// m = -a*x + 1         // constrain m to be 1 if a == 0
	c1 := builder.cs.AddConstraint(builder.newR1C(builder.Neg(a), x[0], builder.Sub(m, 1)))

	// a * m = 0            // constrain m to be 0 if a != 0
	c2 := builder.cs.AddConstraint(builder.newR1C(a, m, builder.cstZero()))

	builder.cs.AttachDebugInfo(debug, []int{c1, c2})

	return m
}

// Cmp returns 1 if i1>i2, 0 if i1=i2, -1 if i1<i2
func (builder *builder) Cmp(i1, i2 frontend.Variable) frontend.Variable {

	vars, _ := builder.toVariables(i1, i2)
	bi1 := builder.ToBinary(vars[0], builder.cs.FieldBitLen())
	bi2 := builder.ToBinary(vars[1], builder.cs.FieldBitLen())

	res := builder.cstZero()

	for i := builder.cs.FieldBitLen() - 1; i >= 0; i-- {

		iszeroi1 := builder.IsZero(bi1[i])
		iszeroi2 := builder.IsZero(bi2[i])

		i1i2 := builder.And(bi1[i], iszeroi2)
		i2i1 := builder.And(bi2[i], iszeroi1)

		n := builder.Select(i2i1, -1, 0)
		m := builder.Select(i1i2, 1, n)

		res = builder.Select(builder.IsZero(res), m, res).(expr.LinearExpression)

	}
	return res
}

// Println enables circuit debugging and behaves almost like fmt.Println()
//
// the print will be done once the R1CS.Solve() method is executed
//
// if one of the input is a variable, its value will be resolved avec R1CS.Solve() method is called
func (builder *builder) Println(a ...frontend.Variable) {
	var log constraint.LogEntry

	// prefix log line with file.go:line
	if _, file, line, ok := runtime.Caller(1); ok {
		log.Caller = fmt.Sprintf("%s:%d", filepath.Base(file), line)
	}

	var sbb strings.Builder

	for i, arg := range a {
		if i > 0 {
			sbb.WriteByte(' ')
		}
		if v, ok := arg.(expr.LinearExpression); ok {
			assertIsSet(v)

			sbb.WriteString("%s")
			// we set limits to the linear expression, so that the log printer
			// can evaluate it before printing it
			log.ToResolve = append(log.ToResolve, builder.getLinearExpression(v))
		} else {
			builder.printArg(&log, &sbb, arg)
		}
	}

	// set format string to be used with fmt.Sprintf, once the variables are solved in the R1CS.Solve() method
	log.Format = sbb.String()

	builder.cs.AddLog(log)
}

func (builder *builder) printArg(log *constraint.LogEntry, sbb *strings.Builder, a frontend.Variable) {

	leafCount, err := schema.Walk(a, tVariable, nil)
	count := leafCount.Public + leafCount.Secret

	// no variables in nested struct, we use fmt std print function
	if count == 0 || err != nil {
		sbb.WriteString(fmt.Sprint(a))
		return
	}

	sbb.WriteByte('{')
	printer := func(f schema.LeafInfo, tValue reflect.Value) error {
		count--
		sbb.WriteString(f.FullName())
		sbb.WriteString(": ")
		sbb.WriteString("%s")
		if count != 0 {
			sbb.WriteString(", ")
		}

		v := tValue.Interface().(expr.LinearExpression)
		// we set limits to the linear expression, so that the log printer
		// can evaluate it before printing it
		log.ToResolve = append(log.ToResolve, builder.getLinearExpression(v))
		return nil
	}
	// ignoring error, printer() doesn't return errors
	_, _ = schema.Walk(a, tVariable, printer)
	sbb.WriteByte('}')
}

// returns -le, the result is a copy
func (builder *builder) negateLinExp(l expr.LinearExpression) expr.LinearExpression {
	res := make(expr.LinearExpression, len(l))
	copy(res, l)
	for i := 0; i < len(res); i++ {
		builder.cs.Neg(&res[i].Coeff)
	}
	return res
}

func (builder *builder) Compiler() frontend.Compiler {
	return builder
}

func (builder *builder) Commit(v ...frontend.Variable) (frontend.Variable, error) {
	// we want to build a sorted slice of committed variables, without duplicates
	// this is the same algorithm as builder.add(...); but we expect len(v) to be quite large.

	vars, s := builder.toVariables(v...)

	// initialize the min-heap
	// this is the same algorithm as api.add --> we want to merge k sorted linear expression
	for lID, v := range vars {
		builder.heap = append(builder.heap, linMeta{val: v[0].VID, lID: lID})
	}
	builder.heap.heapify()

	// sort all the wires
	committed := make([]int, 0, s)

	curr := -1
	nbPublicCommitted := 0

	// process all the terms from all the inputs, in sorted order
	for len(builder.heap) > 0 {
		lID, tID := builder.heap[0].lID, builder.heap[0].tID
		if tID == len(vars[lID])-1 {
			// last element, we remove it from the heap.
			builder.heap.popHead()
		} else {
			// increment and fix the heap
			builder.heap[0].tID++
			builder.heap[0].val = vars[lID][tID+1].VID
			builder.heap.fix(0)
		}
		t := &vars[lID][tID]
		if t.VID == 0 {
			continue // don't commit to ONE_WIRE
		}
		if curr != -1 && t.VID == committed[curr] {
			// it's the same variable ID, do nothing
			continue
		} else {
			// append, it's a new variable ID
			committed = append(committed, t.VID)
			if t.VID < builder.cs.GetNbPublicVariables() {
				nbPublicCommitted++
			}
			curr++
		}
	}

	if len(committed) == 0 {
		return nil, errors.New("must commit to at least one variable")
	}

	// build commitment
	commitment := constraint.NewCommitment(committed, nbPublicCommitted)

	// hint is used at solving time to compute the actual value of the commitment
	// it is going to be dynamically replaced at solving time.
	hintOut, err := builder.NewHint(cs.Bsb22CommitmentComputePlaceholder, 1, builder.getCommittedVariables(&commitment)...)
	if err != nil {
		return nil, err
	}
	cVar := hintOut[0]
	commitment.HintID = solver.GetHintID(cs.Bsb22CommitmentComputePlaceholder) // TODO @gbotrel probably not needed

	commitment.CommitmentIndex = (cVar.(expr.LinearExpression))[0].WireID()

	// TODO @Tabaie Get rid of this field
	commitment.CommittedAndCommitment = append(commitment.Committed, commitment.CommitmentIndex)
	if commitment.CommitmentIndex <= commitment.Committed[len(commitment.Committed)-1] {
		return nil, fmt.Errorf("commitment variable index smaller than some committed variable indices")
	}

	if err := builder.cs.AddCommitment(commitment); err != nil {
		return nil, err
	}

	return cVar, nil
}

func (builder *builder) getCommittedVariables(i *constraint.Commitment) []frontend.Variable {
	res := make([]frontend.Variable, len(i.Committed))
	for j, wireIndex := range i.Committed {
		res[j] = expr.NewLinearExpression(wireIndex, builder.tOne)
	}
	return res
}
