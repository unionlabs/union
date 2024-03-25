package bitslice

import (
	"math/big"

	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/rangecheck"
)

// Partition partitions v into two parts splitted at bit numbered split. The
// following holds
//
//	v = lower + 2^split * upper.
//
// The method enforces that lower < 2^split and upper < 2^split', where
// split'=nbScalar-split. When giving the option [WithNbDigits], we instead use
// the bound split'=nbDigits-split.
func Partition(api frontend.API, v frontend.Variable, split uint, opts ...Option) (lower, upper frontend.Variable) {
	opt, err := parseOpts(opts...)
	if err != nil {
		panic(err)
	}
	// handle constant case
	if vc, ok := api.Compiler().ConstantValue(v); ok {
		if opt.digits > 0 && vc.BitLen() > opt.digits {
			panic("input larger than bound")
		}
		if split == 0 {
			return 0, vc
		}
		div := new(big.Int).Lsh(big.NewInt(1), split)
		l, u := new(big.Int), new(big.Int)
		u.QuoRem(vc, div, l)
		return l, u
	}
	rh := rangecheck.New(api)
	if split == 0 {
		if opt.digits > 0 {
			rh.Check(v, opt.digits)
		}
		return 0, v
	}
	ret, err := api.Compiler().NewHint(partitionHint, 2, split, v)
	if err != nil {
		panic(err)
	}

	upper = ret[0]
	lower = ret[1]

	if opt.nocheck {
		if opt.digits > 0 {
			rh.Check(v, opt.digits)
		}
		return
	}
	upperBound := api.Compiler().FieldBitLen()
	if opt.digits > 0 {
		upperBound = opt.digits
	}
	rh.Check(upper, upperBound)
	rh.Check(lower, int(split))

	m := big.NewInt(1)
	m.Lsh(m, split)
	composed := api.Add(lower, api.Mul(upper, m))
	api.AssertIsEqual(composed, v)
	return
}
