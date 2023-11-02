package bits

import (
	"math/big"

	"github.com/consensys/gnark/frontend"
)

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

	bits, err := api.Compiler().NewHint(nNaf, cfg.NbDigits, v)
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
