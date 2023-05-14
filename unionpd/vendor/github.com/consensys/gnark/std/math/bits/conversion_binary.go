package bits

import (
	"math/big"

	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend"
)

func init() {
	// register hints
	solver.RegisterHint(IthBit)
	solver.RegisterHint(NBits)
}

// ToBinary is an alias of ToBase(api, Binary, v, opts)
func ToBinary(api frontend.API, v frontend.Variable, opts ...BaseConversionOption) []frontend.Variable {
	return ToBase(api, Binary, v, opts...)
}

// FromBinary is an alias of FromBase(api, Binary, digits)
func FromBinary(api frontend.API, digits []frontend.Variable, opts ...BaseConversionOption) frontend.Variable {
	return FromBase(api, Binary, digits, opts...)
}

func fromBinary(api frontend.API, digits []frontend.Variable, opts ...BaseConversionOption) frontend.Variable {

	cfg := baseConversionConfig{}

	for _, o := range opts {
		if err := o(&cfg); err != nil {
			panic(err)
		}
	}

	// Σbi = Σ (2**i * b[i])
	Σbi := frontend.Variable(0)

	c := big.NewInt(1)

	for i := 0; i < len(digits); i++ {
		if !cfg.UnconstrainedInputs {
			api.AssertIsBoolean(digits[i]) // ensures the digits are actual bits
		}

		Σbi = api.Add(Σbi, api.Mul(c, digits[i])) // no constraint is recorded
		c.Lsh(c, 1)
	}

	return Σbi
}

func toBinary(api frontend.API, v frontend.Variable, opts ...BaseConversionOption) []frontend.Variable {
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

	bits, err := api.Compiler().NewHint(NBits, cfg.NbDigits, v)
	if err != nil {
		panic(err)
	}

	var Σbi frontend.Variable
	Σbi = 0
	for i := 0; i < cfg.NbDigits; i++ {
		Σbi = api.Add(Σbi, api.Mul(bits[i], c))
		c.Lsh(c, 1)
		if !cfg.UnconstrainedOutputs {
			api.AssertIsBoolean(bits[i])
		}
	}

	// record the constraint Σ (2**i * b[i]) == a
	api.AssertIsEqual(Σbi, v)

	return bits
}

// IthBit returns the i-tb bit the input. The function expects exactly two
// integer inputs i and n, takes the little-endian bit representation of n and
// returns its i-th bit.
func IthBit(_ *big.Int, inputs []*big.Int, results []*big.Int) error {
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
func NBits(_ *big.Int, inputs []*big.Int, results []*big.Int) error {
	n := inputs[0]
	for i := 0; i < len(results); i++ {
		results[i].SetUint64(uint64(n.Bit(i)))
	}
	return nil
}
