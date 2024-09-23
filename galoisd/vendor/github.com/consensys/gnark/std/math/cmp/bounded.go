package cmp

import (
	"fmt"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/math/bits"
	"math/big"
)

func init() {
	// register hints
	solver.RegisterHint(GetHints()...)
}

// GetHints returns all hint functions used in this package. This method is
// useful for registering all hints in the solver.
func GetHints() []solver.Hint {
	return []solver.Hint{isLessOutputHint, minOutputHint}
}

// BoundedComparator provides comparison methods, with relatively low circuit
// complexity, for signed comparison of two integers a and b, when an upper
// bound for their absolute difference (|a - b|) is known. These methods perform
// only one binary conversion of length: absDiffUppBitLen.
//
// a and b can be any signed integers, as long as their absolute difference
// respects the specified bound: |a - b| <= absDiffUpp. See
// NewBoundedComparator, for more information.
type BoundedComparator struct {
	// absDiffUppBitLen is the assumed maximum length for the binary representation
	// of |a - b|. Every method preforms exactly one binary decomposition of this
	// length.
	absDiffUppBitLen int
	api              frontend.API

	// we will use value receiver for methods of this struct,
	// since: 1) the struct is small. 2) methods should not modify any fields.
}

// NewBoundedComparator creates a new BoundedComparator, which provides methods
// for comparing two numbers a and b.
//
// absDiffUpp is the upper bound of the absolute difference of a and b, such
// that |a - b| <= absDiffUpp. Notice that |a - b| can be equal to absDiffUpp.
// absDiffUpp must be a positive number, and P - absDiffUpp - 1 must have a
// longer binary representation than absDiffUpp, where P is the order of the
// underlying field. Lower values of absDiffUpp will reduce the number of
// generated constraints.
//
// This function can detect invalid values of absDiffUpp and panics when the
// provided value is not positive or is too big.
//
// As long as |a - b| <= absDiffUpp, all the methods of BoundedComparator work
// correctly.
//
// If |a - b| > absDiffUpp, as long as |a - b| < P - 2^absDiffUpp.BitLen(),
// either a proof can not be generated or the comparison methods work correctly.
//
// When |a - b| >= P - 2^absDiffUpp.BitLen(), if allowNonDeterministicBehaviour
// is not set, either a proof can not be generated or the methods wrongly
// produce reversed results. The exact behaviour depends on the specific method
// and the value of |a - b|, but it will be always well-defined and
// deterministic.
//
// If allowNonDeterministicBehaviour is set, when
// |a - b| >= P - 2^absDiffUpp.BitLen(), the generated constraint system
// sometimes may have multiple solutions and hence the behaviour of the exported
// methods of BoundedComparator will be undefined.
func NewBoundedComparator(api frontend.API, absDiffUpp *big.Int, allowNonDeterministicBehaviour bool) *BoundedComparator {
	// Our comparison methods work by using the fact that when a != b,
	// between certain two numbers at the same time only one can be
	// non-negative (i.e. positive or zero):
	//
	// AssertIsLessEq -> (a - b, b - a)
	// AssertIsLess   -> (a - b - 1, b - a - 1)
	// IsLess         -> (a - b, b - a - 1)
	// IsLessEq       -> (a - b - 1, b - a)
	// Min            -> (a - b, b - a)
	//
	// We assume that the underlying field is of prime order P, so the negative
	// of x is P - x. We need to be able to determine the non-negative number in
	// each case, and we are doing that by relying on the fact that the negative
	// number has a longer binary decomposition than a certain threshold:
	// absDiffUppBitLen. So, we'll need to find a suitable absDiffUppBitLen, and
	// make sure that any possible negative number has a longer binary
	// representation than absDiffUppBitLen.
	//
	// We see that, between different methods, the biggest possible positive number
	// is |a - b| and the smallest possible negative number is -(|a - b| + 1).
	//
	// On the other hand, we have |a - b| <= absDiffUpp which means:
	// -(|a - b| + 1) >= -(absDiffUpp + 1). Therefore, if we let
	// absDiffUppBitLen = absDiffUpp.BitLen(),
	// that would be the minimum possible value for absDiffUppBitLen.
	// Then, we will need to make sure that P - absDiffUpp - 1 has a binary
	// representation longer than absDiffUppBitLen.
	//
	// If we increase |a - b|, as soon as P - |a - b| - 1 becomes smaller than
	// 2^absDiffUpp.BitLen(), the negative number will have a binary representation
	// that is shorter than the threshold and proofs for wrong results can be
	// generated. In this case, if the positive number has a longer than threshold
	// binary representation the behaviour of the comparison methods will stay
	// well-defined, and the constraint system will have a wrong but unique
	// solution. The positive number is always bigger than |a - b| - 1. So, we need
	// to make sure that if
	// P - |a - b| <= 2^absDiffUpp.BitLen(), then |a - b| - 1 >= 2^absDiffUpp.BitLen().
	// Obviously this condition holds when P - 1 >= 2^(absDiffUpp.BitLen()+1) or
	// P > 2^(absDiffUpp.BitLen()+1).
	P := api.Compiler().Field()
	if absDiffUpp.Cmp(big.NewInt(0)) != 1 || absDiffUpp.Cmp(P) != -1 {
		panic("absDiffUpp must be a positive number smaller than the field order")
	}
	// we checked absDiffUpp < P, so we'll not have an overflow here.
	smallestNeg := new(big.Int).Sub(P, absDiffUpp)
	smallestNeg.Sub(smallestNeg, big.NewInt(1))
	if smallestNeg.BitLen() <= absDiffUpp.BitLen() {
		panic("cannot construct the comparator, the specified absDiffUpp is too high")
	}

	if !allowNonDeterministicBehaviour {
		// if not P > 2^(absDiffUpp.BitLen()+1)
		if P.Cmp(new(big.Int).Lsh(big.NewInt(1), uint(absDiffUpp.BitLen()+1))) != 1 {
			panic("absDiffUpp has to be smaller for ensuring deterministic behaviour")
		}
	}

	return &BoundedComparator{
		absDiffUppBitLen: absDiffUpp.BitLen(),
		api:              api,
	}
}

// assertIsNonNegative defines constraints that ensure x >= 0.
func (bc BoundedComparator) assertIsNonNegative(x frontend.Variable) {
	bits.ToBinary(bc.api, x, bits.WithNbDigits(bc.absDiffUppBitLen))
}

// AssertIsLessEq defines a set of constraints that can be satisfied only
// if a <= b.
func (bc BoundedComparator) AssertIsLessEq(a, b frontend.Variable) {
	// a <= b <==> b - a >= 0
	bc.assertIsNonNegative(bc.api.Sub(b, a))
}

// AssertIsLess defines a set of constraints that can be satisfied only
// if a < b.
func (bc BoundedComparator) AssertIsLess(a, b frontend.Variable) {
	// a < b <==> a <= b - 1
	bc.AssertIsLessEq(a, bc.api.Sub(b, 1))
}

// IsLess returns 1 if a < b, and returns 0 if a >= b.
func (bc BoundedComparator) IsLess(a, b frontend.Variable) frontend.Variable {
	res, err := bc.api.Compiler().NewHint(isLessOutputHint, 1, a, b)
	if err != nil {
		panic(fmt.Sprintf("error in calling isLessOutputHint: %v", err))
	}
	indicator := res[0]
	// a < b  <==> b - a - 1 >= 0
	// a >= b <==> a - b >= 0
	bc.assertIsNonNegative(bc.api.Select(indicator, bc.api.Sub(b, a, 1), bc.api.Sub(a, b)))
	return indicator
}

// IsLessEq returns 1 if a <= b, and returns 0 if a > b.
func (bc BoundedComparator) IsLessEq(a, b frontend.Variable) frontend.Variable {
	// a <= b <==> a < b + 1
	return bc.IsLess(a, bc.api.Add(b, 1))
}

// Min returns the minimum of a and b.
func (bc BoundedComparator) Min(a, b frontend.Variable) frontend.Variable {
	res, err := bc.api.Compiler().NewHint(minOutputHint, 1, a, b)
	if err != nil {
		panic(fmt.Sprintf("error in calling minOutputHint: %v", err))
	}
	min := res[0]

	aDiff := bc.api.Sub(a, min)
	bDiff := bc.api.Sub(b, min)

	// (a - min) * (b - min) == 0
	bc.api.AssertIsEqual(0, bc.api.Mul(aDiff, bDiff))

	// (a - min) + (b - min) >= 0
	bc.assertIsNonNegative(bc.api.Add(aDiff, bDiff))

	return min
}

// cmpInField compares a and b in a finite field of the specified order.
func cmpInField(a, b, order *big.Int) int {
	biggestPositiveNum := new(big.Int).Rsh(order, 1)
	if a.Cmp(biggestPositiveNum)*b.Cmp(biggestPositiveNum) == -1 {
		return -a.Cmp(b)
	}
	return a.Cmp(b)
}

// minOutputHint produces the output of [BoundedComparator.Min] as a hint.
func minOutputHint(fieldOrder *big.Int, inputs, results []*big.Int) error {
	a := inputs[0]
	b := inputs[1]

	if cmpInField(a, b, fieldOrder) == -1 {
		// a < b
		results[0].Set(a)
	} else {
		// a >= b
		results[0].Set(b)
	}
	return nil
}

// isLessOutputHint produces the output of [BoundedComparator.IsLess] as a hint.
func isLessOutputHint(fieldOrder *big.Int, inputs, results []*big.Int) error {
	a := inputs[0]
	b := inputs[1]

	if cmpInField(a, b, fieldOrder) == -1 {
		// a < b
		results[0].SetUint64(1)
	} else {
		// a >= b
		results[0].SetUint64(0)
	}
	return nil
}
