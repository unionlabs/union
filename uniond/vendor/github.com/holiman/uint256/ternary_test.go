// uint256: Fixed size 256-bit math library
// Copyright 2020 uint256 Authors
// SPDX-License-Identifier: BSD-3-Clause

package uint256

import (
	"fmt"
	"math/big"
	"testing"
)

type opThreeArgFunc func(*Int, *Int, *Int, *Int) *Int
type bigThreeArgFunc func(*big.Int, *big.Int, *big.Int, *big.Int) *big.Int

var ternaryOpFuncs = []struct {
	name   string
	u256Fn opThreeArgFunc
	bigFn  bigThreeArgFunc
}{
	{"AddMod", (*Int).AddMod, bigAddMod},
	{"MulMod", (*Int).MulMod, bigMulMod},
	{"MulModWithReciprocal", (*Int).mulModWithReciprocalWrapper, bigMulMod},
	{"DivModZ", divModZ, bigDivModZ},
	{"DivModM", divModM, bigDivModM},
}

func checkTernaryOperation(t *testing.T, opName string, op opThreeArgFunc, bigOp bigThreeArgFunc, x, y, z Int) {
	var (
		f1orig    = x.Clone()
		f2orig    = y.Clone()
		f3orig    = z.Clone()
		b1        = x.ToBig()
		b2        = y.ToBig()
		b3        = z.ToBig()
		f1        = new(Int).Set(f1orig)
		f2        = new(Int).Set(f2orig)
		f3        = new(Int).Set(f3orig)
		operation = fmt.Sprintf("op: %v ( %v, %v, %v ) ", opName, x.Hex(), y.Hex(), z.Hex())
		want, _   = FromBig(bigOp(new(big.Int), b1, b2, b3))
		have      = op(new(Int), f1, f2, f3)
	)
	if !have.Eq(want) {
		t.Fatalf("%v\nwant : %#x\nhave : %#x\n", operation, want, have)
	}
	// Check if arguments are unmodified.
	if !f1.Eq(f1orig) {
		t.Fatalf("%v\nfirst argument had been modified: %x", operation, f1)
	}
	if !f2.Eq(f2orig) {
		t.Fatalf("%v\nsecond argument had been modified: %x", operation, f2)
	}
	if !f3.Eq(f3orig) {
		if opName != "DivModZ" && opName != "DivModM" {
			// DivMod takes m as third argument, modifies it, and returns it. That is by design.
			t.Fatalf("%v\nthird argument had been modified: %x", operation, f3)
		}
	}
	// Check if reusing args as result works correctly.
	if have = op(f1, f1, f2orig, f3orig); have != f1 {
		t.Fatalf("%v\nunexpected pointer returned: %p, expected: %p\n", operation, have, f1)
	} else if !have.Eq(want) {
		t.Fatalf("%v\non argument reuse x.op(x,y,z)\nwant : %#x\nhave : %#x\n", operation, want, have)
	}

	if have = op(f2, f1orig, f2, f3orig); have != f2 {
		t.Fatalf("%v\nunexpected pointer returned: %p, expected: %p\n", operation, have, f2)
	} else if !have.Eq(want) {
		t.Fatalf("%v\non argument reuse y.op(x,y,z)\nwant : %#x\nhave : %#x\n", operation, want, have)
	}

	if have = op(f3, f1orig, f2orig, f3); have != f3 {
		t.Fatalf("%v\nunexpected pointer returned: %p, expected: %p\n", operation, have, f3)
	} else if !have.Eq(want) {
		t.Fatalf("%v\non argument reuse z.op(x,y,z)\nwant : %#x\nhave : %#x\n", operation, want, have)
	}
}

func TestTernaryOperations(t *testing.T) {
	for _, tc := range ternaryOpFuncs {
		for _, inputs := range ternTestCases {
			f1 := MustFromHex(inputs[0])
			f2 := MustFromHex(inputs[1])
			f3 := MustFromHex(inputs[2])
			t.Run(tc.name, func(t *testing.T) {
				checkTernaryOperation(t, tc.name, tc.u256Fn, tc.bigFn, *f1, *f2, *f3)
			})
		}
	}
}

func FuzzTernaryOperations(f *testing.F) {
	f.Fuzz(func(t *testing.T,
		x0, x1, x2, x3,
		y0, y1, y2, y3,
		z0, z1, z2, z3 uint64) {

		x := Int{x0, x1, x2, x3}
		y := Int{y0, y1, y2, y3}
		z := Int{z0, z1, z2, z3}
		for _, tc := range ternaryOpFuncs {
			checkTernaryOperation(t, tc.name, tc.u256Fn, tc.bigFn, x, y, z)
		}
	})
}

func bigAddMod(result, x, y, mod *big.Int) *big.Int {
	if mod.Sign() == 0 {
		return result.SetUint64(0)
	}
	return result.Mod(result.Add(x, y), mod)
}

func bigMulMod(result, x, y, mod *big.Int) *big.Int {
	if mod.Sign() == 0 {
		return result.SetUint64(0)
	}
	return result.Mod(result.Mul(x, y), mod)
}

func (z *Int) mulModWithReciprocalWrapper(x, y, mod *Int) *Int {
	mu := Reciprocal(mod)
	return z.MulModWithReciprocal(x, y, mod, &mu)
}

func divModZ(z, x, y, m *Int) *Int {
	z2, _ := z.DivMod(x, y, m)
	return z2
}

func bigDivModZ(result, x, y, mod *big.Int) *big.Int {
	if y.Sign() == 0 {
		return result.SetUint64(0)
	}
	z2, _ := result.DivMod(x, y, mod)
	return z2
}

func divModM(z, x, y, m *Int) *Int {
	_, m2 := z.DivMod(x, y, m)
	return z.Set(m2)
}

func bigDivModM(result, x, y, mod *big.Int) *big.Int {
	if y.Sign() == 0 {
		return result.SetUint64(0)
	}
	_, m2 := result.DivMod(x, y, mod)
	return result.Set(m2)
}
