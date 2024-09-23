package emulated

import (
	"fmt"
	"math/big"

	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend"
)

// TODO @gbotrel hint[T FieldParams] would simplify this . Issue is when registering hint, if QuoRem[T] was declared
// inside a func, then it becomes anonymous and hint identification is screwed.

func init() {
	solver.RegisterHint(GetHints()...)
}

// GetHints returns all hint functions used in the package.
func GetHints() []solver.Hint {
	return []solver.Hint{
		DivHint,
		InverseHint,
		SqrtHint,
		mulHint,
		subPaddingHint,
	}
}

// nbMultiplicationResLimbs returns the number of limbs which fit the
// multiplication result.
func nbMultiplicationResLimbs(lenLeft, lenRight int) int {
	res := lenLeft + lenRight - 1
	if res < 0 {
		res = 0
	}
	return res
}

// computeInverseHint packs the inputs for the InverseHint hint function.
func (f *Field[T]) computeInverseHint(inLimbs []frontend.Variable) (inverseLimbs []frontend.Variable, err error) {
	var fp T
	hintInputs := []frontend.Variable{
		fp.BitsPerLimb(),
		fp.NbLimbs(),
	}
	p := f.Modulus()
	hintInputs = append(hintInputs, p.Limbs...)
	hintInputs = append(hintInputs, inLimbs...)
	return f.api.NewHint(InverseHint, int(fp.NbLimbs()), hintInputs...)
}

// InverseHint computes the inverse x^-1 for the input x and stores it in outputs.
func InverseHint(mod *big.Int, inputs []*big.Int, outputs []*big.Int) error {
	if len(inputs) < 2 {
		return fmt.Errorf("input must be at least two elements")
	}
	nbBits := uint(inputs[0].Uint64())
	nbLimbs := int(inputs[1].Int64())
	if len(inputs[2:]) < 2*nbLimbs {
		return fmt.Errorf("inputs missing")
	}
	if len(outputs) != nbLimbs {
		return fmt.Errorf("result does not fit into output")
	}
	p := new(big.Int)
	if err := recompose(inputs[2:2+nbLimbs], nbBits, p); err != nil {
		return fmt.Errorf("recompose emulated order: %w", err)
	}
	x := new(big.Int)
	if err := recompose(inputs[2+nbLimbs:], nbBits, x); err != nil {
		return fmt.Errorf("recompose value: %w", err)
	}
	if x.ModInverse(x, p) == nil {
		return fmt.Errorf("input and modulus not relatively primes")
	}
	if err := decompose(x, nbBits, outputs); err != nil {
		return fmt.Errorf("decompose: %w", err)
	}
	return nil
}

// computeDivisionHint packs the inputs for DivisionHint hint function.
func (f *Field[T]) computeDivisionHint(nomLimbs, denomLimbs []frontend.Variable) (divLimbs []frontend.Variable, err error) {
	var fp T
	hintInputs := []frontend.Variable{
		fp.BitsPerLimb(),
		fp.NbLimbs(),
		len(denomLimbs),
		len(nomLimbs),
	}
	p := f.Modulus()
	hintInputs = append(hintInputs, p.Limbs...)
	hintInputs = append(hintInputs, nomLimbs...)
	hintInputs = append(hintInputs, denomLimbs...)
	return f.api.NewHint(DivHint, int(fp.NbLimbs()), hintInputs...)
}

// DivHint computes the value z = x/y for inputs x and y and stores z in
// outputs.
func DivHint(mod *big.Int, inputs []*big.Int, outputs []*big.Int) error {
	if len(inputs) < 3 {
		return fmt.Errorf("input must be at least three elements")
	}
	nbBits := uint(inputs[0].Uint64())
	nbLimbs := int(inputs[1].Int64())
	nbDenomLimbs := int(inputs[2].Int64())
	// nominator does not have to be reduced and can be more than nbLimbs.
	// Denominator and order have to be nbLimbs long.
	nbNomLimbs := int(inputs[3].Int64())
	if len(inputs[4:]) != nbLimbs+nbNomLimbs+nbDenomLimbs {
		return fmt.Errorf("input length mismatch")
	}
	if len(outputs) != nbLimbs {
		return fmt.Errorf("result does not fit into output")
	}
	p := new(big.Int)
	if err := recompose(inputs[4:4+nbLimbs], nbBits, p); err != nil {
		return fmt.Errorf("recompose emulated order: %w", err)
	}
	nominator := new(big.Int)
	if err := recompose(inputs[4+nbLimbs:4+nbLimbs+nbNomLimbs], nbBits, nominator); err != nil {
		return fmt.Errorf("recompose nominator: %w", err)
	}
	denominator := new(big.Int)
	if err := recompose(inputs[4+nbLimbs+nbNomLimbs:], nbBits, denominator); err != nil {
		return fmt.Errorf("recompose denominator: %w", err)
	}
	res := new(big.Int).ModInverse(denominator, p)
	if res == nil {
		return fmt.Errorf("no modular inverse")
	}
	res.Mul(res, nominator)
	res.Mod(res, p)
	if err := decompose(res, nbBits, outputs); err != nil {
		return fmt.Errorf("decompose division: %w", err)
	}
	return nil
}

// SqrtHint compute square root of the input.
func SqrtHint(mod *big.Int, inputs []*big.Int, outputs []*big.Int) error {
	return UnwrapHint(inputs, outputs, func(field *big.Int, inputs, outputs []*big.Int) error {
		if len(inputs) != 1 {
			return fmt.Errorf("expecting single input")
		}
		if len(outputs) != 1 {
			return fmt.Errorf("expecting single output")
		}
		res := new(big.Int)
		if res.ModSqrt(inputs[0], field) == nil {
			return fmt.Errorf("no square root")
		}
		outputs[0].Set(res)
		return nil
	})
}

// subPaddingHint computes the padding for the subtraction of two numbers. It
// ensures that the padding is a multiple of the modulus. Can be used to avoid
// underflow.
//
// In case of fixed modulus use subPadding instead.
func subPaddingHint(mod *big.Int, inputs, outputs []*big.Int) error {
	if len(inputs) < 4 {
		return fmt.Errorf("input must be at least four elements")
	}
	nbLimbs := int(inputs[0].Int64())
	bitsPerLimbs := uint(inputs[1].Uint64())
	overflow := uint(inputs[2].Uint64())
	retLimbs := int(inputs[3].Int64())
	if len(inputs[4:]) != nbLimbs {
		return fmt.Errorf("input length mismatch")
	}
	if len(outputs) != retLimbs {
		return fmt.Errorf("result does not fit into output")
	}
	pLimbs := inputs[4 : 4+nbLimbs]
	p := new(big.Int)
	if err := recompose(pLimbs, bitsPerLimbs, p); err != nil {
		return fmt.Errorf("recompose modulus: %w", err)
	}
	padLimbs := subPadding(p, bitsPerLimbs, overflow, uint(nbLimbs))
	for i := range padLimbs {
		outputs[i].Set(padLimbs[i])
	}

	return nil
}

func (f *Field[T]) computeSubPaddingHint(overflow uint, nbLimbs uint, modulus *Element[T]) *Element[T] {
	var fp T
	inputs := []frontend.Variable{fp.NbLimbs(), fp.BitsPerLimb(), overflow, nbLimbs}
	inputs = append(inputs, modulus.Limbs...)
	res, err := f.api.NewHint(subPaddingHint, int(nbLimbs), inputs...)
	if err != nil {
		panic(fmt.Sprintf("sub padding hint: %v", err))
	}
	for i := range res {
		f.checker.Check(res[i], int(fp.BitsPerLimb()+overflow+1))
	}
	padding := f.newInternalElement(res, fp.BitsPerLimb()+overflow+1)
	f.checkZero(padding, modulus)
	return padding
}
