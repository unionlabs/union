package emulated

import (
	"errors"
	"fmt"
	"math/bits"

	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/selector"
)

// Div computes a/b and returns it. It uses [DivHint] as a hint function.
func (f *Field[T]) Div(a, b *Element[T]) *Element[T] {
	return f.reduceAndOp(f.div, f.divPreCond, a, b)
}

func (f *Field[T]) divPreCond(a, b *Element[T]) (nextOverflow uint, err error) {
	mulOf, err := f.mulPreCond(&Element[T]{Limbs: make([]frontend.Variable, f.fParams.NbLimbs()), overflow: 0}, b)
	if err != nil {
		return mulOf, err
	}
	return f.subPreCond(a, &Element[T]{overflow: mulOf})
}

func (f *Field[T]) div(a, b *Element[T], _ uint) *Element[T] {
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
	return f.reduceAndOp(f.inverse, f.inversePreCond, a, nil)
}

func (f *Field[T]) inversePreCond(a, _ *Element[T]) (nextOverflow uint, err error) {
	mulOf, err := f.mulPreCond(a, &Element[T]{Limbs: make([]frontend.Variable, f.fParams.NbLimbs()), overflow: 0}) // order is important, we want that reduce left side
	if err != nil {
		return mulOf, err
	}
	return f.subPreCond(&Element[T]{overflow: 0}, &Element[T]{overflow: mulOf})
}

func (f *Field[T]) inverse(a, _ *Element[T], _ uint) *Element[T] {
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
	return f.reduceAndOp(f.sqrt, f.sqrtPreCond, a, nil)
}

func (f *Field[T]) sqrtPreCond(a, _ *Element[T]) (nextOverflow uint, err error) {
	mulOf, err := f.mulPreCond(a, a)
	if err != nil {
		return mulOf, err
	}
	return f.subPreCond(a, &Element[T]{overflow: mulOf})
}

func (f *Field[T]) sqrt(a, _ *Element[T], _ uint) *Element[T] {
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

func (f *Field[T]) Sum(inputs ...*Element[T]) *Element[T] {
	if len(inputs) == 0 {
		return f.Zero()
	}
	if len(inputs) == 1 {
		return inputs[0]
	}
	overflow := uint(0)
	nbLimbs := 0
	for i := range inputs {
		f.enforceWidthConditional(inputs[i])
		if inputs[i].overflow > overflow {
			overflow = inputs[i].overflow
		}
		if len(inputs[i].Limbs) > nbLimbs {
			nbLimbs = len(inputs[i].Limbs)
		}
	}
	addOverflow := bits.Len(uint(len(inputs)))
	limbs := make([]frontend.Variable, nbLimbs)
	for i := range limbs {
		limbs[i] = 0
	}
	for i := range inputs {
		for j := range inputs[i].Limbs {
			limbs[j] = f.api.Add(limbs[j], inputs[i].Limbs[j])
		}
	}
	return f.newInternalElement(limbs, overflow+uint(addOverflow))
}

// Reduce reduces a modulo the field order and returns it.
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
	return f.mulMod(a, f.One(), 0, nil)
}

// Sub subtracts b from a and returns it. Reduces locally if wouldn't fit into
// Element. Doesn't mutate inputs.
func (f *Field[T]) Sub(a, b *Element[T]) *Element[T] {
	return f.reduceAndOp(f.sub, f.subPreCond, a, b)
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
	var fp T
	nbLimbs := max(len(a.Limbs), len(b.Limbs))
	limbs := make([]frontend.Variable, nbLimbs)
	padLimbs := subPadding(fp.Modulus(), fp.BitsPerLimb(), b.overflow, uint(nbLimbs))
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

// Mux selects element inputs[sel] and returns it. The number of the limbs and
// overflow in the result is the maximum of the inputs'. If the inputs are very
// unbalanced, then reduce the inputs before calling the method. It is most
// efficient for power of two lengths of the inputs, but works for any
// number of inputs.
func (f *Field[T]) Mux(sel frontend.Variable, inputs ...*Element[T]) *Element[T] {
	if len(inputs) == 0 {
		return nil
	}
	nbInputs := len(inputs)
	overflow := uint(0)
	nbLimbs := 0
	for i := range inputs {
		f.enforceWidthConditional(inputs[i])
		if inputs[i].overflow > overflow {
			overflow = inputs[i].overflow
		}
		if len(inputs[i].Limbs) > nbLimbs {
			nbLimbs = len(inputs[i].Limbs)
		}
	}
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
	normLimbs := make([][]frontend.Variable, nbInputs)
	for i := range inputs {
		normLimbs[i] = normalize(inputs[i].Limbs)
	}
	normLimbsTransposed := make([][]frontend.Variable, nbLimbs)
	for i := range normLimbsTransposed {
		normLimbsTransposed[i] = make([]frontend.Variable, nbInputs)
		for j := range normLimbsTransposed[i] {
			normLimbsTransposed[i][j] = normLimbs[j][i]
		}
	}
	e := f.newInternalElement(make([]frontend.Variable, nbLimbs), overflow)
	for i := range inputs[0].Limbs {
		e.Limbs[i] = selector.Mux(f.api, sel, normLimbsTransposed[i]...)
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
