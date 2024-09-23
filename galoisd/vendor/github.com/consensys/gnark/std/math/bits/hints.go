package bits

import (
	"math/big"

	"github.com/consensys/gnark/constraint/solver"
)

func GetHints() []solver.Hint {
	return []solver.Hint{
		ithBit,
		nBits,
		nTrits,
	}
}

func init() {
	solver.RegisterHint(GetHints()...)
}

// IthBit returns the i-tb bit the input. The function expects exactly two
// integer inputs i and n, takes the little-endian bit representation of n and
// returns its i-th bit.
func ithBit(_ *big.Int, inputs []*big.Int, results []*big.Int) error {
	result := results[0]
	if !inputs[1].IsUint64() {
		result.SetUint64(0)
		return nil
	}

	result.SetUint64(uint64(inputs[0].Bit(int(inputs[1].Uint64()))))
	return nil
}

// NBits returns the first bits of the input. The number of returned bits is
// defined by the length of the results slice.
func nBits(_ *big.Int, inputs []*big.Int, results []*big.Int) error {
	n := inputs[0]
	for i := 0; i < len(results); i++ {
		results[i].SetUint64(uint64(n.Bit(i)))
	}
	return nil
}

// nTrits returns the first trits of the input. The number of returned trits is
// defined by the length of the results slice.
func nTrits(_ *big.Int, inputs []*big.Int, results []*big.Int) error {
	n := inputs[0]
	// TODO using big.Int Text method is likely not cheap
	base3 := n.Text(3)
	i := 0
	for j := len(base3) - 1; j >= 0 && i < len(results); j-- {
		results[i].SetUint64(uint64(base3[j] - 48))
		i++
	}
	for ; i < len(results); i++ {
		results[i].SetUint64(0)
	}

	return nil
}
