package bits

import (
	"errors"
	"math/big"

	"github.com/consensys/gnark/constraint/solver"
)

func GetHints() []solver.Hint {
	return []solver.Hint{
		ithBit,
		nBits,
		nTrits,
		nNaf,
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

// NNAF returns the NAF decomposition of the input. The number of digits is
// defined by the number of elements in the results slice.
func nNaf(_ *big.Int, inputs []*big.Int, results []*big.Int) error {
	n := inputs[0]
	return nafDecomposition(n, results)
}

// nafDecomposition gets the naf decomposition of a big number
func nafDecomposition(a *big.Int, results []*big.Int) error {
	if a == nil || a.Sign() == -1 {
		return errors.New("invalid input to naf decomposition; negative (or nil) big.Int not supported")
	}

	var zero, one, three big.Int

	one.SetUint64(1)
	three.SetUint64(3)

	n := 0

	// some buffers
	var buf, aCopy big.Int
	aCopy.Set(a)

	for aCopy.Cmp(&zero) != 0 && n < len(results) {

		// if aCopy % 2 == 0
		buf.And(&aCopy, &one)

		// aCopy even
		if buf.Cmp(&zero) == 0 {
			results[n].SetUint64(0)
		} else { // aCopy odd
			buf.And(&aCopy, &three)
			if buf.IsUint64() && buf.Uint64() == 3 {
				results[n].SetInt64(-1)
				aCopy.Add(&aCopy, &one)
			} else {
				results[n].SetUint64(1)
			}
		}
		aCopy.Rsh(&aCopy, 1)
		n++
	}
	for ; n < len(results); n++ {
		results[n].SetUint64(0)
	}

	return nil
}
