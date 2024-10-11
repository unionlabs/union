// uint256: Fixed size 256-bit math library
// Copyright 2020 uint256 Authors
// SPDX-License-Identifier: BSD-3-Clause

package uint256

import (
	"fmt"
	"math/big"
	"testing"
)

type opDualArgFunc func(*Int, *Int, *Int) *Int
type bigDualArgFunc func(*big.Int, *big.Int, *big.Int) *big.Int

type opCmpArgFunc func(*Int, *Int) bool
type bigCmpArgFunc func(*big.Int, *big.Int) bool

type binaryOpEntry struct {
	name   string
	u256Fn opDualArgFunc
	bigFn  bigDualArgFunc
}

func lookupBinary(name string) binaryOpEntry {
	for _, tc := range binaryOpFuncs {
		if tc.name == name {
			return tc
		}
	}
	panic(fmt.Sprintf("%v not found", name))
}

var binaryOpFuncs = []binaryOpEntry{
	{"Add", (*Int).Add, (*big.Int).Add},
	{"Sub", (*Int).Sub, (*big.Int).Sub},
	{"Mul", (*Int).Mul, (*big.Int).Mul},
	{"Div", (*Int).Div, bigDiv},
	{"Mod", (*Int).Mod, bigMod},
	{"SDiv", (*Int).SDiv, bigSDiv},
	{"SMod", (*Int).SMod, bigSMod},
	{"And", (*Int).And, (*big.Int).And},
	{"Or", (*Int).Or, (*big.Int).Or},
	{"Xor", (*Int).Xor, (*big.Int).Xor},
	{"Exp", (*Int).Exp, func(b1, b2, b3 *big.Int) *big.Int { return b1.Exp(b2, b3, bigtt256) }},
	{"Lsh", u256Lsh, bigLsh},
	{"Rsh", u256Rsh, bigRsh},
	{"SRsh", u256SRsh, bigSRsh},
	{"DivModDiv", divModDiv, bigDiv},
	{"DivModMod", divModMod, bigMod},
	{"udivremDiv", udivremDiv, bigDiv},
	{"udivremMod", udivremMod, bigMod},
	{"ExtendSign", (*Int).ExtendSign, bigExtendSign},
}

var cmpOpFuncs = []struct {
	name   string
	u256Fn opCmpArgFunc
	bigFn  bigCmpArgFunc
}{
	{"Eq", (*Int).Eq, func(a, b *big.Int) bool { return a.Cmp(b) == 0 }},
	{"Lt", (*Int).Lt, func(a, b *big.Int) bool { return a.Cmp(b) < 0 }},
	{"Gt", (*Int).Gt, func(a, b *big.Int) bool { return a.Cmp(b) > 0 }},
	{"Slt", (*Int).Slt, func(a, b *big.Int) bool { return bigS256(a).Cmp(bigS256(b)) < 0 }},
	{"Sgt", (*Int).Sgt, func(a, b *big.Int) bool { return bigS256(a).Cmp(bigS256(b)) > 0 }},
	{"CmpEq", func(a, b *Int) bool { return a.Cmp(b) == 0 }, func(a, b *big.Int) bool { return a.Cmp(b) == 0 }},
	{"CmpLt", func(a, b *Int) bool { return a.Cmp(b) < 0 }, func(a, b *big.Int) bool { return a.Cmp(b) < 0 }},
	{"CmpGt", func(a, b *Int) bool { return a.Cmp(b) > 0 }, func(a, b *big.Int) bool { return a.Cmp(b) > 0 }},
	{"LtUint64", func(a, b *Int) bool { return a.LtUint64(b.Uint64()) }, func(a, b *big.Int) bool { return a.Cmp(new(big.Int).SetUint64(b.Uint64())) < 0 }},
	{"GtUint64", func(a, b *Int) bool { return a.GtUint64(b.Uint64()) }, func(a, b *big.Int) bool { return a.Cmp(new(big.Int).SetUint64(b.Uint64())) > 0 }},
}

func checkBinaryOperation(t *testing.T, opName string, op opDualArgFunc, bigOp bigDualArgFunc, x, y Int) {
	var (
		b1        = x.ToBig()
		b2        = y.ToBig()
		f1        = x.Clone()
		f2        = y.Clone()
		operation = fmt.Sprintf("op: %v ( %v, %v ) ", opName, x.Hex(), y.Hex())
		want, _   = FromBig(bigOp(new(big.Int), b1, b2))
		have      = op(new(Int), f1, f2)
	)
	// Compare result with big.Int.
	if !have.Eq(want) {
		t.Fatalf("%v\nwant : %#x\nhave : %#x\n", operation, want, have)
	}

	// Check if arguments are unmodified.
	if !f1.Eq(x.Clone()) {
		t.Fatalf("%v\nfirst argument had been modified: %x", operation, f1)
	}
	if !f2.Eq(y.Clone()) {
		t.Fatalf("%v\nsecond argument had been modified: %x", operation, f2)
	}

	// Check if reusing args as result works correctly.
	have = op(f1, f1, y.Clone())
	if have != f1 {
		t.Fatalf("%v\nunexpected pointer returned: %p, expected: %p\n", operation, have, f1)
	}
	if !have.Eq(want) {
		t.Fatalf("%v\non argument reuse x.op(x,y)\nwant : %#x\nhave : %#x\n", operation, want, have)
	}
	have = op(f2, x.Clone(), f2)
	if have != f2 {
		t.Fatalf("%v\nunexpected pointer returned: %p, expected: %p\n", operation, have, f2)
	}
	if !have.Eq(want) {
		t.Fatalf("%v\n on argument reuse x.op(y,x)\nwant : %#x\nhave : %#x\n", operation, want, have)
	}
}

func TestBinaryOperations(t *testing.T) {
	for _, tc := range binaryOpFuncs {
		for _, inputs := range binTestCases {
			f1 := MustFromHex(inputs[0])
			f2 := MustFromHex(inputs[1])
			checkBinaryOperation(t, tc.name, tc.u256Fn, tc.bigFn, *f1, *f2)
		}
	}
}

func Test10KRandomBinaryOperations(t *testing.T) {
	for _, tc := range binaryOpFuncs {
		for i := 0; i < 10000; i++ {
			f1 := randNum()
			f2 := randNum()
			checkBinaryOperation(t, tc.name, tc.u256Fn, tc.bigFn, *f1, *f2)
		}
	}
}

func FuzzBinaryOperations(f *testing.F) {
	f.Fuzz(func(t *testing.T, x0, x1, x2, x3, y0, y1, y2, y3 uint64) {
		x := Int{x0, x1, x2, x3}
		y := Int{y0, y1, y2, y3}
		for _, tc := range binaryOpFuncs {
			checkBinaryOperation(t, tc.name, tc.u256Fn, tc.bigFn, x, y)
		}
	})
}

func u256Rsh(z, x, y *Int) *Int {
	return z.Rsh(x, uint(y.Uint64()&0x1FF))
}
func bigRsh(z, x, y *big.Int) *big.Int {
	return z.Rsh(x, uint(y.Uint64()&0x1FF))
}

func u256Lsh(z, x, y *Int) *Int {
	return z.Lsh(x, uint(y.Uint64()&0x1FF))
}
func u256SRsh(z, x, y *Int) *Int {
	return z.SRsh(x, uint(y.Uint64()&0x1FF))
}

func bigLsh(z, x, y *big.Int) *big.Int {
	return z.Lsh(x, uint(y.Uint64()&0x1FF))
}

func bigSRsh(z, x, y *big.Int) *big.Int {
	return z.Rsh(bigS256(x), uint(y.Uint64()&0x1FF))
}

func bigExtendSign(result, num, byteNum *big.Int) *big.Int {
	if byteNum.Cmp(big.NewInt(31)) >= 0 {
		return result.Set(num)
	}
	bit := uint(byteNum.Uint64()*8 + 7)
	mask := byteNum.Lsh(big.NewInt(1), bit)
	mask.Sub(mask, big.NewInt(1))
	if num.Bit(int(bit)) > 0 {
		result.Or(num, mask.Not(mask))
	} else {
		result.And(num, mask)
	}
	return result
}

// bigDiv implements uint256/EVM compatible division for big.Int: returns 0 when dividing by 0
func bigDiv(z, x, y *big.Int) *big.Int {
	if y.Sign() == 0 {
		return z.SetUint64(0)
	}
	return z.Div(x, y)
}

// bigMod implements uint256/EVM compatible mod for big.Int: returns 0 when dividing by 0
func bigMod(z, x, y *big.Int) *big.Int {
	if y.Sign() == 0 {
		return z.SetUint64(0)
	}
	return z.Mod(x, y)
}

// bigSDiv implements EVM-compatible SDIV operation on big.Int
func bigSDiv(result, x, y *big.Int) *big.Int {
	if y.Sign() == 0 {
		return result.SetUint64(0)
	}
	sx := bigS256(x)
	sy := bigS256(y)

	n := new(big.Int)
	if sx.Sign() == sy.Sign() {
		n.SetInt64(1)
	} else {
		n.SetInt64(-1)
	}
	result.Div(sx.Abs(sx), sy.Abs(sy))
	result.Mul(result, n)
	return result
}

// bigSMod implements EVM-compatible SMOD operation on big.Int
func bigSMod(result, x, y *big.Int) *big.Int {
	if y.Sign() == 0 {
		return result.SetUint64(0)
	}

	sx := bigS256(x)
	sy := bigS256(y)
	neg := sx.Sign() < 0

	result.Mod(sx.Abs(sx), sy.Abs(sy))
	if neg {
		result.Neg(result)
	}
	return bigU256(result)
}

// divModDiv wraps DivMod and returns quotient only
func divModDiv(z, x, y *Int) *Int {
	var m Int
	z.DivMod(x, y, &m)
	return z
}

// divModMod wraps DivMod and returns modulus only
func divModMod(z, x, y *Int) *Int {
	new(Int).DivMod(x, y, z)
	return z
}

// udivremDiv wraps udivrem and returns quotient
func udivremDiv(z, x, y *Int) *Int {
	var quot Int
	if !y.IsZero() {
		udivrem(quot[:], x[:], y, nil)
	}
	return z.Set(&quot)
}

// udivremMod wraps udivrem and returns remainder
func udivremMod(z, x, y *Int) *Int {
	if y.IsZero() {
		return z.Clear()
	}
	var quot, rem Int
	udivrem(quot[:], x[:], y, &rem)
	return z.Set(&rem)
}

func checkCompareOperation(t *testing.T, opName string, op opCmpArgFunc, bigOp bigCmpArgFunc, x, y Int) {
	var (
		f1orig    = x.Clone()
		f2orig    = y.Clone()
		b1        = x.ToBig()
		b2        = y.ToBig()
		f1        = new(Int).Set(f1orig)
		f2        = new(Int).Set(f2orig)
		operation = fmt.Sprintf("op: %v ( %v, %v ) ", opName, x.Hex(), y.Hex())
		want      = bigOp(b1, b2)
		have      = op(f1, f2)
	)
	// Compare result with big.Int.
	if have != want {
		t.Fatalf("%v\nwant : %v\nhave : %v\n", operation, want, have)
	}
	// Check if arguments are unmodified.
	if !f1.Eq(f1orig) {
		t.Fatalf("%v\nfirst argument had been modified: %x", operation, f1)
	}
	if !f2.Eq(f2orig) {
		t.Fatalf("%v\nsecond argument had been modified: %x", operation, f2)
	}
}

func TestCompareOperations(t *testing.T) {
	for _, tc := range cmpOpFuncs {
		for _, inputs := range binTestCases {
			f1 := MustFromHex(inputs[0])
			f2 := MustFromHex(inputs[1])
			checkCompareOperation(t, tc.name, tc.u256Fn, tc.bigFn, *f1, *f2)
		}
	}
}

func FuzzCompareOperations(f *testing.F) {
	f.Fuzz(func(t *testing.T, x0, x1, x2, x3, y0, y1, y2, y3 uint64) {
		x := Int{x0, x1, x2, x3}
		y := Int{y0, y1, y2, y3}
		for _, tc := range cmpOpFuncs {
			checkCompareOperation(t, tc.name, tc.u256Fn, tc.bigFn, x, y)
		}
	})
}
