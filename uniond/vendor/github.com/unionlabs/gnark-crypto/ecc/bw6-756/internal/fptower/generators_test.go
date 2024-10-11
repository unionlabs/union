package fptower

import (
	"github.com/consensys/gnark-crypto/ecc/bw6-756/fp"
	"github.com/leanovate/gopter"
)

// TODO all gopter.Gen are incorrect, use same model as goff

// GenFp generates an Fp element
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

// GenE3 generates an E3 elmt
func GenE3() gopter.Gen {
	return gopter.CombineGens(
		GenFp(),
		GenFp(),
		GenFp(),
	).Map(func(values []interface{}) *E3 {
		return &E3{A0: values[0].(fp.Element), A1: values[1].(fp.Element), A2: values[2].(fp.Element)}
	})
}

// E6 generates an E6 elmt
func GenE6() gopter.Gen {
	return gopter.CombineGens(
		GenE3(),
		GenE3(),
	).Map(func(values []interface{}) *E6 {
		return &E6{B0: *values[0].(*E3), B1: *values[1].(*E3)}
	})
}
