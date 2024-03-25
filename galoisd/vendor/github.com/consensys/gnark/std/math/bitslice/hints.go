package bitslice

import (
	"fmt"
	"math/big"

	"github.com/consensys/gnark/constraint/solver"
)

func init() {
	solver.RegisterHint(GetHints()...)
}

func GetHints() []solver.Hint {
	return []solver.Hint{
		partitionHint,
	}
}

func partitionHint(_ *big.Int, inputs []*big.Int, outputs []*big.Int) error {
	if len(inputs) != 2 {
		return fmt.Errorf("expecting two inputs")
	}
	if len(outputs) != 2 {
		return fmt.Errorf("expecting two outputs")
	}
	if !inputs[0].IsUint64() {
		return fmt.Errorf("split location must be int")
	}
	split := uint(inputs[0].Uint64())
	div := new(big.Int).Lsh(big.NewInt(1), split)
	outputs[0].QuoRem(inputs[1], div, outputs[1])
	return nil
}
