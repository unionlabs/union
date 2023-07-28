package bits

import (
	"errors"
	"math/big"

	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend"
)

// NNAF returns the NAF decomposition of the input. The number of digits is
// defined by the number of elements in the results slice.
var NNAF = nNaf

func init() {
	solver.RegisterHint(NNAF)
}

// ToNAF returns the NAF decomposition of given input.
// The non-adjacent form (NAF) of a number is a unique signed-digit representation,
// in which non-zero values cannot be adjacent. For example, NAF(13) = [1, 0, -1, 0, 1].
func ToNAF(api frontend.API, v frontend.Variable, opts ...BaseConversionOption) []frontend.Variable {
	// parse options
	cfg := baseConversionConfig{
		NbDigits:             api.Compiler().FieldBitLen(),
		UnconstrainedOutputs: false,
	}

	for _, o := range opts {
		if err := o(&cfg); err != nil {
			panic(err)
		}
	}

	c := big.NewInt(1)

	bits, err := api.Compiler().NewHint(NNAF, cfg.NbDigits, v)
	if err != nil {
		panic(err)
	}

	var Σbi frontend.Variable
	Σbi = 0
	for i := 0; i < cfg.NbDigits; i++ {
		Σbi = api.Add(Σbi, api.Mul(bits[i], c))
		c.Lsh(c, 1)
		if !cfg.UnconstrainedOutputs {
			// b * (1 - b) * (1 + b) == 0
			// TODO this adds 3 constraint, not 2. Need api.Compiler().AddConstraint(...)
			b := bits[i]
			y := api.Mul(api.Sub(1, b), api.Add(1, b))
			api.AssertIsEqual(api.Mul(b, y), 0)
		}
	}

	// record the constraint Σ (2**i * b[i]) == v
	api.AssertIsEqual(Σbi, v)

	return bits
}

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
