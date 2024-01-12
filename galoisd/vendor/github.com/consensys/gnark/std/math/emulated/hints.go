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
		QuoHint,
		InverseHint,
		MultiplicationHint,
		RightShift,
		SqrtHint,
		mulHint,
	}
}

// computeMultiplicationHint packs the inputs for the MultiplicationHint hint function.
func (f *Field[T]) computeMultiplicationHint(leftLimbs, rightLimbs []frontend.Variable) (mulLimbs []frontend.Variable, err error) {
	hintInputs := []frontend.Variable{
		f.fParams.BitsPerLimb(),
		len(leftLimbs),
		len(rightLimbs),
	}
	hintInputs = append(hintInputs, leftLimbs...)
	hintInputs = append(hintInputs, rightLimbs...)
	return f.api.NewHint(MultiplicationHint, nbMultiplicationResLimbs(len(leftLimbs), len(rightLimbs)), hintInputs...)
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

// MultiplicationHint unpacks the factors and parameters from inputs, computes
// the product and stores it in output. See internal method
// computeMultiplicationHint for the input packing.
func MultiplicationHint(mod *big.Int, inputs []*big.Int, outputs []*big.Int) error {
	if len(inputs) < 3 {
		return fmt.Errorf("input must be at least three elements")
	}
	nbBits := int(inputs[0].Int64())
	if 2*nbBits+1 >= mod.BitLen() {
		return fmt.Errorf("can not fit multiplication result into limb of length %d", nbBits)
	}
	// TODO: check that the scalar field fits 2*nbBits + nbLimbs. 2*nbBits comes
	// from multiplication and nbLimbs comes from additions.
	// TODO: check that all limbs all fully reduced
	nbLimbsLeft := int(inputs[1].Int64())
	// TODO: get the limb length from the input instead of packing into input
	nbLimbsRight := int(inputs[2].Int64())
	if len(inputs) != 3+nbLimbsLeft+nbLimbsRight {
		return fmt.Errorf("input invalid")
	}
	if len(outputs) < nbLimbsLeft+nbLimbsRight-1 {
		return fmt.Errorf("can not fit multiplication result into %d limbs", len(outputs))
	}
	for _, oi := range outputs {
		if oi == nil {
			return fmt.Errorf("output not initialized")
		}
		oi.SetUint64(0)
	}
	tmp := new(big.Int)
	for i, li := range inputs[3 : 3+nbLimbsLeft] {
		for j, rj := range inputs[3+nbLimbsLeft:] {
			outputs[i+j].Add(outputs[i+j], tmp.Mul(li, rj))
		}
	}
	return nil
}

// computeQuoHint packs the inputs for QuoHint function and returns z = x / y
// (discards remainder)
func (f *Field[T]) computeQuoHint(x *Element[T]) (z *Element[T], err error) {
	var fp T
	resLen := (uint(len(x.Limbs))*fp.BitsPerLimb() + x.overflow + 1 - // diff total bitlength
		uint(fp.Modulus().BitLen()) + // subtract modulus bitlength
		fp.BitsPerLimb() - 1) / // to round up
		fp.BitsPerLimb()

	hintInputs := []frontend.Variable{
		fp.BitsPerLimb(),
		len(x.Limbs),
	}
	p := f.Modulus()
	hintInputs = append(hintInputs, x.Limbs...)
	hintInputs = append(hintInputs, p.Limbs...)

	limbs, err := f.api.NewHint(QuoHint, int(resLen), hintInputs...)
	if err != nil {
		return nil, err
	}

	return f.packLimbs(limbs, false), nil
}

// QuoHint sets z to the quotient x/y for y != 0 and returns z.
// If y == 0, returns an error.
// Quo implements truncated division (like Go); see QuoRem for more details.
func QuoHint(_ *big.Int, inputs []*big.Int, outputs []*big.Int) error {
	nbBits, _, x, y, err := parseHintDivInputs(inputs)
	if err != nil {
		return err
	}
	z := new(big.Int)
	z.Quo(x, y) //.Mod(z, y)

	if err := decompose(z, nbBits, outputs); err != nil {
		return fmt.Errorf("decompose: %w", err)
	}

	return nil
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

// input[0] = nbBits per limb
// input[1] = nbLimbs(x)
// input[2:2+nbLimbs(x)] = limbs(x)
// input[2+nbLimbs(x):] = limbs(y)
// errors if y == 0
func parseHintDivInputs(inputs []*big.Int) (uint, int, *big.Int, *big.Int, error) {
	if len(inputs) < 2 {
		return 0, 0, nil, nil, fmt.Errorf("at least 2 inputs required")
	}
	nbBits := uint(inputs[0].Uint64())
	nbLimbs := int(inputs[1].Int64())
	if len(inputs[2:]) < nbLimbs {
		return 0, 0, nil, nil, fmt.Errorf("x limbs missing")
	}
	x, y := new(big.Int), new(big.Int)
	if err := recompose(inputs[2:2+nbLimbs], nbBits, x); err != nil {
		return 0, 0, nil, nil, fmt.Errorf("recompose x: %w", err)
	}
	if err := recompose(inputs[2+nbLimbs:], nbBits, y); err != nil {
		return 0, 0, nil, nil, fmt.Errorf("recompose y: %w", err)
	}
	if y.IsUint64() && y.Uint64() == 0 {
		return 0, 0, nil, nil, fmt.Errorf("y == 0")
	}
	return nbBits, nbLimbs, x, y, nil
}

// RightShift shifts input by the given number of bits. Expects two inputs:
//   - first input is the shift, will be represented as uint64;
//   - second input is the value to be shifted.
//
// Returns a single output which is the value shifted. Errors if number of
// inputs is not 2 and number of outputs is not 1.
func RightShift(_ *big.Int, inputs []*big.Int, outputs []*big.Int) error {
	if len(inputs) != 2 {
		return fmt.Errorf("expecting two inputs")
	}
	if len(outputs) != 1 {
		return fmt.Errorf("expecting single output")
	}
	shift := inputs[0].Uint64()
	outputs[0].Rsh(inputs[1], uint(shift))
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
