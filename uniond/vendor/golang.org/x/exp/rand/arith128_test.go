// Copyright 2017 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package rand

import (
	"math/big"
	"math/rand"
	"testing"
)

var bigMaxUint64 = big.NewInt(0).SetUint64(maxUint64)

func bigInt(xHi, xLo uint64) *big.Int {
	b := big.NewInt(0).SetUint64(xHi)
	b.Lsh(b, 64)
	b.Or(b, big.NewInt(0).SetUint64(xLo))
	return b
}

func splitBigInt(b *big.Int) (outHi, outLo uint64) {
	outHi = big.NewInt(0).Rsh(b, 64).Uint64()
	outLo = big.NewInt(0).And(b, bigMaxUint64).Uint64()
	return
}

func bigMulMod128bits(xHi, xLo, yHi, yLo uint64) (outHi, outLo uint64) {
	bigX := bigInt(xHi, xLo)
	bigY := bigInt(yHi, yLo)
	return splitBigInt(bigX.Mul(bigX, bigY))
}

func bigAddMod128bits(xHi, xLo, yHi, yLo uint64) (outHi, outLo uint64) {
	bigX := bigInt(xHi, xLo)
	bigY := bigInt(yHi, yLo)
	return splitBigInt(bigX.Add(bigX, bigY))
}

type arithTest struct {
	xHi, xLo uint64
}

const (
	iLo = increment & maxUint64
	iHi = (increment >> 64) & maxUint64
)

var arithTests = []arithTest{
	{0, 0},
	{0, 1},
	{1, 0},
	{0, maxUint64},
	{maxUint64, 0},
	{maxUint64, maxUint64},
	// Randomly generated 64-bit integers.
	{3757956613005209672, 17983933746665545631},
	{511324141977587414, 5626651684620191081},
	{1534313104606153588, 2415006486399353367},
	{6873586429837825902, 13854394671140464137},
	{6617134480561088940, 18421520694158684312},
}

func TestPCGAdd(t *testing.T) {
	for i, test := range arithTests {
		p := &PCGSource{
			low:  test.xLo,
			high: test.xHi,
		}
		p.add()
		expectHi, expectLo := bigAddMod128bits(test.xHi, test.xLo, iHi, iLo)
		if p.low != expectLo || p.high != expectHi {
			t.Errorf("%d: got hi=%d lo=%d; expect hi=%d lo=%d", i, p.high, p.low, expectHi, expectLo)
		}
	}
}

const (
	mLo = multiplier & maxUint64
	mHi = (multiplier >> 64) & maxUint64
)

func TestPCGMultiply(t *testing.T) {
	for i, test := range arithTests {
		p := &PCGSource{
			low:  test.xLo,
			high: test.xHi,
		}
		p.multiply()
		expectHi, expectLo := bigMulMod128bits(test.xHi, test.xLo, mHi, mLo)
		if p.low != expectLo || p.high != expectHi {
			t.Errorf("%d: got hi=%d lo=%d; expect hi=%d lo=%d", i, p.high, p.low, expectHi, expectLo)
		}
	}
}

func TestPCGMultiplyLong(t *testing.T) {
	if testing.Short() {
		return
	}
	for i := 0; i < 1e6; i++ {
		low := rand.Uint64()
		high := rand.Uint64()
		p := &PCGSource{
			low:  low,
			high: high,
		}
		p.multiply()
		expectHi, expectLo := bigMulMod128bits(high, low, mHi, mLo)
		if p.low != expectLo || p.high != expectHi {
			t.Fatalf("%d: (%d,%d): got hi=%d lo=%d; expect hi=%d lo=%d", i, high, low, p.high, p.low, expectHi, expectLo)
		}
	}
}

func BenchmarkPCGMultiply(b *testing.B) {
	low := rand.Uint64()
	high := rand.Uint64()
	p := &PCGSource{
		low:  low,
		high: high,
	}
	for i := 0; i < b.N; i++ {
		p.multiply()
	}
}
