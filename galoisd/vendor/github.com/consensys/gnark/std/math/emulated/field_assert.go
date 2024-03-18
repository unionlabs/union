package emulated

import (
	"fmt"

	"github.com/consensys/gnark/frontend"
)

// enforceWidth enforces the width of the limbs. When modWidth is true, then the
// limbs are asserted to be the width of the modulus (highest limb may be less
// than full limb width). Otherwise, every limb is assumed to have same width
// (defined by the field parameter).
func (f *Field[T]) enforceWidth(a *Element[T], modWidth bool) {
	if _, aConst := f.constantValue(a); aConst {
		if modWidth && len(a.Limbs) != int(f.fParams.NbLimbs()) {
			panic("constant limb width doesn't match parametrized field")
		}
	}
	if modWidth && len(a.Limbs) != int(f.fParams.NbLimbs()) {
		panic("enforcing modulus width element with inexact number of limbs")
	}

	for i := range a.Limbs {
		limbNbBits := int(f.fParams.BitsPerLimb())
		if modWidth && i == len(a.Limbs)-1 {
			// take only required bits from the most significant limb
			limbNbBits = ((f.fParams.Modulus().BitLen() - 1) % int(f.fParams.BitsPerLimb())) + 1
		}
		f.checker.Check(a.Limbs[i], limbNbBits)
	}
}

// AssertIsEqual ensures that a is equal to b modulo the modulus.
func (f *Field[T]) AssertIsEqual(a, b *Element[T]) {
	f.enforceWidthConditional(a)
	f.enforceWidthConditional(b)
	ba, aConst := f.constantValue(a)
	bb, bConst := f.constantValue(b)
	if aConst && bConst {
		ba.Mod(ba, f.fParams.Modulus())
		bb.Mod(bb, f.fParams.Modulus())
		if ba.Cmp(bb) != 0 {
			panic(fmt.Sprintf("%s != %s", ba, bb))
		}
		return
	}

	diff := f.Sub(b, a)
	f.checkZero(diff)
}

// AssertIsLessOrEqual ensures that e is less or equal than a. For proper
// bitwise comparison first reduce the element using [Reduce] and then assert
// that its value is less than the modulus using [AssertIsInRange].
func (f *Field[T]) AssertIsLessOrEqual(e, a *Element[T]) {
	// we omit conditional width assertion as is done in ToBits below
	if e.overflow+a.overflow > 0 {
		panic("inputs must have 0 overflow")
	}
	eBits := f.ToBits(e)
	aBits := f.ToBits(a)
	ff := func(xbits, ybits []frontend.Variable) []frontend.Variable {
		diff := len(xbits) - len(ybits)
		ybits = append(ybits, make([]frontend.Variable, diff)...)
		for i := len(ybits) - diff; i < len(ybits); i++ {
			ybits[i] = 0
		}
		return ybits
	}
	if len(eBits) > len(aBits) {
		aBits = ff(eBits, aBits)
	} else {
		eBits = ff(aBits, eBits)
	}
	p := make([]frontend.Variable, len(eBits)+1)
	p[len(eBits)] = 1
	for i := len(eBits) - 1; i >= 0; i-- {
		v := f.api.Mul(p[i+1], eBits[i])
		p[i] = f.api.Select(aBits[i], v, p[i+1])
		t := f.api.Select(aBits[i], 0, p[i+1])
		l := f.api.Sub(1, t, eBits[i])
		ll := f.api.Mul(l, eBits[i])
		f.api.AssertIsEqual(ll, 0)
	}
}

// AssertIsInRange ensures that a is less than the emulated modulus. When we
// call [Reduce] then we only ensure that the result is width-constrained, but
// not actually less than the modulus. This means that the actual value may be
// either x or x + p. For arithmetic it is sufficient, but for binary comparison
// it is not. For binary comparison the values have both to be below the
// modulus.
func (f *Field[T]) AssertIsInRange(a *Element[T]) {
	// we omit conditional width assertion as is done in ToBits down the calling stack
	f.AssertIsLessOrEqual(a, f.modulusPrev())
}

// IsZero returns a boolean indicating if the element is strictly zero. The
// method internally reduces the element and asserts that the value is less than
// the modulus.
func (f *Field[T]) IsZero(a *Element[T]) frontend.Variable {
	ca := f.Reduce(a)
	f.AssertIsInRange(ca)
	// we use two approaches for checking if the element is exactly zero. The
	// first approach is to check that every limb individually is zero. The
	// second approach is to check if the sum of all limbs is zero. Usually, we
	// cannot use this approach as we could have false positive due to overflow
	// in the native field. However, as the widths of the limbs are restricted,
	// then we can ensure in most cases that no overflows happen.

	// as ca is already reduced, then every limb overflow is already 0. Only
	// every addition adds a bit to the overflow
	totalOverflow := len(ca.Limbs) - 1
	if totalOverflow < int(f.maxOverflow()) {
		// the sums of limbs would overflow the native field. Use the first
		// approach instead.
		res := f.api.IsZero(ca.Limbs[0])
		for i := 1; i < len(ca.Limbs); i++ {
			res = f.api.Mul(res, f.api.IsZero(ca.Limbs[i]))
		}
		return res
	}
	// default case, limbs sum does not overflow the native field
	limbSum := ca.Limbs[0]
	for i := 1; i < len(ca.Limbs); i++ {
		limbSum = f.api.Add(limbSum, ca.Limbs[i])
	}
	return f.api.IsZero(limbSum)
}

// // Cmp returns:
// //   - -1 if a < b
// //   - 0 if a = b
// //   - 1 if a > b
// //
// // The method internally reduces the element and asserts that the value is less
// // than the modulus.
// func (f *Field[T]) Cmp(a, b *Element[T]) frontend.Variable {
// 	ca := f.Reduce(a)
// 	f.AssertIsInRange(ca)
// 	cb := f.Reduce(b)
// 	f.AssertIsInRange(cb)
// 	var res frontend.Variable = 0
// 	for i := int(f.fParams.NbLimbs() - 1); i >= 0; i-- {
// 		lmbCmp := f.api.Cmp(ca.Limbs[i], cb.Limbs[i])
// 		res = f.api.Select(f.api.IsZero(res), lmbCmp, res)
// 	}
// 	return res
// }

// TODO(@ivokub)
// func (f *Field[T]) AssertIsDifferent(a, b *Element[T]) {
// 	ca := f.Reduce(a)
// 	f.AssertIsInRange(ca)
// 	cb := f.Reduce(b)
// 	f.AssertIsInRange(cb)
// 	var res frontend.Variable = 0
// 	for i := 0; i < int(f.fParams.NbLimbs()); i++ {
// 		cmp := f.api.Cmp(ca.Limbs[i], cb.Limbs[i])
// 		cmpsq := f.api.Mul(cmp, cmp)
// 		res = f.api.Add(res, cmpsq)
// 	}
// 	f.api.AssertIsDifferent(res, 0)
// }
