// Package logderivarg implements log-derivative argument.
//
// The log-derivative argument was described in [Haböck22] as an improvement
// over [BCG+18]. In [BCG+18], it was shown that to show inclusion of a multiset
// S in T, one can show
//
//	∏_{f∈F} (x-f)^count(f, S) == ∏_{s∈S} x-s,
//
// where function `count` counts the number of occurences of f in S. The problem
// with this approach is the high cost for exponentiating the left-hand side of
// the equation. However, in [Haböck22] it was shown that when avoiding the
// poles, we can perform the same check for the log-derivative variant of the
// equation:
//
//	∑_{f∈F} count(f,S)/(x-f) == ∑_{s∈S} 1/(x-s).
//
// Additionally, when the entries of both S and T are vectors, then instead we
// can check random linear combinations. So, when F is a matrix and S is a
// multiset of its rows, we first generate random linear coefficients (r_1, ...,
// r_n) and check
//
//	∑_{f∈F} count(f,S)/(x-∑_{i∈[n]}r_i*f_i) == ∑_{s∈S} 1/(x-∑_{i∈[n]}r_i*s_i).
//
// This package is a low-level primitive for building more extensive gadgets. It
// only checks the last equation, but the tables and queries should be built by
// the users.
//
// NB! The package doesn't check that the entries in table F are unique.
//
// [BCG+18]: https://eprint.iacr.org/2018/380
// [Haböck22]: https://eprint.iacr.org/2022/1530
package logderivarg

// TODO: we handle both constant and variable tables. But for variable tables we
// have to ensure that all the table entries differ! Right now isn't a problem
// because everywhere we build we also have indices which ensure uniqueness. I
// guess the best approach is to have safe and unsafe versions where the safe
// version performs additional sorting. But that is really really expensive as
// we have to show that all sorted values ara monotonically increasing.

import (
	"fmt"
	"math/big"

	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/hash/mimc"
	"github.com/consensys/gnark/std/multicommit"
)

func init() {
	solver.RegisterHint(GetHints()...)
}

// GetHints returns all hints used in this package
func GetHints() []solver.Hint {
	return []solver.Hint{countHint}
}

// Table is a vector of vectors.
type Table [][]frontend.Variable

// AsTable returns a vector as a single-column table.
func AsTable(vector []frontend.Variable) Table {
	ret := make([][]frontend.Variable, len(vector))
	for i := range vector {
		ret[i] = []frontend.Variable{vector[i]}
	}
	return ret
}

// Build builds the argument using the table and queries. If both table and
// queries are multiple-column, then also samples coefficients for the random
// linear combinations.
func Build(api frontend.API, table Table, queries Table) error {
	if len(table) == 0 {
		return fmt.Errorf("table empty")
	}
	nbRow := len(table[0])
	constTable := true
	countInputs := []frontend.Variable{len(table), nbRow}
	for i := range table {
		if len(table[i]) != nbRow {
			return fmt.Errorf("table row length mismatch")
		}
		if constTable {
			for j := range table[i] {
				if _, isConst := api.Compiler().ConstantValue(table[i][j]); !isConst {
					constTable = false
				}
			}
		}
		countInputs = append(countInputs, table[i]...)
	}
	for i := range queries {
		if len(queries[i]) != nbRow {
			return fmt.Errorf("query row length mismatch")
		}
		countInputs = append(countInputs, queries[i]...)
	}
	exps, err := api.NewHint(countHint, len(table), countInputs...)
	if err != nil {
		return fmt.Errorf("hint: %w", err)
	}

	var toCommit []frontend.Variable
	if !constTable {
		for i := range table {
			toCommit = append(toCommit, table[i]...)
		}
	}
	for i := range queries {
		toCommit = append(toCommit, queries[i]...)
	}
	toCommit = append(toCommit, exps...)

	multicommit.WithCommitment(api, func(api frontend.API, commitment frontend.Variable) error {
		rowCoeffs, challenge := randLinearCoefficients(api, nbRow, commitment)
		var lp frontend.Variable = 0
		for i := range table {
			tmp := api.DivUnchecked(exps[i], api.Sub(challenge, randLinearCombination(api, rowCoeffs, table[i])))
			lp = api.Add(lp, tmp)
		}
		var rp frontend.Variable = 0

		toInvert := make([]frontend.Variable, len(queries))
		for i := range queries {
			toInvert[i] = api.Sub(challenge, randLinearCombination(api, rowCoeffs, queries[i]))
		}

		if bapi, ok := api.(frontend.BatchInverter); ok {
			toInvert = bapi.BatchInvert(toInvert)
		} else {
			for i := range toInvert {
				toInvert[i] = api.Inverse(toInvert[i])
			}
		}

		for i := range queries {
			// tmp := api.Inverse(api.Sub(challenge, randLinearCombination(api, rowCoeffs, queries[i])))
			rp = api.Add(rp, toInvert[i])
		}
		api.AssertIsEqual(lp, rp)
		return nil
	}, toCommit...)
	return nil
}

func randLinearCoefficients(api frontend.API, nbRow int, commitment frontend.Variable) (rowCoeffs []frontend.Variable, challenge frontend.Variable) {
	if nbRow == 1 {
		return []frontend.Variable{1}, commitment
	}
	hasher, err := mimc.NewMiMC(api)
	if err != nil {
		panic(err)
	}
	rowCoeffs = make([]frontend.Variable, nbRow)
	for i := 0; i < nbRow; i++ {
		hasher.Reset()
		hasher.Write(i+1, commitment)
		rowCoeffs[i] = hasher.Sum()
	}
	return rowCoeffs, commitment
}

func randLinearCombination(api frontend.API, rowCoeffs []frontend.Variable, row []frontend.Variable) frontend.Variable {
	if len(rowCoeffs) != len(row) {
		panic("coefficient count mismatch")
	}
	var res frontend.Variable = 0
	for i := range rowCoeffs {
		res = api.Add(res, api.Mul(rowCoeffs[i], row[i]))
	}
	return res
}

func countHint(m *big.Int, inputs []*big.Int, outputs []*big.Int) error {
	if len(inputs) <= 2 {
		return fmt.Errorf("at least two input required")
	}
	if !inputs[0].IsInt64() {
		return fmt.Errorf("first element must be length of table")
	}
	nbTable := int(inputs[0].Int64())
	if !inputs[1].IsInt64() {
		return fmt.Errorf("first element must be length of row")
	}
	nbRow := int(inputs[1].Int64())
	if len(inputs) < 2+nbTable {
		return fmt.Errorf("input doesn't fit table")
	}
	if len(outputs) != nbTable {
		return fmt.Errorf("output not table size")
	}
	if (len(inputs)-2-nbTable*nbRow)%nbRow != 0 {
		return fmt.Errorf("query count not full integer")
	}
	nbQueries := (len(inputs) - 2 - nbTable*nbRow) / nbRow
	if nbQueries <= 0 {
		return fmt.Errorf("at least one query required")
	}
	nbBytes := (m.BitLen() + 7) / 8
	buf := make([]byte, nbBytes*nbRow)
	histo := make(map[string]int64, nbTable) // string key as big ints not comparable
	for i := 0; i < nbTable; i++ {
		for j := 0; j < nbRow; j++ {
			inputs[2+nbRow*i+j].FillBytes(buf[j*nbBytes : (j+1)*nbBytes])
		}
		k := string(buf)
		if _, ok := histo[k]; ok {
			return fmt.Errorf("duplicate key")
		}
		histo[k] = 0
	}
	for i := 0; i < nbQueries; i++ {
		for j := 0; j < nbRow; j++ {
			inputs[2+nbRow*nbTable+nbRow*i+j].FillBytes(buf[j*nbBytes : (j+1)*nbBytes])
		}
		k := string(buf)
		v, ok := histo[k]
		if !ok {
			return fmt.Errorf("query element not in table")
		}
		v++
		histo[k] = v
	}
	for i := 0; i < nbTable; i++ {
		for j := 0; j < nbRow; j++ {
			inputs[2+nbRow*i+j].FillBytes(buf[j*nbBytes : (j+1)*nbBytes])
		}
		outputs[i].Set(big.NewInt(histo[string(buf)]))
	}
	return nil
}
