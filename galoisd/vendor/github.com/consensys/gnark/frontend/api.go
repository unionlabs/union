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

package frontend

import (
	"math/big"

	"github.com/consensys/gnark/constraint/solver"
)

// API represents the available functions to circuit developers
type API interface {
	// ---------------------------------------------------------------------------------------------
	// Arithmetic

	// Add returns res = i1+i2+...in
	Add(i1, i2 Variable, in ...Variable) Variable

	// MulAcc sets and return a = a + (b*c).
	//
	// ! The method may mutate a without allocating a new result. If the input
	// is used elsewhere, then first initialize new variable, for example by
	// doing:
	//
	//     acopy := api.Mul(a, 1)
	//     acopy = MulAcc(acopy, b, c)
	//
	// ! But it may not modify a, always use MulAcc(...) result for correctness.
	MulAcc(a, b, c Variable) Variable

	// Neg returns -i
	Neg(i1 Variable) Variable

	// Sub returns res = i1 - i2 - ...in
	Sub(i1, i2 Variable, in ...Variable) Variable

	// Mul returns res = i1 * i2 * ... in
	Mul(i1, i2 Variable, in ...Variable) Variable

	// DivUnchecked returns i1 / i2 . if i1 == i2 == 0, returns 0
	DivUnchecked(i1, i2 Variable) Variable

	// Div returns i1 / i2
	Div(i1, i2 Variable) Variable

	// Inverse returns res = 1 / i1
	Inverse(i1 Variable) Variable

	// ---------------------------------------------------------------------------------------------
	// Bit operations
	// TODO @gbotrel move bit operations in std/math/bits

	// ToBinary unpacks a Variable in binary,
	// n is the number of bits to select (starting from lsb)
	// n default value is fr.Bits the number of bits needed to represent a field element
	//
	// The result in little endian (first bit= lsb)
	ToBinary(i1 Variable, n ...int) []Variable

	// FromBinary packs b, seen as a fr.Element in little endian
	FromBinary(b ...Variable) Variable

	// Xor returns a ^ b
	// a and b must be 0 or 1
	Xor(a, b Variable) Variable

	// Or returns a | b
	// a and b must be 0 or 1
	Or(a, b Variable) Variable

	// Or returns a & b
	// a and b must be 0 or 1
	And(a, b Variable) Variable

	// ---------------------------------------------------------------------------------------------
	// Conditionals

	// Select if b is true, yields i1 else yields i2
	Select(b Variable, i1, i2 Variable) Variable

	// Lookup2 performs a 2-bit lookup between i1, i2, i3, i4 based on bits b0
	// and b1. Returns i0 if b0=b1=0, i1 if b0=1 and b1=0, i2 if b0=0 and b1=1
	// and i3 if b0=b1=1.
	Lookup2(b0, b1 Variable, i0, i1, i2, i3 Variable) Variable

	// IsZero returns 1 if a is zero, 0 otherwise
	IsZero(i1 Variable) Variable

	// Cmp returns:
	//  * 1 if i1>i2,
	//  * 0 if i1=i2,
	//  * -1 if i1<i2.
	//
	// If the absolute difference between the variables i1 and i2 is known, then
	// it is more efficient to use the bounded methdods in package
	// [github.com/consensys/gnark/std/math/bits].
	Cmp(i1, i2 Variable) Variable

	// ---------------------------------------------------------------------------------------------
	// Assertions

	// AssertIsEqual fails if i1 != i2
	AssertIsEqual(i1, i2 Variable)

	// AssertIsDifferent fails if i1 == i2
	AssertIsDifferent(i1, i2 Variable)

	// AssertIsBoolean fails if v != 0 and v != 1
	AssertIsBoolean(i1 Variable)
	// AssertIsCrumb fails if v ∉ {0,1,2,3} (crumb is a 2-bit variable; see https://en.wikipedia.org/wiki/Units_of_information)
	AssertIsCrumb(i1 Variable)

	// AssertIsLessOrEqual fails if v > bound.
	//
	// If the absolute difference between the variables b and bound is known, then
	// it is more efficient to use the bounded methdods in package
	// [github.com/consensys/gnark/std/math/bits].
	AssertIsLessOrEqual(v Variable, bound Variable)

	// Println behaves like fmt.Println but accepts cd.Variable as parameter
	// whose value will be resolved at runtime when computed by the solver
	Println(a ...Variable)

	// Compiler returns the compiler object for advanced circuit development
	Compiler() Compiler

	// Deprecated APIs

	// NewHint is a shortcut to api.Compiler().NewHint()
	// Deprecated: use api.Compiler().NewHint() instead
	NewHint(f solver.Hint, nbOutputs int, inputs ...Variable) ([]Variable, error)

	// ConstantValue is a shortcut to api.Compiler().ConstantValue()
	// Deprecated: use api.Compiler().ConstantValue() instead
	ConstantValue(v Variable) (*big.Int, bool)
}

// BatchInvert returns a slice of variables containing the inverse of each element in i1
// This is a temporary API, do not use it in your circuit
type BatchInverter interface {
	// BatchInvert returns a slice of variables containing the inverse of each element in i1
	// This is a temporary API, do not use it in your circuit
	BatchInvert(i1 []Variable) []Variable
}

type PlonkAPI interface {
	// EvaluatePlonkExpression returns res = qL.a + qR.b + qM.ab + qC
	EvaluatePlonkExpression(a, b Variable, qL, qR, qM, qC int) Variable

	// AddPlonkConstraint asserts qL.a + qR.b + qM.ab + qO.o + qC
	AddPlonkConstraint(a, b, o Variable, qL, qR, qO, qM, qC int)
}
