package fptower

import (
	"github.com/consensys/gnark-crypto/ecc/bls12-377/fp"
	"github.com/leanovate/gopter"
)

// Fp generates an Fp element
func GenFp() gopter.Gen {
	return func(genParams *gopter.GenParameters) *gopter.GenResult {
		var elmt fp.Element

		if _, err := elmt.SetRandom(); err != nil {
			panic(err)
		}
		genResult := gopter.NewGenResult(elmt, gopter.NoShrinker)
		return genResult
	}
}

// E2 generates an E2 elmt
func GenE2() gopter.Gen {
	return gopter.CombineGens(
		GenFp(),
		GenFp(),
	).Map(func(values []interface{}) *E2 {
		return &E2{A0: values[0].(fp.Element), A1: values[1].(fp.Element)}
	})
}

// E6 generates an E6 elmt
func GenE6() gopter.Gen {
	return gopter.CombineGens(
		GenE2(),
		GenE2(),
		GenE2(),
	).Map(func(values []interface{}) *E6 {
		return &E6{B0: *values[0].(*E2), B1: *values[1].(*E2), B2: *values[2].(*E2)}
	})
}

// E12 generates an E6 elmt
func GenE12() gopter.Gen {
	return gopter.CombineGens(
		GenE6(),
		GenE6(),
	).Map(func(values []interface{}) *E12 {
		return &E12{C0: *values[0].(*E6), C1: *values[1].(*E6)}
	})
}
