// Package cmp provides methods and functions for comparing two numbers.
package cmp

import (
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/math/bits"
	"math/big"
)

// IsLess returns 1 if a < b, and returns 0 if a >= b. a and b should be
// integers in range [0, P-1], where P is the order of the underlying field used
// by the proof system.
//
// When inputs are not in range [0, P-1], the remainder of their division by P
// will be considered for comparison.
func IsLess(api frontend.API, a, b frontend.Variable) frontend.Variable {
	return isLessRecursive(api, bits.ToBinary(api, a), bits.ToBinary(api, b), false, true)
}

// IsLessOrEqual returns 1 if a <= b, and returns 0 if a > b. a and b should be
// integers in range [0, P-1], where P is the order of the underlying field used
// by the proof system.
//
// When inputs are not in range [0, P-1], the remainder of their division by P
// will be considered for comparison.
func IsLessOrEqual(api frontend.API, a, b frontend.Variable) frontend.Variable {
	return isLessRecursive(api, bits.ToBinary(api, a), bits.ToBinary(api, b), true, true)
}

// IsLessBinary compares two non-negative binary numbers represented by aBits
// and bBits. It returns 1 if the integer represented by aBits is less than the
// integer represented by bBits, and returns 0 otherwise.
func IsLessBinary(api frontend.API, aBits, bBits []frontend.Variable) frontend.Variable {
	if len(aBits) != len(bBits) {
		panic("a and b must have the same length")
	}
	assertBits(api, aBits)
	assertBits(api, bBits)
	return isLessRecursive(api, aBits, bBits, false, true)
}

// IsLessOrEqualBinary compares two non-negative binary numbers represented by
// aBits and bBits. It returns 1 if the integer represented by aBits is less
// than or equal to the integer represented by bBits, and returns 0 otherwise.
func IsLessOrEqualBinary(api frontend.API, aBits, bBits []frontend.Variable) frontend.Variable {
	if len(aBits) != len(bBits) {
		panic("a and b must have the same length")
	}
	assertBits(api, aBits)
	assertBits(api, bBits)
	return isLessRecursive(api, aBits, bBits, true, true)
}

// isLessRecursive compares binary numbers a and b. When useBoundedCmp is false
// it performs normal bit by bit comparison which defines 2*n multiplication
// constraints. When useBoundedCmp is true, bit by bit comparison will be used
// for a few high order bits and the rest of bits will be compared by
// BoundedComparator. If addition is free, this will define n + 5 constraints
// when n == api.Compiler().FieldBitLen().
//
// acceptEquality determines the return value of the function when a == b.
func isLessRecursive(api frontend.API,
	a, b []frontend.Variable,
	acceptEquality bool, useBoundedCmp bool) frontend.Variable {
	n := len(a)
	if n == 0 {
		if acceptEquality {
			return 1
		} else {
			return 0
		}
	}
	// Interestingly when one of the two numbers is a constant, using bit by bit
	// comparison will produce 3 fewer constraints than using the BoundedComparator.
	if useBoundedCmp && n <= api.Compiler().FieldBitLen()-2 &&
		isNotConstant(api, a[n-1]) && isNotConstant(api, b[n-1]) {
		diffBound := new(big.Int).Lsh(big.NewInt(1), uint(n))
		diffBound.Sub(diffBound, big.NewInt(1))
		comparator := NewBoundedComparator(api, diffBound, false)
		a := bits.FromBinary(api, a, bits.WithUnconstrainedInputs())
		b := bits.FromBinary(api, b, bits.WithUnconstrainedInputs())
		if acceptEquality {
			return comparator.IsLessEq(a, b)
		} else {
			return comparator.IsLess(a, b)
		}
	}

	// out = (a[n-1] + b[n-1] - 2*a[n-1]*b[n-1])*(b[n-1] - cmp) + cmp
	eq := api.Add(a[n-1], b[n-1], api.Mul(-2, a[n-1], b[n-1]))
	cmp := isLessRecursive(api, a[:n-1], b[:n-1], acceptEquality, useBoundedCmp)
	return api.Add(cmp, api.Mul(eq, api.Sub(b[n-1], cmp)))
}

func isNotConstant(api frontend.API, x frontend.Variable) bool {
	_, isConstant := api.Compiler().ConstantValue(x)
	return !isConstant
}

// assertBits defines boolean constraints for every element of bits.
func assertBits(api frontend.API, bits []frontend.Variable) {
	for _, b := range bits {
		api.AssertIsBoolean(b)
	}
}
