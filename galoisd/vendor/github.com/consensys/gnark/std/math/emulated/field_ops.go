package emulated

import (
	"errors"
	"fmt"
	"math"
	"math/big"

	"github.com/consensys/gnark/frontend"
)

// Div computes a/b and returns it. It uses [DivHint] as a hint function.
func (f *Field[T]) Div(a, b *Element[T]) *Element[T] {
	// omit width assertion as for a is done in AssertIsEqual and for b is done in Mul below
	if !f.fParams.IsPrime() {
		// TODO shouldn't we still try to do a classic int div in a hint, constraint the result, and let it fail?
		// that would enable things like uint32 div ?
		panic("modulus not a prime")
	}
	div, err := f.computeDivisionHint(a.Limbs, b.Limbs)
	if err != nil {
		panic(fmt.Sprintf("compute division: %v", err))
	}
	e := f.packLimbs(div, true)
	res := f.Mul(e, b)
	f.AssertIsEqual(res, a)
	return e
}

// Inverse compute 1/a and returns it. It uses [InverseHint].
func (f *Field[T]) Inverse(a *Element[T]) *Element[T] {
	// omit width assertion as is done in Mul below
	if !f.fParams.IsPrime() {
		panic("modulus not a prime")
	}
	k, err := f.computeInverseHint(a.Limbs)
	if err != nil {
		panic(fmt.Sprintf("compute inverse: %v", err))
	}
	e := f.packLimbs(k, true)
	res := f.Mul(e, a)
	one := f.One()
	f.AssertIsEqual(res, one)
	return e
}

// Sqrt computes square root of a and returns it. It uses [SqrtHint].
func (f *Field[T]) Sqrt(a *Element[T]) *Element[T] {
	// omit width assertion as is done in Mul below
	if !f.fParams.IsPrime() {
		panic("modulus not a prime")
	}
	res, err := f.NewHint(SqrtHint, 1, a)
	if err != nil {
		panic(fmt.Sprintf("compute sqrt: %v", err))
	}
	_a := f.Mul(res[0], res[0])
	f.AssertIsEqual(_a, a)
	return res[0]
}

// Add computes a+b and returns it. If the result wouldn't fit into Element, then
// first reduces the inputs (larger first) and tries again. Doesn't mutate
// inputs.
func (f *Field[T]) Add(a, b *Element[T]) *Element[T] {
	return f.reduceAndOp(f.add, f.addPreCond, a, b)
}

func (f *Field[T]) addPreCond(a, b *Element[T]) (nextOverflow uint, err error) {
	reduceRight := a.overflow < b.overflow
	nextOverflow = max(a.overflow, b.overflow) + 1
	if nextOverflow > f.maxOverflow() {
		err = overflowError{op: "add", nextOverflow: nextOverflow, maxOverflow: f.maxOverflow(), reduceRight: reduceRight}
	}
	return
}

func (f *Field[T]) add(a, b *Element[T], nextOverflow uint) *Element[T] {
	ba, aConst := f.constantValue(a)
	bb, bConst := f.constantValue(b)
	if aConst && bConst {
		ba.Add(ba, bb).Mod(ba, f.fParams.Modulus())
		return newConstElement[T](ba)
	}

	nbLimbs := max(len(a.Limbs), len(b.Limbs))
	limbs := make([]frontend.Variable, nbLimbs)
	for i := range limbs {
		limbs[i] = 0
		if i < len(a.Limbs) {
			limbs[i] = f.api.Add(limbs[i], a.Limbs[i])
		}
		if i < len(b.Limbs) {
			limbs[i] = f.api.Add(limbs[i], b.Limbs[i])
		}
	}
	return f.newInternalElement(limbs, nextOverflow)
}

// Mul computes a*b and returns it. It doesn't reduce the output and it may be
// larger than the modulus. The returned Element has as many limbs as the inputs
// together. If the result wouldn't fit into Element, then locally reduces the
// inputs first. Doesn't mutate inputs.
//
// Even though this method skips reduction and allows for multiplication chains,
// then in most cases it is more efficient to use [Field[T].MulMod] as reducing
// Element with 2 times the limbs is 2 times more expensive.
//
// For multiplying by a constant, use [Field[T].MulConst] method which is more
// efficient.
//
// Uses [MultiplicationHint].
func (f *Field[T]) Mul(a, b *Element[T]) *Element[T] {
	return f.reduceAndOp(f.mul, f.mulPreCond, a, b)
}

// Mul computes a*b and reduces it modulo the field order. The returned Element
// has default number of limbs and zero overflow.
func (f *Field[T]) MulMod(a, b *Element[T]) *Element[T] {
	r := f.Mul(a, b)
	return f.Reduce(r)
}

// MulConst multiplies a by a constant c and returns it. We assume that the
// input constant is "small", so that we can compute the product by multiplying
// all individual limbs with the constant. If it is not small, then use the
// general [Field[T].Mul] or [Field[T].MulMod] with creating new Element from
// the constant on-the-fly.
func (f *Field[T]) MulConst(a *Element[T], c *big.Int) *Element[T] {
	switch c.Sign() {
	case -1:
		f.MulConst(f.Neg(a), new(big.Int).Neg(c))
	case 0:
		return f.Zero()
	}
	cbl := uint(c.BitLen())
	if cbl > f.maxOverflow() {
		panic(fmt.Sprintf("constant bit length %d exceeds max %d", cbl, f.maxOverflow()))
	}
	return f.reduceAndOp(
		func(a, _ *Element[T], u uint) *Element[T] {
			if ba, aConst := f.constantValue(a); aConst {
				ba.Mul(ba, c)
				return newConstElement[T](ba)
			}
			limbs := make([]frontend.Variable, len(a.Limbs))
			for i := range a.Limbs {
				limbs[i] = f.api.Mul(a.Limbs[i], c)
			}
			return f.newInternalElement(limbs, a.overflow+cbl)
		},
		func(a, _ *Element[T]) (nextOverflow uint, err error) {
			nextOverflow = a.overflow + uint(cbl)
			if nextOverflow > f.maxOverflow() {
				err = overflowError{op: "mulConst", nextOverflow: nextOverflow, maxOverflow: f.maxOverflow()}
			}
			return
		},
		a, nil,
	)
}

func (f *Field[T]) mulPreCond(a, b *Element[T]) (nextOverflow uint, err error) {
	reduceRight := a.overflow < b.overflow
	nbResLimbs := nbMultiplicationResLimbs(len(a.Limbs), len(b.Limbs))
	nextOverflow = f.fParams.BitsPerLimb() + uint(math.Log2(float64(2*nbResLimbs-1))) + 1 + a.overflow + b.overflow
	if nextOverflow > f.maxOverflow() {
		err = overflowError{op: "mul", nextOverflow: nextOverflow, maxOverflow: f.maxOverflow(), reduceRight: reduceRight}
	}
	return
}

func (f *Field[T]) mul(a, b *Element[T], nextOverflow uint) *Element[T] {
	ba, aConst := f.constantValue(a)
	bb, bConst := f.constantValue(b)
	if aConst && bConst {
		ba.Mul(ba, bb).Mod(ba, f.fParams.Modulus())
		return newConstElement[T](ba)
	}

	// mulResult contains the result (out of circuit) of a * b school book multiplication
	// len(mulResult) == len(a) + len(b) - 1
	mulResult, err := f.computeMultiplicationHint(a.Limbs, b.Limbs)
	if err != nil {
		panic(fmt.Sprintf("multiplication hint: %s", err))
	}

	// we computed the result of the mul outside the circuit (mulResult)
	// and we want to constrain inside the circuit that this injected value
	// actually matches the in-circuit a * b values
	// create constraints (\sum_{i=0}^{m-1} a_i c^i) * (\sum_{i=0}^{m-1} b_i
	// c^i) = (\sum_{i=0}^{2m-2} z_i c^i) for c \in {1, 2m-1}
	w := new(big.Int)
	for c := 1; c <= len(mulResult); c++ {
		w.SetInt64(1) // c^i
		l := f.api.Mul(a.Limbs[0], 1)
		r := f.api.Mul(b.Limbs[0], 1)
		o := f.api.Mul(mulResult[0], 1)

		for i := 1; i < len(mulResult); i++ {
			w.Lsh(w, uint(c))
			if i < len(a.Limbs) {
				l = f.api.MulAcc(l, a.Limbs[i], w)
			}
			if i < len(b.Limbs) {
				r = f.api.MulAcc(r, b.Limbs[i], w)
			}
			o = f.api.MulAcc(o, mulResult[i], w)
		}
		f.api.AssertIsEqual(f.api.Mul(l, r), o)
	}
	return f.newInternalElement(mulResult, nextOverflow)
}

// Reduce reduces a modulo the field order and returns it. Uses hint [RemHint].
func (f *Field[T]) Reduce(a *Element[T]) *Element[T] {
	f.enforceWidthConditional(a)
	if a.overflow == 0 {
		// fast path - already reduced, omit reduction.
		return a
	}
	// sanity check
	if _, aConst := f.constantValue(a); aConst {
		panic("trying to reduce a constant, which happen to have an overflow flag set")
	}

	// slow path - use hint to reduce value
	e, err := f.computeRemHint(a, f.Modulus())
	if err != nil {
		panic(fmt.Sprintf("reduction hint: %v", err))
	}
	f.AssertIsEqual(e, a)
	return e
}

// Sub subtracts b from a and returns it. Reduces locally if wouldn't fit into
// Element. Doesn't mutate inputs.
func (f *Field[T]) Sub(a, b *Element[T]) *Element[T] {
	return f.reduceAndOp(f.sub, f.subPreCond, a, b)
}

// subReduce returns a-b and returns it. Contrary to [Field[T].Sub] method this
// method does not reduce the inputs if the result would overflow. This method
// is currently only used as a subroutine in [Field[T].Reduce] method to avoid
// infinite recursion when we are working exactly on the overflow limits.
func (f *Field[T]) subNoReduce(a, b *Element[T]) *Element[T] {
	nextOverflow, _ := f.subPreCond(a, b)
	// we ignore error as it only indicates if we should reduce or not. But we
	// are in non-reducing version of sub.
	return f.sub(a, b, nextOverflow)
}

func (f *Field[T]) subPreCond(a, b *Element[T]) (nextOverflow uint, err error) {
	reduceRight := a.overflow < (b.overflow + 1)
	nextOverflow = max(b.overflow+1, a.overflow) + 1
	if nextOverflow > f.maxOverflow() {
		err = overflowError{op: "sub", nextOverflow: nextOverflow, maxOverflow: f.maxOverflow(), reduceRight: reduceRight}
	}
	return
}

func (f *Field[T]) sub(a, b *Element[T], nextOverflow uint) *Element[T] {
	ba, aConst := f.constantValue(a)
	bb, bConst := f.constantValue(b)
	if aConst && bConst {
		ba.Sub(ba, bb).Mod(ba, f.fParams.Modulus())
		return newConstElement[T](ba)
	}

	// first we have to compute padding to ensure that the subtraction does not
	// underflow.
	nbLimbs := max(len(a.Limbs), len(b.Limbs))
	limbs := make([]frontend.Variable, nbLimbs)
	padLimbs := subPadding[T](b.overflow, uint(nbLimbs))
	for i := range limbs {
		limbs[i] = padLimbs[i]
		if i < len(a.Limbs) {
			limbs[i] = f.api.Add(limbs[i], a.Limbs[i])
		}
		if i < len(b.Limbs) {
			limbs[i] = f.api.Sub(limbs[i], b.Limbs[i])
		}
	}
	return f.newInternalElement(limbs, nextOverflow)
}

func (f *Field[T]) Neg(a *Element[T]) *Element[T] {
	return f.Sub(f.Zero(), a)
}

// Select sets e to a if selector == 1 and to b otherwise. Sets the number of
// limbs and overflow of the result to be the maximum of the limb lengths and
// overflows. If the inputs are strongly unbalanced, then it would better to
// reduce the result after the operation.
func (f *Field[T]) Select(selector frontend.Variable, a, b *Element[T]) *Element[T] {
	f.enforceWidthConditional(a)
	f.enforceWidthConditional(b)
	overflow := max(a.overflow, b.overflow)
	nbLimbs := max(len(a.Limbs), len(b.Limbs))
	e := f.newInternalElement(make([]frontend.Variable, nbLimbs), overflow)
	normalize := func(limbs []frontend.Variable) []frontend.Variable {
		if len(limbs) < nbLimbs {
			tail := make([]frontend.Variable, nbLimbs-len(limbs))
			for i := range tail {
				tail[i] = 0
			}
			return append(limbs, tail...)
		}
		return limbs
	}
	aNormLimbs := normalize(a.Limbs)
	bNormLimbs := normalize(b.Limbs)
	for i := range e.Limbs {
		e.Limbs[i] = f.api.Select(selector, aNormLimbs[i], bNormLimbs[i])
	}
	return e
}

// Lookup2 performs two-bit lookup between a, b, c, d based on lookup bits b1
// and b2 such that:
//   - if b0=0 and b1=0, sets to a,
//   - if b0=1 and b1=0, sets to b,
//   - if b0=0 and b1=1, sets to c,
//   - if b0=1 and b1=1, sets to d.
//
// The number of the limbs and overflow in the result is the maximum of the
// inputs'. If the inputs are very unbalanced, then reduce the result.
func (f *Field[T]) Lookup2(b0, b1 frontend.Variable, a, b, c, d *Element[T]) *Element[T] {
	f.enforceWidthConditional(a)
	f.enforceWidthConditional(b)
	f.enforceWidthConditional(c)
	f.enforceWidthConditional(d)
	overflow := max(a.overflow, b.overflow, c.overflow, d.overflow)
	nbLimbs := max(len(a.Limbs), len(b.Limbs), len(c.Limbs), len(d.Limbs))
	e := f.newInternalElement(make([]frontend.Variable, nbLimbs), overflow)
	normalize := func(limbs []frontend.Variable) []frontend.Variable {
		if len(limbs) < nbLimbs {
			tail := make([]frontend.Variable, nbLimbs-len(limbs))
			for i := range tail {
				tail[i] = 0
			}
			return append(limbs, tail...)
		}
		return limbs
	}
	aNormLimbs := normalize(a.Limbs)
	bNormLimbs := normalize(b.Limbs)
	cNormLimbs := normalize(c.Limbs)
	dNormLimbs := normalize(d.Limbs)
	for i := range a.Limbs {
		e.Limbs[i] = f.api.Lookup2(b0, b1, aNormLimbs[i], bNormLimbs[i], cNormLimbs[i], dNormLimbs[i])
	}
	return e
}

// reduceAndOp applies op on the inputs. If the pre-condition check preCond
// errs, then first reduces the input arguments. The reduction is done
// one-by-one with the element with highest overflow reduced first.
func (f *Field[T]) reduceAndOp(op func(*Element[T], *Element[T], uint) *Element[T], preCond func(*Element[T], *Element[T]) (uint, error), a, b *Element[T]) *Element[T] {
	f.enforceWidthConditional(a)
	f.enforceWidthConditional(b)
	var nextOverflow uint
	var err error
	var target overflowError

	for nextOverflow, err = preCond(a, b); errors.As(err, &target); nextOverflow, err = preCond(a, b) {
		if !target.reduceRight {
			a = f.Reduce(a)
		} else {
			b = f.Reduce(b)
		}
	}
	return op(a, b, nextOverflow)
}

type overflowError struct {
	op           string
	nextOverflow uint
	maxOverflow  uint
	reduceRight  bool
}

func (e overflowError) Error() string {
	return fmt.Sprintf("op %s overflow %d exceeds max %d", e.op, e.nextOverflow, e.maxOverflow)
}
