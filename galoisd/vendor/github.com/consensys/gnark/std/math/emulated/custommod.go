package emulated

import (
	"errors"

	"github.com/consensys/gnark/frontend"
)

// ModMul computes a*b mod modulus. Instead of taking modulus as a constant
// parametrized by T, it is passed as an argument. This allows to use a variable
// modulus in the circuit. Type parameter T should be sufficiently big to fit a,
// b and modulus. Recommended to use [emparams.Mod1e512] or
// [emparams.Mod1e4096].
//
// NB! circuit complexity depends on T rather on the actual length of the modulus.
func (f *Field[T]) ModMul(a, b *Element[T], modulus *Element[T]) *Element[T] {
	res := f.mulMod(a, b, 0, modulus)
	return res
}

// ModAdd computes a+b mod modulus. Instead of taking modulus as a constant
// parametrized by T, it is passed as an argument. This allows to use a variable
// modulus in the circuit. Type parameter T should be sufficiently big to fit a,
// b and modulus. Recommended to use [emparams.Mod1e512] or
// [emparams.Mod1e4096].
//
// NB! circuit complexity depends on T rather on the actual length of the modulus.
func (f *Field[T]) ModAdd(a, b *Element[T], modulus *Element[T]) *Element[T] {
	// inlined version of [Field.reduceAndOp] which uses variable-modulus reduction
	var nextOverflow uint
	var err error
	var target overflowError
	for nextOverflow, err = f.addPreCond(a, b); errors.As(err, &target); nextOverflow, err = f.addPreCond(a, b) {
		if errors.As(err, &target) {
			if !target.reduceRight {
				a = f.mulMod(a, f.shortOne(), 0, modulus)
			} else {
				b = f.mulMod(b, f.shortOne(), 0, modulus)
			}
		}
	}
	res := f.add(a, b, nextOverflow)
	return res
}

func (f *Field[T]) modSub(a, b *Element[T], modulus *Element[T]) *Element[T] {
	// like fixed modulus subtraction, but for sub padding need to use hint
	// instead of assuming T as a constant. And when doing as a hint, then need
	// to assert that the padding is a multiple of the modulus (done inside callSubPaddingHint)
	nextOverflow := max(b.overflow+1, a.overflow) + 1
	nbLimbs := max(len(a.Limbs), len(b.Limbs))
	limbs := make([]frontend.Variable, nbLimbs)
	padding := f.computeSubPaddingHint(b.overflow, uint(nbLimbs), modulus)
	for i := range limbs {
		limbs[i] = padding.Limbs[i]
		if i < len(a.Limbs) {
			limbs[i] = f.api.Add(limbs[i], a.Limbs[i])
		}
		if i < len(b.Limbs) {
			limbs[i] = f.api.Sub(limbs[i], b.Limbs[i])
		}
	}
	res := f.newInternalElement(limbs, nextOverflow)
	return res
}

// ModAssertIsEqual asserts equality of a and b mod modulus. Instead of taking
// modulus as a constant parametrized by T, it is passed as an argument. This
// allows to use a variable modulus in the circuit. Type parameter T should be
// sufficiently big to fit a, b and modulus. Recommended to use
// [emparams.Mod1e512] or [emparams.Mod1e4096].
//
// NB! circuit complexity depends on T rather on the actual length of the modulus.
func (f *Field[T]) ModAssertIsEqual(a, b *Element[T], modulus *Element[T]) {
	// like fixed modulus AssertIsEqual, but uses current Sub implementation for
	// computing the diff
	diff := f.modSub(b, a, modulus)
	f.checkZero(diff, modulus)
}

// ModExp computes base^exp mod modulus. Instead of taking modulus as a constant
// parametrized by T, it is passed as an argument. This allows to use a variable
// modulus in the circuit. Type parameter T should be sufficiently big to fit
// base, exp and modulus. Recommended to use [emparams.Mod1e512] or
// [emparams.Mod1e4096].
//
// NB! circuit complexity depends on T rather on the actual length of the modulus.
func (f *Field[T]) ModExp(base, exp, modulus *Element[T]) *Element[T] {
	expBts := f.ToBits(exp)
	n := len(expBts)
	res := f.Select(expBts[0], base, f.One())
	base = f.ModMul(base, base, modulus)
	for i := 1; i < n-1; i++ {
		res = f.Select(expBts[i], f.ModMul(base, res, modulus), res)
		base = f.ModMul(base, base, modulus)
	}
	res = f.Select(expBts[n-1], f.ModMul(base, res, modulus), res)
	return res
}
