package uints

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
		andHint,
		xorHint,
		toBytes,
	}
}

func xorHint(_ *big.Int, inputs, outputs []*big.Int) error {
	outputs[0].Xor(inputs[0], inputs[1])
	return nil
}

func andHint(_ *big.Int, inputs, outputs []*big.Int) error {
	outputs[0].And(inputs[0], inputs[1])
	return nil
}

func toBytes(m *big.Int, inputs []*big.Int, outputs []*big.Int) error {
	if len(inputs) != 2 {
		return fmt.Errorf("input must be 2 elements")
	}
	if !inputs[0].IsUint64() {
		return fmt.Errorf("first input must be uint64")
	}
	nbLimbs := int(inputs[0].Uint64())
	if len(outputs) != nbLimbs {
		return fmt.Errorf("output must be 8 elements")
	}
	if !inputs[1].IsUint64() {
		return fmt.Errorf("input must be 64 bits")
	}
	base := new(big.Int).Lsh(big.NewInt(1), uint(8))
	tmp := new(big.Int).Set(inputs[1])
	for i := 0; i < nbLimbs; i++ {
		outputs[i].Mod(tmp, base)
		tmp.Rsh(tmp, 8)
	}
	return nil
}
