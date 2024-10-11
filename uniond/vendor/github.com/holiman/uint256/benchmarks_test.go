// uint256: Fixed size 256-bit math library
// Copyright 2020 uint256 Authors
// SPDX-License-Identifier: BSD-3-Clause

package uint256

import (
	"math/big"
	"math/rand"
	"testing"
)

const numSamples = 1024

var (
	int32Samples    [numSamples]Int
	int32SamplesLt  [numSamples]Int
	int64Samples    [numSamples]Int
	int64SamplesLt  [numSamples]Int
	int128Samples   [numSamples]Int
	int128SamplesLt [numSamples]Int
	int192Samples   [numSamples]Int
	int192SamplesLt [numSamples]Int
	int256Samples   [numSamples]Int
	int256SamplesLt [numSamples]Int // int256SamplesLt[i] <= int256Samples[i]

	big32Samples    [numSamples]big.Int
	big32SamplesLt  [numSamples]big.Int
	big64Samples    [numSamples]big.Int
	big64SamplesLt  [numSamples]big.Int
	big128Samples   [numSamples]big.Int
	big128SamplesLt [numSamples]big.Int
	big192Samples   [numSamples]big.Int
	big192SamplesLt [numSamples]big.Int
	big256Samples   [numSamples]big.Int
	big256SamplesLt [numSamples]big.Int // big256SamplesLt[i] <= big256Samples[i]

	_ = initSamples()
)

func initSamples() bool {
	rnd := rand.New(rand.NewSource(0))

	// newRandInt creates new Int with so many highly likely non-zero random words.
	newRandInt := func(numWords int) Int {
		var z Int
		for i := 0; i < numWords; i++ {
			z[i] = rnd.Uint64()
		}
		return z
	}

	for i := 0; i < numSamples; i++ {
		x32g := rnd.Uint32()
		x32l := rnd.Uint32()
		if x32g < x32l {
			x32g, x32l = x32l, x32g
		}
		int32Samples[i].SetUint64(uint64(x32g))
		big32Samples[i] = *int32Samples[i].ToBig()
		int32SamplesLt[i].SetUint64(uint64(x32l))
		big32SamplesLt[i] = *int32SamplesLt[i].ToBig()

		l := newRandInt(1)
		g := newRandInt(1)
		if g.Lt(&l) {
			g, l = l, g
		}
		if g[0] == 0 {
			g[0]++
		}
		int64Samples[i] = g
		big64Samples[i] = *int64Samples[i].ToBig()
		int64SamplesLt[i] = l
		big64SamplesLt[i] = *int64SamplesLt[i].ToBig()

		l = newRandInt(2)
		g = newRandInt(2)
		if g.Lt(&l) {
			g, l = l, g
		}
		if g[1] == 0 {
			g[1]++
		}
		int128Samples[i] = g
		big128Samples[i] = *int128Samples[i].ToBig()
		int128SamplesLt[i] = l
		big128SamplesLt[i] = *int128SamplesLt[i].ToBig()

		l = newRandInt(3)
		g = newRandInt(3)
		if g.Lt(&l) {
			g, l = l, g
		}
		if g[2] == 0 {
			g[2]++
		}
		int192Samples[i] = g
		big192Samples[i] = *int192Samples[i].ToBig()
		int192SamplesLt[i] = l
		big192SamplesLt[i] = *int192SamplesLt[i].ToBig()

		l = newRandInt(4)
		g = newRandInt(4)
		if g.Lt(&l) {
			g, l = l, g
		}
		if g[3] == 0 {
			g[3]++
		}
		int256Samples[i] = g
		big256Samples[i] = *int256Samples[i].ToBig()
		int256SamplesLt[i] = l
		big256SamplesLt[i] = *int256SamplesLt[i].ToBig()
	}

	return true
}

func benchmark_Add_Bit(bench *testing.B) {
	b1 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	b2 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdefaaaaaa9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	f, _ := FromBig(b1)
	f2, _ := FromBig(b2)
	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		f.Add(f, f2)
	}
}
func benchmark_Add_Big(bench *testing.B) {
	b := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	b2 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdefaaaaaa9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		b.Add(b, b2)
	}
}
func BenchmarkAdd(bench *testing.B) {
	bench.Run("single/big", benchmark_Add_Big)
	bench.Run("single/uint256", benchmark_Add_Bit)
}

func benchmark_SubOverflow_Bit(bench *testing.B) {
	b1 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	b2 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdefaaaaaa9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	f, _ := FromBig(b1)
	f2, _ := FromBig(b2)

	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		f.SubOverflow(f, f2)
	}
}
func benchmark_Sub_Bit(bench *testing.B) {
	b1 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	b2 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdefaaaaaa9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	f, _ := FromBig(b1)
	f2, _ := FromBig(b2)

	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		f.Sub(f, f2)
	}
}

func benchmark_Sub_Big(bench *testing.B) {
	b1 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	b2 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdefaaaaaa9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))

	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		b1.Sub(b1, b2)
	}
}
func BenchmarkSub(bench *testing.B) {
	bench.Run("single/big", benchmark_Sub_Big)
	bench.Run("single/uint256", benchmark_Sub_Bit)
	bench.Run("single/uint256_of", benchmark_SubOverflow_Bit)
}

func BenchmarkMul(bench *testing.B) {
	benchmarkUint256 := func(bench *testing.B) {
		a := big.NewInt(0).SetBytes(hex2Bytes("f123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
		b := big.NewInt(0).SetBytes(hex2Bytes("f123456789abcdefaaaaaa9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
		fa, _ := FromBig(a)
		fb, _ := FromBig(b)

		result := new(Int)
		bench.ResetTimer()
		for i := 0; i < bench.N; i++ {
			result.Mul(fa, fb)
		}
	}
	benchmarkBig := func(bench *testing.B) {
		a := new(big.Int).SetBytes(hex2Bytes("f123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
		b := new(big.Int).SetBytes(hex2Bytes("f123456789abcdefaaaaaa9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))

		result := new(big.Int)
		bench.ResetTimer()
		for i := 0; i < bench.N; i++ {
			bigU256(result.Mul(a, b))
		}
	}

	bench.Run("single/uint256", benchmarkUint256)
	bench.Run("single/big", benchmarkBig)
}

func BenchmarkMulOverflow(bench *testing.B) {
	benchmarkUint256 := func(bench *testing.B) {
		a := big.NewInt(0).SetBytes(hex2Bytes("f123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
		b := big.NewInt(0).SetBytes(hex2Bytes("f123456789abcdefaaaaaa9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
		fa, _ := FromBig(a)
		fb, _ := FromBig(b)

		result := new(Int)
		bench.ResetTimer()
		for i := 0; i < bench.N; i++ {
			result.MulOverflow(fa, fb)
		}
	}
	benchmarkBig := func(bench *testing.B) {
		a := new(big.Int).SetBytes(hex2Bytes("f123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
		b := new(big.Int).SetBytes(hex2Bytes("f123456789abcdefaaaaaa9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))

		result := new(big.Int)
		bench.ResetTimer()
		for i := 0; i < bench.N; i++ {
			bigU256(result.Mul(a, b))
		}
	}

	bench.Run("single/uint256", benchmarkUint256)
	bench.Run("single/big", benchmarkBig)
}

func BenchmarkSquare(bench *testing.B) {

	benchmarkUint256 := func(bench *testing.B) {
		a := new(Int).SetBytes(hex2Bytes("f123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))

		result := new(Int)
		bench.ResetTimer()
		for i := 0; i < bench.N; i++ {
			result.Set(a).squared()
		}
	}
	benchmarkBig := func(bench *testing.B) {
		a := new(big.Int).SetBytes(hex2Bytes("f123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))

		result := new(big.Int)
		bench.ResetTimer()
		for i := 0; i < bench.N; i++ {
			bigU256(result.Mul(a, a))
		}
	}

	bench.Run("single/uint256", benchmarkUint256)
	bench.Run("single/big", benchmarkBig)
}

func BenchmarkSqrt(bench *testing.B) {
	benchmarkUint256 := func(bench *testing.B) {
		bench.ReportAllocs()
		a := new(Int).SetBytes(hex2Bytes("f123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))

		result := new(Int)
		bench.ResetTimer()
		for i := 0; i < bench.N; i++ {
			result.Sqrt(a)
		}
	}
	benchmarkBig := func(bench *testing.B) {
		bench.ReportAllocs()
		a := new(big.Int).SetBytes(hex2Bytes("f123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))

		result := new(big.Int)
		bench.ResetTimer()
		for i := 0; i < bench.N; i++ {
			result.Sqrt(a)
		}
	}
	bench.Run("single/uint256", benchmarkUint256)
	bench.Run("single/big", benchmarkBig)
}

func benchmark_And_Big(bench *testing.B) {
	b1 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	b2 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdefaaaaaa9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		b1.And(b1, b2)
	}
}
func benchmark_And_Bit(bench *testing.B) {
	b1 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	b2 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdefaaaaaa9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	f, _ := FromBig(b1)
	f2, _ := FromBig(b2)
	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		f.And(f, f2)
	}
}
func BenchmarkAnd(bench *testing.B) {
	bench.Run("single/big", benchmark_And_Big)
	bench.Run("single/uint256", benchmark_And_Bit)
}

func benchmark_Or_Big(bench *testing.B) {
	b1 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	b2 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdefaaaaaa9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		b1.Or(b1, b2)
	}
}
func benchmark_Or_Bit(bench *testing.B) {
	b1 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	b2 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdefaaaaaa9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	f, _ := FromBig(b1)
	f2, _ := FromBig(b2)
	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		f.Or(f, f2)
	}
}
func BenchmarkOr(bench *testing.B) {
	bench.Run("single/big", benchmark_Or_Big)
	bench.Run("single/uint256", benchmark_Or_Bit)
}

func benchmark_Xor_Big(bench *testing.B) {
	b1 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	b2 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdefaaaaaa9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		b1.Xor(b1, b2)
	}
}
func benchmark_Xor_Bit(bench *testing.B) {
	b1 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	b2 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdefaaaaaa9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	f, _ := FromBig(b1)
	f2, _ := FromBig(b2)
	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		f.Xor(f, f2)
	}
}

func BenchmarkXor(bench *testing.B) {
	bench.Run("single/big", benchmark_Xor_Big)
	bench.Run("single/uint256", benchmark_Xor_Bit)
}

func benchmark_Cmp_Big(bench *testing.B) {
	b1 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	b2 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdefaaaaaa9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		b1.Cmp(b2)
		b2.Cmp(b1)
	}
}
func benchmark_Cmp_Bit(bench *testing.B) {
	b1 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	b2 := big.NewInt(0).SetBytes(hex2Bytes("0123456789abcdefaaaaaa9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
	f, _ := FromBig(b1)
	f2, _ := FromBig(b2)

	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		f.Cmp(f2)
		f2.Cmp(f)
	}
}

func BenchmarkCmp(bench *testing.B) {
	bench.Run("single/big", benchmark_Cmp_Big)
	bench.Run("single/uint256", benchmark_Cmp_Bit)
}

func BenchmarkCmpBig(bench *testing.B) {
	// This bench tests where the Cmp is non-equal
	bench.Run("nonequal", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			x := &big256Samples[i%len(big256Samples)]
			z := &int256Samples[len(int256Samples)-(i%len(int256Samples))-1]
			_ = z.CmpBig(x)
		}
	})
	bench.Run("equal", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			x := &big256Samples[i%len(big256Samples)]
			z := &int256Samples[i%len(big256Samples)]
			if z.CmpBig(x) != 0 {
				b.Fatal("test error")
			}
		}
	})
}

func BenchmarkLt(b *testing.B) {
	benchmarkUint256 := func(b *testing.B, samples *[numSamples]Int) (flag bool) {
		var x Int
		for j := 0; j < b.N; j += numSamples {
			for i := 0; i < numSamples; i++ {
				y := samples[i]
				flag = x.Lt(&y)
				x = y
			}
		}
		return
	}
	benchmarkBig := func(b *testing.B, samples *[numSamples]big.Int) (flag bool) {
		var x big.Int
		for j := 0; j < b.N; j += numSamples {
			for i := 0; i < numSamples; i++ {
				y := samples[i]
				flag = x.Cmp(&y) < 0
				x = y
			}
		}
		return
	}

	b.Run("large/uint256", func(b *testing.B) { benchmarkUint256(b, &int256Samples) })
	b.Run("large/big", func(b *testing.B) { benchmarkBig(b, &big256Samples) })
	b.Run("small/uint256", func(b *testing.B) { benchmarkUint256(b, &int64Samples) })
	b.Run("small/big", func(b *testing.B) { benchmarkBig(b, &big64Samples) })
}

func benchmark_Lsh_Big(n uint, bench *testing.B) {
	original := big.NewInt(0).SetBytes(hex2Bytes("FBCDEF090807060504030201ffffffffFBCDEF090807060504030201ffffffff"))
	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		b1 := big.NewInt(0)
		b1.Lsh(original, n)
	}
}
func benchmark_Lsh_Big_N_EQ_0(bench *testing.B) {
	benchmark_Lsh_Big(0, bench)
}
func benchmark_Lsh_Big_N_GT_192(bench *testing.B) {
	benchmark_Lsh_Big(193, bench)
}
func benchmark_Lsh_Big_N_GT_128(bench *testing.B) {
	benchmark_Lsh_Big(129, bench)
}
func benchmark_Lsh_Big_N_GT_64(bench *testing.B) {
	benchmark_Lsh_Big(65, bench)
}
func benchmark_Lsh_Big_N_GT_0(bench *testing.B) {
	benchmark_Lsh_Big(1, bench)
}
func benchmark_Lsh_Bit(n uint, bench *testing.B) {
	original := big.NewInt(0).SetBytes(hex2Bytes("FBCDEF090807060504030201ffffffffFBCDEF090807060504030201ffffffff"))
	f2, _ := FromBig(original)
	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		f1 := new(Int)
		f1.Lsh(f2, n)
	}
}
func benchmark_Lsh_Bit_N_EQ_0(bench *testing.B) {
	benchmark_Lsh_Bit(0, bench)
}
func benchmark_Lsh_Bit_N_GT_192(bench *testing.B) {
	benchmark_Lsh_Bit(193, bench)
}
func benchmark_Lsh_Bit_N_GT_128(bench *testing.B) {
	benchmark_Lsh_Bit(129, bench)
}
func benchmark_Lsh_Bit_N_GT_64(bench *testing.B) {
	benchmark_Lsh_Bit(65, bench)
}
func benchmark_Lsh_Bit_N_GT_0(bench *testing.B) {
	benchmark_Lsh_Bit(1, bench)
}
func BenchmarkLsh(bench *testing.B) {
	bench.Run("n_eq_0/big", benchmark_Lsh_Big_N_EQ_0)
	bench.Run("n_gt_192/big", benchmark_Lsh_Big_N_GT_192)
	bench.Run("n_gt_128/big", benchmark_Lsh_Big_N_GT_128)
	bench.Run("n_gt_64/big", benchmark_Lsh_Big_N_GT_64)
	bench.Run("n_gt_0/big", benchmark_Lsh_Big_N_GT_0)

	bench.Run("n_eq_0/uint256", benchmark_Lsh_Bit_N_EQ_0)
	bench.Run("n_gt_192/uint256", benchmark_Lsh_Bit_N_GT_192)
	bench.Run("n_gt_128/uint256", benchmark_Lsh_Bit_N_GT_128)
	bench.Run("n_gt_64/uint256", benchmark_Lsh_Bit_N_GT_64)
	bench.Run("n_gt_0/uint256", benchmark_Lsh_Bit_N_GT_0)
}

func benchmark_Rsh_Big(n uint, bench *testing.B) {
	original := big.NewInt(0).SetBytes(hex2Bytes("FBCDEF090807060504030201ffffffffFBCDEF090807060504030201ffffffff"))
	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		b1 := big.NewInt(0)
		b1.Rsh(original, n)
	}
}
func benchmark_Rsh_Big_N_EQ_0(bench *testing.B) {
	benchmark_Rsh_Big(0, bench)
}
func benchmark_Rsh_Big_N_GT_192(bench *testing.B) {
	benchmark_Rsh_Big(193, bench)
}
func benchmark_Rsh_Big_N_GT_128(bench *testing.B) {
	benchmark_Rsh_Big(129, bench)
}
func benchmark_Rsh_Big_N_GT_64(bench *testing.B) {
	benchmark_Rsh_Big(65, bench)
}
func benchmark_Rsh_Big_N_GT_0(bench *testing.B) {
	benchmark_Rsh_Big(1, bench)
}
func benchmark_Rsh_Bit(n uint, bench *testing.B) {
	original := big.NewInt(0).SetBytes(hex2Bytes("FBCDEF090807060504030201ffffffffFBCDEF090807060504030201ffffffff"))
	f2, _ := FromBig(original)
	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		f1 := new(Int)
		f1.Rsh(f2, n)
	}
}
func benchmark_Rsh_Bit_N_EQ_0(bench *testing.B) {
	benchmark_Rsh_Bit(0, bench)
}
func benchmark_Rsh_Bit_N_GT_192(bench *testing.B) {
	benchmark_Rsh_Bit(193, bench)
}
func benchmark_Rsh_Bit_N_GT_128(bench *testing.B) {
	benchmark_Rsh_Bit(129, bench)
}
func benchmark_Rsh_Bit_N_GT_64(bench *testing.B) {
	benchmark_Rsh_Bit(65, bench)
}
func benchmark_Rsh_Bit_N_GT_0(bench *testing.B) {
	benchmark_Rsh_Bit(1, bench)
}
func BenchmarkRsh(bench *testing.B) {
	bench.Run("n_eq_0/big", benchmark_Rsh_Big_N_EQ_0)
	bench.Run("n_gt_192/big", benchmark_Rsh_Big_N_GT_192)
	bench.Run("n_gt_128/big", benchmark_Rsh_Big_N_GT_128)
	bench.Run("n_gt_64/big", benchmark_Rsh_Big_N_GT_64)
	bench.Run("n_gt_0/big", benchmark_Rsh_Big_N_GT_0)

	bench.Run("n_eq_0/uint256", benchmark_Rsh_Bit_N_EQ_0)
	bench.Run("n_gt_192/uint256", benchmark_Rsh_Bit_N_GT_192)
	bench.Run("n_gt_128/uint256", benchmark_Rsh_Bit_N_GT_128)
	bench.Run("n_gt_64/uint256", benchmark_Rsh_Bit_N_GT_64)
	bench.Run("n_gt_0/uint256", benchmark_Rsh_Bit_N_GT_0)
}

// bigExp implements exponentiation by squaring.
// The result is truncated to 256 bits.
func bigExp(result, base, exponent *big.Int) *big.Int {
	result.SetUint64(1)

	for _, word := range exponent.Bits() {
		for i := 0; i < wordBits; i++ {
			if word&1 == 1 {
				bigU256(result.Mul(result, base))
			}
			bigU256(base.Mul(base, base))
			word >>= 1
		}
	}
	return result
}

func benchmark_Exp_Big(bench *testing.B) {
	x := "ABCDEF090807060504030201ffffffffffffffffffffffffffffffffffffffff"
	y := "ABCDEF090807060504030201ffffffffffffffffffffffffffffffffffffffff"

	orig := big.NewInt(0).SetBytes(hex2Bytes(x))
	base := big.NewInt(0).SetBytes(hex2Bytes(x))
	exp := big.NewInt(0).SetBytes(hex2Bytes(y))

	result := new(big.Int)
	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		bigExp(result, base, exp)
		base.Set(orig)
	}
}
func benchmark_Exp_Bit(bench *testing.B) {
	x := "ABCDEF090807060504030201ffffffffffffffffffffffffffffffffffffffff"
	y := "ABCDEF090807060504030201ffffffffffffffffffffffffffffffffffffffff"

	base := big.NewInt(0).SetBytes(hex2Bytes(x))
	exp := big.NewInt(0).SetBytes(hex2Bytes(y))

	f_base, _ := FromBig(base)
	f_orig, _ := FromBig(base)
	f_exp, _ := FromBig(exp)
	f_res := Int{}

	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		f_res.Exp(f_base, f_exp)
		f_base.Set(f_orig)
	}
}
func benchmark_ExpSmall_Big(bench *testing.B) {
	x := "ABCDEF090807060504030201ffffffffffffffffffffffffffffffffffffffff"
	y := "8abcdef"

	orig := big.NewInt(0).SetBytes(hex2Bytes(x))
	base := big.NewInt(0).SetBytes(hex2Bytes(x))
	exp := big.NewInt(0).SetBytes(hex2Bytes(y))

	result := new(big.Int)
	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		bigExp(result, base, exp)
		base.Set(orig)
	}
}
func benchmark_ExpSmall_Bit(bench *testing.B) {
	x := "ABCDEF090807060504030201ffffffffffffffffffffffffffffffffffffffff"
	y := "8abcdef"

	base := big.NewInt(0).SetBytes(hex2Bytes(x))
	exp := big.NewInt(0).SetBytes(hex2Bytes(y))

	f_base, _ := FromBig(base)
	f_orig, _ := FromBig(base)
	f_exp, _ := FromBig(exp)
	f_res := Int{}

	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		f_res.Exp(f_base, f_exp)
		f_base.Set(f_orig)
	}
}
func BenchmarkExp(bench *testing.B) {
	bench.Run("large/big", benchmark_Exp_Big)
	bench.Run("large/uint256", benchmark_Exp_Bit)
	bench.Run("small/big", benchmark_ExpSmall_Big)
	bench.Run("small/uint256", benchmark_ExpSmall_Bit)
}

func BenchmarkDiv(b *testing.B) {
	benchmarkDivUint256 := func(b *testing.B, xSamples, modSamples *[numSamples]Int) {
		var sink Int
		for j := 0; j < b.N; j += numSamples {
			for i := 0; i < numSamples; i++ {
				sink.Div(&xSamples[i], &modSamples[i])
			}
		}
	}
	benchmarkDivBig := func(b *testing.B, xSamples, modSamples *[numSamples]big.Int) {
		var sink big.Int
		for j := 0; j < b.N; j += numSamples {
			for i := 0; i < numSamples; i++ {
				sink.Div(&xSamples[i], &modSamples[i])
			}
		}
	}

	b.Run("small/uint256", func(b *testing.B) { benchmarkDivUint256(b, &int32Samples, &int32SamplesLt) })
	b.Run("mod64/uint256", func(b *testing.B) { benchmarkDivUint256(b, &int256Samples, &int64Samples) })
	b.Run("mod128/uint256", func(b *testing.B) { benchmarkDivUint256(b, &int256Samples, &int128Samples) })
	b.Run("mod192/uint256", func(b *testing.B) { benchmarkDivUint256(b, &int256Samples, &int192Samples) })
	b.Run("mod256/uint256", func(b *testing.B) { benchmarkDivUint256(b, &int256Samples, &int256SamplesLt) })
	b.Run("small/big", func(b *testing.B) { benchmarkDivBig(b, &big32Samples, &big32SamplesLt) })
	b.Run("mod64/big", func(b *testing.B) { benchmarkDivBig(b, &big256Samples, &big64Samples) })
	b.Run("mod128/big", func(b *testing.B) { benchmarkDivBig(b, &big256Samples, &big128Samples) })
	b.Run("mod192/big", func(b *testing.B) { benchmarkDivBig(b, &big256Samples, &big192Samples) })
	b.Run("mod256/big", func(b *testing.B) { benchmarkDivBig(b, &big256Samples, &big256SamplesLt) })
}

func BenchmarkMod(b *testing.B) {
	benchmarkModUint256 := func(b *testing.B, xSamples, modSamples *[numSamples]Int) {
		var sink Int
		for j := 0; j < b.N; j += numSamples {
			for i := 0; i < numSamples; i++ {
				sink.Mod(&xSamples[i], &modSamples[i])
			}
		}
	}
	benchmarkModBig := func(b *testing.B, xSamples, modSamples *[numSamples]big.Int) {
		var sink big.Int
		for j := 0; j < b.N; j += numSamples {
			for i := 0; i < numSamples; i++ {
				sink.Mod(&xSamples[i], &modSamples[i])
			}
		}
	}

	b.Run("small/uint256", func(b *testing.B) { benchmarkModUint256(b, &int32Samples, &int32SamplesLt) })
	b.Run("mod64/uint256", func(b *testing.B) { benchmarkModUint256(b, &int256Samples, &int64Samples) })
	b.Run("mod128/uint256", func(b *testing.B) { benchmarkModUint256(b, &int256Samples, &int128Samples) })
	b.Run("mod192/uint256", func(b *testing.B) { benchmarkModUint256(b, &int256Samples, &int192Samples) })
	b.Run("mod256/uint256", func(b *testing.B) { benchmarkModUint256(b, &int256Samples, &int256SamplesLt) })
	b.Run("small/big", func(b *testing.B) { benchmarkModBig(b, &big32Samples, &big32SamplesLt) })
	b.Run("mod64/big", func(b *testing.B) { benchmarkModBig(b, &big256Samples, &big64Samples) })
	b.Run("mod128/big", func(b *testing.B) { benchmarkModBig(b, &big256Samples, &big128Samples) })
	b.Run("mod192/big", func(b *testing.B) { benchmarkModBig(b, &big256Samples, &big192Samples) })
	b.Run("mod256/big", func(b *testing.B) { benchmarkModBig(b, &big256Samples, &big256SamplesLt) })
}

func BenchmarkDivMod(b *testing.B) {
	benchmarkDivModUint256 := func(b *testing.B, xSamples, modSamples *[numSamples]Int) {
		var sink, mod Int
		for j := 0; j < b.N; j += numSamples {
			for i := 0; i < numSamples; i++ {
				sink.DivMod(&xSamples[i], &modSamples[i], &mod)
			}
		}
	}
	benchmarkDivModBig := func(b *testing.B, xSamples, modSamples *[numSamples]big.Int) {
		var sink, mod big.Int
		for j := 0; j < b.N; j += numSamples {
			for i := 0; i < numSamples; i++ {
				sink.DivMod(&xSamples[i], &modSamples[i], &mod)
			}
		}
	}

	b.Run("small/uint256", func(b *testing.B) { benchmarkDivModUint256(b, &int32Samples, &int32SamplesLt) })
	b.Run("mod64/uint256", func(b *testing.B) { benchmarkDivModUint256(b, &int256Samples, &int64Samples) })
	b.Run("mod128/uint256", func(b *testing.B) { benchmarkDivModUint256(b, &int256Samples, &int128Samples) })
	b.Run("mod192/uint256", func(b *testing.B) { benchmarkDivModUint256(b, &int256Samples, &int192Samples) })
	b.Run("mod256/uint256", func(b *testing.B) { benchmarkDivModUint256(b, &int256Samples, &int256SamplesLt) })
	b.Run("small/big", func(b *testing.B) { benchmarkDivModBig(b, &big32Samples, &big32SamplesLt) })
	b.Run("mod64/big", func(b *testing.B) { benchmarkDivModBig(b, &big256Samples, &big64Samples) })
	b.Run("mod128/big", func(b *testing.B) { benchmarkDivModBig(b, &big256Samples, &big128Samples) })
	b.Run("mod192/big", func(b *testing.B) { benchmarkDivModBig(b, &big256Samples, &big192Samples) })
	b.Run("mod256/big", func(b *testing.B) { benchmarkDivModBig(b, &big256Samples, &big256SamplesLt) })
}

func BenchmarkAddMod(b *testing.B) {
	benchmarkAddModUint256 := func(b *testing.B, factorsSamples, modSamples *[numSamples]Int) {
		iter := (b.N + numSamples - 1) / numSamples

		for j := 0; j < numSamples; j++ {
			var x Int
			y := factorsSamples[j]

			for i := 0; i < iter; i++ {
				x.AddMod(&x, &y, &modSamples[j])
			}
		}
	}
	benchmarkAddModBig := func(b *testing.B, factorsSamples, modSamples *[numSamples]big.Int) {
		iter := (b.N + numSamples - 1) / numSamples

		for j := 0; j < numSamples; j++ {
			var x big.Int
			y := factorsSamples[j]

			for i := 0; i < iter; i++ {
				x.Add(&x, &y)
				x.Mod(&x, &modSamples[j])
			}
		}
	}

	b.Run("small/uint256", func(b *testing.B) { benchmarkAddModUint256(b, &int32SamplesLt, &int32Samples) })
	b.Run("mod64/uint256", func(b *testing.B) { benchmarkAddModUint256(b, &int64SamplesLt, &int64Samples) })
	b.Run("mod128/uint256", func(b *testing.B) { benchmarkAddModUint256(b, &int128SamplesLt, &int128Samples) })
	b.Run("mod192/uint256", func(b *testing.B) { benchmarkAddModUint256(b, &int192SamplesLt, &int192Samples) })
	b.Run("mod256/uint256", func(b *testing.B) { benchmarkAddModUint256(b, &int256SamplesLt, &int256Samples) })
	b.Run("small/big", func(b *testing.B) { benchmarkAddModBig(b, &big32SamplesLt, &big32Samples) })
	b.Run("mod64/big", func(b *testing.B) { benchmarkAddModBig(b, &big64SamplesLt, &big64Samples) })
	b.Run("mod128/big", func(b *testing.B) { benchmarkAddModBig(b, &big128SamplesLt, &big128Samples) })
	b.Run("mod192/big", func(b *testing.B) { benchmarkAddModBig(b, &big192SamplesLt, &big192Samples) })
	b.Run("mod256/big", func(b *testing.B) { benchmarkAddModBig(b, &big256SamplesLt, &big256Samples) })
}

func BenchmarkMulMod(b *testing.B) {
	benchmarkMulModUint256R := func(b *testing.B, factorsSamples, modSamples *[numSamples]Int) {
		iter := (b.N + numSamples - 1) / numSamples

		var mu [numSamples][5]uint64

		for i := 0; i < numSamples; i++ {
			mu[i] = Reciprocal(&modSamples[i])
		}

		b.ResetTimer()

		for j := 0; j < numSamples; j++ {
			x := factorsSamples[j]

			for i := 0; i < iter; i++ {
				x.MulModWithReciprocal(&x, &factorsSamples[j], &modSamples[j], &mu[j])
			}
		}
	}
	benchmarkMulModUint256 := func(b *testing.B, factorsSamples, modSamples *[numSamples]Int) {
		iter := (b.N + numSamples - 1) / numSamples

		for j := 0; j < numSamples; j++ {
			x := factorsSamples[j]

			for i := 0; i < iter; i++ {
				x.MulMod(&x, &factorsSamples[j], &modSamples[j])
			}
		}
	}
	benchmarkMulModBig := func(b *testing.B, factorsSamples, modSamples *[numSamples]big.Int) {
		iter := (b.N + numSamples - 1) / numSamples

		for j := 0; j < numSamples; j++ {
			x := factorsSamples[j]

			for i := 0; i < iter; i++ {
				x.Mul(&x, &factorsSamples[j])
				x.Mod(&x, &modSamples[j])
			}
		}
	}

	b.Run("small/uint256", func(b *testing.B) { benchmarkMulModUint256(b, &int32SamplesLt, &int32Samples) })
	b.Run("mod64/uint256", func(b *testing.B) { benchmarkMulModUint256(b, &int64SamplesLt, &int64Samples) })
	b.Run("mod128/uint256", func(b *testing.B) { benchmarkMulModUint256(b, &int128SamplesLt, &int128Samples) })
	b.Run("mod192/uint256", func(b *testing.B) { benchmarkMulModUint256(b, &int192SamplesLt, &int192Samples) })
	b.Run("mod256/uint256", func(b *testing.B) { benchmarkMulModUint256(b, &int256SamplesLt, &int256Samples) })
	b.Run("mod256/uint256r", func(b *testing.B) { benchmarkMulModUint256R(b, &int256SamplesLt, &int256Samples) })
	b.Run("small/big", func(b *testing.B) { benchmarkMulModBig(b, &big32SamplesLt, &big32Samples) })
	b.Run("mod64/big", func(b *testing.B) { benchmarkMulModBig(b, &big64SamplesLt, &big64Samples) })
	b.Run("mod128/big", func(b *testing.B) { benchmarkMulModBig(b, &big128SamplesLt, &big128Samples) })
	b.Run("mod192/big", func(b *testing.B) { benchmarkMulModBig(b, &big192SamplesLt, &big192Samples) })
	b.Run("mod256/big", func(b *testing.B) { benchmarkMulModBig(b, &big256SamplesLt, &big256Samples) })
}

func benchmark_SdivLarge_Big(bench *testing.B) {
	a := new(big.Int).SetBytes(hex2Bytes("800fffffffffffffffffffffffffd1e870eec79504c60144cc7f5fc2bad1e611"))
	b := new(big.Int).SetBytes(hex2Bytes("ff3f9014f20db29ae04af2c2d265de17"))

	bench.ResetTimer()

	for i := 0; i < bench.N; i++ {
		bigU256(bigSDiv(new(big.Int), a, b))
	}
}

func benchmark_SdivLarge_Bit(bench *testing.B) {
	a := big.NewInt(0).SetBytes(hex2Bytes("800fffffffffffffffffffffffffd1e870eec79504c60144cc7f5fc2bad1e611"))
	b := big.NewInt(0).SetBytes(hex2Bytes("ff3f9014f20db29ae04af2c2d265de17"))
	fa, _ := FromBig(a)
	fb, _ := FromBig(b)

	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		f := new(Int)
		f.SDiv(fa, fb)
	}
}

func BenchmarkSDiv(bench *testing.B) {
	bench.Run("large/big", benchmark_SdivLarge_Big)
	bench.Run("large/uint256", benchmark_SdivLarge_Bit)
}

func BenchmarkEncodeHex(b *testing.B) {
	hexEncodeU256 := func(b *testing.B, samples *[numSamples]Int) {
		b.ReportAllocs()
		for j := 0; j < b.N; j += numSamples {
			for i := 0; i < numSamples; i++ {
				samples[i].Hex()
			}
		}
	}
	hexEncodeBig := func(b *testing.B, samples *[numSamples]big.Int) {
		b.ReportAllocs()
		for j := 0; j < b.N; j += numSamples {
			for i := 0; i < numSamples; i++ {
				// We're being nice to big.Int here, because this method
				// does not add the 0x-prefix -- so an extra alloc is needed to get
				// the same result. We still win the benchmark though...
				samples[i].Text(16)
			}
		}
	}
	b.Run("large/uint256", func(b *testing.B) { hexEncodeU256(b, &int256Samples) })
	b.Run("large/big", func(b *testing.B) { hexEncodeBig(b, &big256Samples) })
}

func BenchmarkFromHexString(b *testing.B) {
	var hexStrings []string
	for _, z := range &int256Samples {
		hexStrings = append(hexStrings, (&z).Hex())
	}
	b.Run("uint256", func(b *testing.B) {
		b.ReportAllocs()
		for j := 0; j < b.N; j++ {
			i := j % numSamples
			if _, err := FromHex(hexStrings[i]); err != nil {
				b.Fatalf("%v: %v", err, string(hexStrings[i]))
			}
		}
	})
	b.Run("big", func(b *testing.B) {
		b.ReportAllocs()
		for j := 0; j < b.N; j++ {
			i := j % numSamples
			if _, ok := big.NewInt(0).SetString(hexStrings[i], 0); !ok {
				b.Fatalf("Error on %v", string(hexStrings[i]))
			}
		}
	})
}

func BenchmarkMulDivOverflow(b *testing.B) {
	benchmarkUint256 := func(b *testing.B, factorsSamples, muldivSamples *[numSamples]Int) {
		iter := (b.N + numSamples - 1) / numSamples

		for j := 0; j < numSamples; j++ {
			x := factorsSamples[j]

			for i := 0; i < iter; i++ {
				res := new(Int)
				res.MulDivOverflow(&x, &x, &muldivSamples[j])
			}
		}
	}

	benchmarkBig := func(b *testing.B, factorsSamples, muldivSamples *[numSamples]big.Int) {
		iter := (b.N + numSamples - 1) / numSamples

		for j := 0; j < numSamples; j++ {
			x := factorsSamples[j]

			for i := 0; i < iter; i++ {
				res := new(big.Int)
				res.Mul(&x, &x)
				res.Div(res, &muldivSamples[j])
			}
		}
	}

	b.Run("small/uint256", func(b *testing.B) { benchmarkUint256(b, &int32SamplesLt, &int32Samples) })
	b.Run("div64/uint256", func(b *testing.B) { benchmarkUint256(b, &int64SamplesLt, &int64Samples) })
	b.Run("div128/uint256", func(b *testing.B) { benchmarkUint256(b, &int128SamplesLt, &int128Samples) })
	b.Run("div192/uint256", func(b *testing.B) { benchmarkUint256(b, &int192SamplesLt, &int192Samples) })
	b.Run("div256/uint256", func(b *testing.B) { benchmarkUint256(b, &int256SamplesLt, &int256Samples) })
	b.Run("small/big", func(b *testing.B) { benchmarkBig(b, &big32SamplesLt, &big32Samples) })
	b.Run("div64/big", func(b *testing.B) { benchmarkBig(b, &big64SamplesLt, &big64Samples) })
	b.Run("div128/big", func(b *testing.B) { benchmarkBig(b, &big128SamplesLt, &big128Samples) })
	b.Run("div192/big", func(b *testing.B) { benchmarkBig(b, &big192SamplesLt, &big192Samples) })
	b.Run("div256/big", func(b *testing.B) { benchmarkBig(b, &big256SamplesLt, &big256Samples) })

}

func BenchmarkHashTreeRoot(b *testing.B) {
	var (
		z   = &Int{1, 2, 3, 4}
		exp = [32]byte{1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0}
	)
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		r, err := z.HashTreeRoot()
		if err != nil {
			b.Fatal(err)
		}
		if r != exp {
			b.Fatalf("have %v want %v", r, exp)
		}
	}
}

func BenchmarkSet(bench *testing.B) {
	benchmarkUint256 := func(bench *testing.B) {
		a := new(Int).SetBytes(hex2Bytes("f123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))

		result := new(Int)
		bench.ResetTimer()
		for i := 0; i < bench.N; i++ {
			result.Set(a)
		}
	}
	benchmarkBig := func(bench *testing.B) {
		a := new(big.Int).SetBytes(hex2Bytes("f123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))

		result := new(big.Int)
		bench.ResetTimer()
		for i := 0; i < bench.N; i++ {
			result.Set(a)
		}
	}

	bench.Run("single/uint256", benchmarkUint256)
	bench.Run("single/big", benchmarkBig)
}

func BenchmarkByte(bench *testing.B) {
	var (
		a      = new(Int).SetBytes(hex2Bytes("f123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9"))
		num    = NewInt(0)
		result = new(Int)
	)
	bench.ResetTimer()
	for i := 0; i < bench.N; i++ {
		for ii := uint64(0); ii < 32; ii++ {
			result.Set(a)
			num.SetUint64(ii)
			_ = result.Byte(num)
		}
	}
}

func BenchmarkExtendSign(b *testing.B) {
	a, err := FromHex("0xf123456789abcdeffedcba9876543210f2f3f4f5f6f7f8f9fff3f4f5f6f7f8f9")
	if err != nil {
		b.Fatal(err)
	}
	result := new(Int)
	n := NewInt(0)
	b.ResetTimer()
	for i := uint64(0); i < uint64(b.N); i++ {
		n.SetUint64(i % 33)
		result.ExtendSign(a, n)
	}
}
