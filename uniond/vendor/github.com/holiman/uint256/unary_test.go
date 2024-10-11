// uint256: Fixed size 256-bit math library
// Copyright 2020 uint256 Authors
// SPDX-License-Identifier: BSD-3-Clause

package uint256

import (
	"fmt"
	"math/big"
	"testing"
)

type opUnaryArgFunc func(*Int, *Int) *Int
type bigUnaryArgFunc func(*big.Int, *big.Int) *big.Int

var unaryOpFuncs = []struct {
	name   string
	u256Fn opUnaryArgFunc
	bigFn  bigUnaryArgFunc
}{
	{"Not", (*Int).Not, (*big.Int).Not},
	{"Neg", (*Int).Neg, (*big.Int).Neg},
	{"Sqrt", (*Int).Sqrt, (*big.Int).Sqrt},
	{"square", func(x *Int, y *Int) *Int {
		res := y.Clone()
		res.squared()
		return x.Set(res)
	}, func(b1, b2 *big.Int) *big.Int { return b1.Mul(b2, b2) }},
	{"Abs", (*Int).Abs, func(b1, b2 *big.Int) *big.Int { return b1.Abs(bigS256(b2)) }},
}

func checkUnaryOperation(t *testing.T, opName string, op opUnaryArgFunc, bigOp bigUnaryArgFunc, x Int) {
	var (
		b1        = x.ToBig()
		f1        = x.Clone()
		operation = fmt.Sprintf("op: %v ( %v ) ", opName, x.Hex())
		want, _   = FromBig(bigOp(new(big.Int), b1))
		have      = op(new(Int), f1)
	)
	// Compare result with big.Int.
	if !have.Eq(want) {
		t.Fatalf("%v\nwant : %#x\nhave : %#x\n", operation, want, have)
	}
	// Check if arguments are unmodified.
	if !f1.Eq(x.Clone()) {
		t.Fatalf("%v\nfirst argument had been modified: %x", operation, f1)
	}
	// Check if reusing args as result works correctly.
	have = op(f1, f1)
	if have != f1 {
		t.Fatalf("%v\nunexpected pointer returned: %p, expected: %p\n", operation, have, f1)
	}
	if !have.Eq(want) {
		t.Fatalf("%v\n on argument reuse x.op(x)\nwant : %#x\nhave : %#x\n", operation, want, have)
	}
}

func TestUnaryOperations(t *testing.T) {
	for _, tc := range unaryOpFuncs {
		for _, arg := range unTestCases {
			f1 := MustFromHex(arg)
			checkUnaryOperation(t, tc.name, tc.u256Fn, tc.bigFn, *f1)
		}
	}
}

func FuzzUnaryOperations(f *testing.F) {
	f.Fuzz(func(t *testing.T, x0, x1, x2, x3 uint64) {
		x := Int{x0, x1, x2, x3}
		for _, tc := range unaryOpFuncs {
			checkUnaryOperation(t, tc.name, tc.u256Fn, tc.bigFn, x)
		}
	})
}
