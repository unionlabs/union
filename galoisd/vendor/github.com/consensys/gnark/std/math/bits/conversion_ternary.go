package bits

import (
	"math"
	"math/big"

	"github.com/consensys/gnark/frontend"
)

// ToTernary is an alias of ToBase(api, Ternary, v, opts...)
func ToTernary(api frontend.API, v frontend.Variable, opts ...BaseConversionOption) []frontend.Variable {
	return ToBase(api, Ternary, v, opts...)
}

// FromTernary is an alias of FromBase(api, Ternary, digits)
func FromTernary(api frontend.API, digits []frontend.Variable, opts ...BaseConversionOption) frontend.Variable {
	return FromBase(api, Ternary, digits, opts...)
}

func fromTernary(api frontend.API, digits []frontend.Variable, opts ...BaseConversionOption) frontend.Variable {
	cfg := baseConversionConfig{}

	for _, o := range opts {
		if err := o(&cfg); err != nil {
			panic(err)
		}
	}

	// Σti = Σ (3**i * b[i])
	Σti := frontend.Variable(0)

	c := big.NewInt(1)
	base := big.NewInt(3)

	for i := 0; i < len(digits); i++ {
		if !cfg.UnconstrainedInputs {
			// TODO ensures the digits are actual trits
			AssertIsTrit(api, digits[i])
		}
		Σti = api.Add(Σti, api.Mul(c, digits[i])) // no constraint is recorded
		c.Mul(c, base)
	}

	return Σti
}

func toTernary(api frontend.API, v frontend.Variable, opts ...BaseConversionOption) []frontend.Variable {
	// parse options
	nbBits := api.Compiler().FieldBitLen()
	nbTrits := int(float64(nbBits)/math.Log2(3.0)) + 1
	cfg := baseConversionConfig{
		NbDigits: nbTrits,
	}

	for _, o := range opts {
		if err := o(&cfg); err != nil {
			panic(err)
		}
	}

	c := big.NewInt(1)
	b := big.NewInt(3)

	trits, err := api.Compiler().NewHint(nTrits, cfg.NbDigits, v)
	if err != nil {
		panic(err)
	}

	var Σti frontend.Variable
	Σti = 0
	for i := 0; i < cfg.NbDigits; i++ {
		Σti = api.Add(Σti, api.Mul(trits[i], c))
		c.Mul(c, b)
		if !cfg.UnconstrainedOutputs {
			AssertIsTrit(api, trits[i])
		}
	}

	// record the constraint Σ (3**i * t[i]) == a
	api.AssertIsEqual(Σti, v)

	return trits
}

// AssertIsTrit constrains digit to be 0, 1 or 2.
func AssertIsTrit(api frontend.API, v frontend.Variable) {
	if c, ok := api.Compiler().ConstantValue(v); ok {
		if c.IsUint64() && c.Uint64() <= 2 {
			return
		}
		panic("value " + c.String() + " is not 0, 1 or 2")
	}

	// v * (1 - v) * (2 - v) == 0
	// TODO this adds 3 constraint, not 2. Need api.Compiler().AddConstraint(...)
	y := api.Mul(api.Sub(1, v), api.Sub(2, v))
	api.AssertIsEqual(api.Mul(v, y), 0)
}
