package emulated

import (
	"fmt"
	"math/big"

	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend"
)

func (f *Field[T]) wrapHint(nonnativeInputs ...*Element[T]) []frontend.Variable {
	res := []frontend.Variable{f.fParams.BitsPerLimb(), f.fParams.NbLimbs()}
	res = append(res, f.Modulus().Limbs...)
	res = append(res, len(nonnativeInputs))
	for i := range nonnativeInputs {
		res = append(res, len(nonnativeInputs[i].Limbs))
		res = append(res, nonnativeInputs[i].Limbs...)
	}
	return res
}

// UnwrapHint unwraps the native inputs into nonnative inputs. Then it calls
// nonnativeHint function with nonnative inputs. After nonnativeHint returns, it
// decomposes the outputs into limbs.
func UnwrapHint(nativeInputs, nativeOutputs []*big.Int, nonnativeHint solver.Hint) error {
	if len(nativeInputs) < 2 {
		return fmt.Errorf("hint wrapper header is 2 elements")
	}
	if !nativeInputs[0].IsInt64() || !nativeInputs[1].IsInt64() {
		return fmt.Errorf("header must be castable to int64")
	}
	nbBits := int(nativeInputs[0].Int64())
	nbLimbs := int(nativeInputs[1].Int64())
	if len(nativeInputs) < 2+nbLimbs {
		return fmt.Errorf("hint wrapper header is 2+nbLimbs elements")
	}
	nonnativeMod := new(big.Int)
	if err := recompose(nativeInputs[2:2+nbLimbs], uint(nbBits), nonnativeMod); err != nil {
		return fmt.Errorf("cannot recover nonnative mod: %w", err)
	}
	if !nativeInputs[2+nbLimbs].IsInt64() {
		return fmt.Errorf("number of nonnative elements must be castable to int64")
	}
	nbInputs := int(nativeInputs[2+nbLimbs].Int64())
	nonnativeInputs := make([]*big.Int, nbInputs)
	readPtr := 3 + nbLimbs
	for i := 0; i < nbInputs; i++ {
		if len(nativeInputs) < readPtr+1 {
			return fmt.Errorf("can not read %d-th native input", i)
		}
		if !nativeInputs[readPtr].IsInt64() {
			return fmt.Errorf("corrupted %d-th native input", i)
		}
		currentInputLen := int(nativeInputs[readPtr].Int64())
		if len(nativeInputs) < (readPtr + 1 + currentInputLen) {
			return fmt.Errorf("cannot read %d-th nonnative element", i)
		}
		nonnativeInputs[i] = new(big.Int)
		if err := recompose(nativeInputs[readPtr+1:readPtr+1+currentInputLen], uint(nbBits), nonnativeInputs[i]); err != nil {
			return fmt.Errorf("recompose %d-th element: %w", i, err)
		}
		readPtr += 1 + currentInputLen
	}
	if len(nativeOutputs)%nbLimbs != 0 {
		return fmt.Errorf("output count doesn't divide limb count")
	}
	nonnativeOutputs := make([]*big.Int, len(nativeOutputs)/nbLimbs)
	for i := range nonnativeOutputs {
		nonnativeOutputs[i] = new(big.Int)
	}
	if err := nonnativeHint(nonnativeMod, nonnativeInputs, nonnativeOutputs); err != nil {
		return fmt.Errorf("nonnative hint: %w", err)
	}
	for i := range nonnativeOutputs {
		nonnativeOutputs[i].Mod(nonnativeOutputs[i], nonnativeMod)
		if err := decompose(nonnativeOutputs[i], uint(nbBits), nativeOutputs[i*nbLimbs:(i+1)*nbLimbs]); err != nil {
			return fmt.Errorf("decompose %d-th element: %w", i, err)
		}
	}
	return nil
}

// NewHint allows to call the emulation hint function hf on inputs, expecting
// nbOutputs results. This function splits internally the emulated element into
// limbs and passes these to the hint function. There is [UnwrapHint] function
// which performs corresponding recomposition of limbs into integer values (and
// vice verse for output).
//
// The hint function for this method is defined as:
//
//	func HintFn(mod *big.Int, inputs, outputs []*big.Int) error {
//	    return emulated.UnwrapHint(inputs, outputs, func(mod *big.Int, inputs, outputs []*big.Int) error { // NB we shadow initial input, output, mod to avoid accidental overwrite!
//		    // here all inputs and outputs are modulo nonnative mod. we decompose them automatically
//	    })}
//
// See the example for full written example.
func (f *Field[T]) NewHint(hf solver.Hint, nbOutputs int, inputs ...*Element[T]) ([]*Element[T], error) {
	nativeInputs := f.wrapHint(inputs...)
	nbNativeOutputs := int(f.fParams.NbLimbs()) * nbOutputs
	nativeOutputs, err := f.api.Compiler().NewHint(hf, nbNativeOutputs, nativeInputs...)
	if err != nil {
		return nil, fmt.Errorf("call hint: %w", err)
	}
	outputs := make([]*Element[T], nbOutputs)
	for i := 0; i < nbOutputs; i++ {
		outputs[i] = f.packLimbs(nativeOutputs[i*int(f.fParams.NbLimbs()):(i+1)*int(f.fParams.NbLimbs())], true)
	}
	return outputs, nil
}
