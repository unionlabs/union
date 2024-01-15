package sw_emulated

import (
	"fmt"
	"math/big"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/std/math/emulated"
)

func init() {
	solver.RegisterHint(GetHints()...)
}

func GetHints() []solver.Hint {
	return []solver.Hint{decomposeScalarG1}
}

func decomposeScalarG1(mod *big.Int, inputs []*big.Int, outputs []*big.Int) error {
	return emulated.UnwrapHint(inputs, outputs, func(field *big.Int, inputs, outputs []*big.Int) error {
		if len(inputs) != 3 {
			return fmt.Errorf("expecting three inputs")
		}
		if len(outputs) != 5 {
			return fmt.Errorf("expecting five outputs")
		}
		glvBasis := new(ecc.Lattice)
		ecc.PrecomputeLattice(inputs[2], inputs[1], glvBasis)
		sp := ecc.SplitScalar(inputs[0], glvBasis)
		outputs[0].Set(&(sp[0]))
		outputs[1].Set(&(sp[1]))
		// figure out how many times we have overflowed
		outputs[1].Set(&(sp[1]))
		outputs[2].Mul(outputs[1], inputs[1]).Add(outputs[2], outputs[0])
		outputs[2].Sub(outputs[2], inputs[0])
		outputs[2].Div(outputs[2], inputs[2])

		outputs[3].Set(outputs[0])
		if outputs[0].Sign() == -1 {
			outputs[3].Neg(outputs[0])
		}
		outputs[4].Set(outputs[1])
		if outputs[1].Sign() == -1 {
			outputs[4].Neg(outputs[1])
		}

		return nil
	})
}
