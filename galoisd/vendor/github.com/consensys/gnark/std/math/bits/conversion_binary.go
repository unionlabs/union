package bits

import (
	"math/big"

	"github.com/consensys/gnark/frontend"
)

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

	// by default we also check that the value to be decomposed is less than the
	// modulus. However, we can omit the check when the number of bits we want
	// to decompose to is less than the modulus or it was strictly requested.
	omitReducednessCheck := cfg.omitModulusCheck || cfg.NbDigits < api.Compiler().FieldBitLen()

	// when cfg.NbDigits == 1, v itself has to be a binary digit. This if clause
	// saves one constraint.
	if cfg.NbDigits == 1 {
		api.AssertIsBoolean(v)
		return []frontend.Variable{v}
	}
	// if we decompose into more bits than fieldbitlen then the rest would be
	// always zeros. Reduce the always-zeros to have fewer edge-cases elsewhere.
	var paddingBits int
	if cfg.NbDigits > api.Compiler().FieldBitLen() {
		paddingBits = cfg.NbDigits - api.Compiler().FieldBitLen()
		cfg.NbDigits = api.Compiler().FieldBitLen()
	}

	c := big.NewInt(1)

	bits, err := api.Compiler().NewHint(nBits, cfg.NbDigits, v)
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
	if !omitReducednessCheck {
		if cmper, ok := api.Compiler().(bitsComparatorConstant); ok {
			bound := new(big.Int).Sub(api.Compiler().Field(), big.NewInt(1))
			cmper.MustBeLessOrEqCst(bits, bound, v)
		} else {
			panic("builder does not expose comparison to constant")
		}
	}

	// restore the zero bits which exceed the field bit-length when requested by
	// setting WithNbDigits larger than the field bitlength.
	bits = append(bits, make([]frontend.Variable, paddingBits)...)
	for i := cfg.NbDigits; i < len(bits); i++ {
		bits[i] = 0 // frontend.Variable is interface{}, we get nil pointer err if trying to access it.
	}

	return bits
}
