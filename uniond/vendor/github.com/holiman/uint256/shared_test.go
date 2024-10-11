// uint256: Fixed size 256-bit math library
// Copyright 2018-2020 uint256 Authors
// SPDX-License-Identifier: BSD-3-Clause

package uint256

import (
	"crypto/rand"
	"math/big"
)

// This file contains some utilities which the oss-fuzz fuzzer shares with the
// regular go-native tests. It is placed in a separate file, becaue the oss-fuzz
// clang-based fuzzing infrastructure requires some instrumentation.
// During this instrumentation, the file under test (e.g. unary_test.go) is modified,
// and the same modification needs to be performed with any other files that
// it requires (this file).

var (
	bigtt256   = new(big.Int).Lsh(big.NewInt(1), 256)
	bigtt255   = new(big.Int).Lsh(big.NewInt(1), 255)
	bigtt256m1 = new(big.Int).Sub(bigtt256, big.NewInt(1))

	unTestCases = []string{
		"0x0",
		"0x1",
		"0x8000000000000000",
		"0x12cbafcee8f60f9f",
		"0x80000000000000000000000000000000",
		"0x80000000000000010000000000000000",
		"0x80000000000000000000000000000001",
		"0x12cbafcee8f60f9f3fa308c90fde8d298772ffea667aa6bc109d5c661e7929a5",
		"0x8000000000000000000000000000000000000000000000000000000000000000",
		"0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe",
		"0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
	}

	// A collection of interesting input values for binary operators (especially for division).
	// No expected results as big.Int can be used as the source of truth.
	binTestCases = [][2]string{
		{"0x0", "0x0"},
		{"0x1", "0x0"},
		{"0x1", "0x767676767676767676000000767676767676"},
		{"0x2", "0x0"},
		{"0x2", "0x1"},
		{"0x12cbafcee8f60f9f3fa308c90fde8d298772ffea667aa6bc109d5c661e7929a5", "0xc76f4afb041407a8ea478d65024f5c3dfe1db1a1bb10c5ea8bec314ccf9"},
		{"0x10000000000000000", "0x2"},
		{"0x7000000000000000", "0x8000000000000000"},
		{"0x8000000000000000", "0x8000000000000000"},
		{"0x8000000000000001", "0x8000000000000000"},
		{"0x80000000000000010000000000000000", "0x80000000000000000000000000000000"},
		{"0x80000000000000000000000000000000", "0x80000000000000000000000000000001"},
		{"0x478392145435897052", "0x111"},
		{"0x767676767676767676000000767676767676", "0x2900760076761e00020076760000000076767676000000"},
		{"0x12121212121212121212121212121212", "0x232323232323232323"},
		{"0xfffff716b61616160b0b0b2b0b0b0becf4bef50a0df4f48b090b2b0bc60a0a00", "0xfffff716b61616160b0b0b2b0b230b000008010d0a2b00"},
		{"0x50beb1c60141a0000dc2b0b0b0b0b0b410a0a0df4f40b090b2b0bc60a0a00", "0x2000110000000d0a300e750a000000090a0a"},
		{"0x4b00000b41000b0b0b2b0b0b0b0b0b410a0aeff4f40b090b2b0bc60a0a1000", "0x4b00000b41000b0b0b2b0b0b0b0b0b410a0aeff4f40b0a0a"},
		{"0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee", "0x7"},
		{"0xf6376770abd3a36b20394c5664afef1194c801c3f05e42566f085ed24d002bb0", "0xb368d219438b7f3f"},
		{"0x0", "0x10900000000000000000000000000000000000000000000000000"},
		{"0x77676767676760000000000000001002e000000000000040000000e000000000", "0xfffc000000000000767676240000000000002b0576047"},
		{"0x767676767676000000000076000000000000005600000000000000000000", "0x767676767676000000000076000000760000"},
		{"0x8200000000000000000000000000000000000000000000000000000000000000", "0x8200000000000000fe000004000000ffff000000fffff700"},
		{"0xdac7fff9ffd9e1322626262626262600", "0xd021262626262626"},
		{"0x8000000000000001800000000000000080000000000000008000000000000000", "0x800000000000000080000000000000008000000000000000"},
		{"0xe8e8e8e2000100000009ea02000000000000ff3ffffff80000001000220000", "0xe8e8e8e2000100000009ea02000000000000ff3ffffff800000010002280ff"},
		{"0xc9700000000000000000023f00c00014ff000000000000000022300805", "0xc9700000000000000000023f00c00014ff002c000000000000223108"},
		{"0x40000000fd000000db0000000000000000000000000000000000000000000001", "0x40000000fd000000db0000000000000000000040000000fd000000db000001"},
		{"0x40000000fd000000db0000000000000000000000000000000000000000000001", "0x40000000fd000000db0000000000000000000040000000fd000000db0000d3"},
		{"0x1f000000000000000000000000000000200000000100000000000000000000", "0x100000000ffffffffffffffff0000000000002e000000"},
		{"0x7effffff80000000000000000000000000020000440000000000000000000001", "0x7effffff800000007effffff800000008000ff0000010000"},
		{"0x5fd8fffffffffffffffffffffffffffffc090000ce700004d0c9ffffff000001", "0x2ffffffffffffffffffffffffffffffffff000000030000"},
		{"0x62d8fffffffffffffffffffffffffffffc18000000000000000000ca00000001", "0x2ffffffffffffffffffffffffffffffffff200000000000"},
		{"0x7effffff8000000000000000000000000000000000000000d900000000000001", "0x7effffff8000000000000000000000000000000000008001"},
		{"0x6400aff20ff00200004e7fd1eff08ffca0afd1eff08ffca0a", "0x210000000000000022"},
		{"0x6d5adef08547abf7eb", "0x13590cab83b779e708b533b0eef3561483ddeefc841f5"},
		{"0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"},
		{"0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"},
		{"0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"},
		{"0xe8e8e8e2000100000009ea02000000000000ff3ffffff80000001000220000", "0xffffffffffffffff7effffff800000007effffff800000008000ff0000010000"},
		{"0x1ce97e1ab91a", "0x66aa0a5319bcf5cb4"}, // regression test for udivrem() where len(x) < len(y)
	}

	// A collection of interesting input values for ternary operators (addmod, mulmod).
	ternTestCases = [][3]string{
		{"0x0", "0x0", "0x0"},
		{"0x1", "0x0", "0x0"},
		{"0x1", "0x1", "0x0"},
		{"0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd", "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe", "0x0"},
		{"0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd", "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"},
		{"0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd", "0x3", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"},
		{"0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"},
		{"0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe"},
		{"0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0x2"},
		{"0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0x1"},
		{"0xffffffffffffffffffffffffffffffff", "0xffffffffffffffffffffffffffffffff", "0xfffffffffffffffffffffffffffffffe00000000000000000000000000000002"},
		{"0xffffffffffffffffffffffffffffffff", "0xffffffffffffffffffffffffffffffff", "0xfffffffffffffffffffffffffffffffe00000000000000000000000000000001"},
		{"0xffffffffffffffffffffffffffff000004020041fffffffffc00000060000020", "0xffffffffffffffffffffffffffffffe6000000ffffffe60000febebeffffffff", "0xffffffffffffffffffe6000000ffffffe60000febebeffffffffffffffffffff"},
		{"0xffffffffffffffffffffffffffffffff00ffffe6ff0000000000000060000020", "0xffffffffffffffffffffffffffffffffffe6000000ffff00e60000febebeffff", "0xffffffffffffffffffe6000000ffff00e60000fe0000ffff00e60000febebeff"},
		{"0xfffffffffffffffffffffffff600000000005af50100bebe000000004a00be0a", "0xffffffffffffffffffffffffffffeaffdfd9fffffffffffff5f60000000000ff", "0xffffffffffffffffffffffeaffdfd9fffffffffffffff60000000000ffffffff"},
		{"0x8000000000000001000000000000000000000000000000000000000000000000", "0x800000000000000100000000000000000000000000000000000000000000000b", "0x8000000000000000000000000000000000000000000000000000000000000000"},
		{"0x8000000000000000000000000000000000000000000000000000000000000000", "0x8000000000000001000000000000000000000000000000000000000000000000", "0x8000000000000000000000000000000000000000000000000000000000000000"},
		{"0x8000000000000000000000000000000000000000000000000000000000000000", "0x8000000000000001000000000000000000000000000000000000000000000000", "0x8000000000000001000000000000000000000000000000000000000000000000"},
		{"0x8000000000000000000000000000000000000000000000000000000000000000", "0x8000000000000000000000000000000100000000000000000000000000000000", "0x8000000000000000000000000000000000000000000000000000000000000001"},
		{"0x1", "0x1", "0xffffffff00000001000000000000000000000000ffffffffffffffffffffffff"},
		{"0x1", "0x1", "0x1000000003030303030303030303030303030303030303030303030303030"},
		{"0x1", "0x1", "0x4000000000000000130303030303030303030303030303030303030303030"},
		{"0x1", "0x1", "0x8000000000000000000000000000000043030303000000000"},
		{"0x1", "0x1", "0x8000000000000000000000000000000003030303030303030"},
	}
)

// bigU256 encodes as a 256 bit two's complement number. This operation is destructive.
func bigU256(x *big.Int) *big.Int {
	return x.And(x, bigtt256m1)
}

func bigS256(x *big.Int) *big.Int {
	if x.Cmp(bigtt255) < 0 {
		return x
	}
	return new(big.Int).Sub(x, bigtt256)
}

func randNum() *Int {
	//How many bits? 0-256
	nbits, _ := rand.Int(rand.Reader, big.NewInt(257))
	//Max random value, a 130-bits integer, i.e 2^130
	max := new(big.Int)
	max.Exp(big.NewInt(2), big.NewInt(nbits.Int64()), nil)
	b, _ := rand.Int(rand.Reader, max)
	f, _ := FromBig(b)
	return f
}
