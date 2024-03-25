// Package logderivprecomp allows computing functions using precomputation.
//
// Instead of computing binary functions and checking that the result is
// correctly constrained, we instead can precompute all valid values of a
// function and then perform lookup to obtain the result. For example, for the
// XOR function we would naively otherwise have to split the inputs into bits,
// XOR one-by-one and recombine.
//
// With this package, we can instead compute all results for two inputs of
// length 8 bit and then just perform a lookup on the inputs.
//
// We use the [logderivarg] package for the actual log-derivative argument.
package logderivprecomp

import (
	"fmt"
	"math/big"
	"reflect"

	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/internal/kvstore"
	"github.com/consensys/gnark/std/internal/logderivarg"
)

type ctxPrecomputedKey struct{ fn uintptr }

// Precomputed holds all precomputed function values and queries.
type Precomputed struct {
	api     frontend.API
	compute solver.Hint
	queries []frontend.Variable
	rets    []uint
}

// New returns a new [Precomputed]. It defers the log-derivative argument.
func New(api frontend.API, fn solver.Hint, rets []uint) (*Precomputed, error) {
	kv, ok := api.Compiler().(kvstore.Store)
	if !ok {
		panic("builder should implement key-value store")
	}
	ch := kv.GetKeyValue(ctxPrecomputedKey{fn: reflect.ValueOf(fn).Pointer()})
	if ch != nil {
		if prt, ok := ch.(*Precomputed); ok {
			return prt, nil
		} else {
			panic("stored rangechecker is not valid")
		}
	}
	// check that the output lengths fit into a single element
	var s uint = 16
	for _, v := range rets {
		s += v
	}
	if s >= uint(api.Compiler().FieldBitLen()) {
		return nil, fmt.Errorf("result doesn't fit into field element")
	}
	t := &Precomputed{
		api:     api,
		compute: fn,
		queries: nil,
		rets:    rets,
	}
	kv.SetKeyValue(ctxPrecomputedKey{fn: reflect.ValueOf(fn).Pointer()}, t)
	api.Compiler().Defer(t.build)
	return t, nil
}

func (t *Precomputed) pack(x, y frontend.Variable, rets []frontend.Variable) frontend.Variable {
	shift := big.NewInt(1 << 8)
	packed := t.api.Add(x, t.api.Mul(y, shift))
	for i := range t.rets {
		shift.Lsh(shift, t.rets[i])
		packed = t.api.Add(packed, t.api.Mul(rets[i], shift))
	}
	return packed
}

// Query
func (t *Precomputed) Query(x, y frontend.Variable) []frontend.Variable {
	// we don't have to check here. We assume the inputs are range checked and
	// range check the output.
	rets, err := t.api.Compiler().NewHint(t.compute, len(t.rets), x, y)
	if err != nil {
		panic(err)
	}
	packed := t.pack(x, y, rets)
	t.queries = append(t.queries, packed)
	return rets
}

func (t *Precomputed) buildTable() []frontend.Variable {
	tmp := new(big.Int)
	shift := new(big.Int)
	tbl := make([]frontend.Variable, 65536)
	inputs := []*big.Int{big.NewInt(0), big.NewInt(0)}
	outputs := make([]*big.Int, len(t.rets))
	for i := range outputs {
		outputs[i] = new(big.Int)
	}
	for x := int64(0); x < 256; x++ {
		inputs[0].SetInt64(x)
		for y := int64(0); y < 256; y++ {
			shift.SetInt64(1 << 8)
			i := x | (y << 8)
			inputs[1].SetInt64(y)
			if err := t.compute(t.api.Compiler().Field(), inputs, outputs); err != nil {
				panic(err)
			}
			tblval := new(big.Int).SetInt64(i)
			for j := range t.rets {
				shift.Lsh(shift, t.rets[j])
				tblval.Add(tblval, tmp.Mul(outputs[j], shift))
			}
			tbl[i] = tblval
		}
	}
	return tbl
}

func (t *Precomputed) build(api frontend.API) error {
	if len(t.queries) == 0 {
		return nil
	}
	table := t.buildTable()
	return logderivarg.Build(t.api, logderivarg.AsTable(table), logderivarg.AsTable(t.queries))
}
