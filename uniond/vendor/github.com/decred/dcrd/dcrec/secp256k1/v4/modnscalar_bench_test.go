// Copyright (c) 2020-2022 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package secp256k1

import (
	"math/big"
	"testing"
)

// benchmarkVals returns the raw bytes for a couple of unsigned 256-bit
// big-endian integers used throughout the benchmarks.
func benchmarkVals() [2][]byte {
	return [2][]byte{
		hexToBytes("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364143"),
		hexToBytes("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364144"),
	}
}

// BenchmarkBigIntModN benchmarks setting and reducing an unsigned 256-bit
// big-endian integer modulo the group order with stdlib big integers.
func BenchmarkBigIntModN(b *testing.B) {
	buf := benchmarkVals()[0]

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		v := new(big.Int).SetBytes(buf)
		v.Mod(v, curveParams.N)
	}
}

// BenchmarkModNScalar benchmarks setting and reducing an unsigned 256-bit
// big-endian integer modulo the group order with the specialized type.
func BenchmarkModNScalar(b *testing.B) {
	slice := benchmarkVals()[0]
	var buf [32]byte
	copy(buf[:], slice)

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		var s ModNScalar
		s.SetBytes(&buf)
	}
}

// BenchmarkBigIntZero benchmarks zeroing an unsigned 256-bit big-endian
// integer modulo the group order with stdlib big integers.
func BenchmarkBigIntZero(b *testing.B) {
	v1 := new(big.Int).SetBytes(benchmarkVals()[0])

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		v1.SetUint64(0)
	}
}

// BenchmarkModNScalarZero benchmarks zeroing an unsigned 256-bit big-endian
// integer modulo the group order with the specialized type.
func BenchmarkModNScalarZero(b *testing.B) {
	var s1 ModNScalar
	s1.SetByteSlice(benchmarkVals()[0])

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		s1.Zero()
	}
}

// BenchmarkBigIntIsZero benchmarks determining if an unsigned 256-bit
// big-endian integer modulo the group order is zero with stdlib big integers.
func BenchmarkBigIntIsZero(b *testing.B) {
	v1 := new(big.Int).SetBytes(benchmarkVals()[0])

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = v1.Sign() == 0
	}
}

// BenchmarkModNScalarIsZero benchmarks determining if an unsigned 256-bit
// big-endian integer modulo the group order is zero with the specialized type.
func BenchmarkModNScalarIsZero(b *testing.B) {
	var s1 ModNScalar
	s1.SetByteSlice(benchmarkVals()[0])

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = s1.IsZero()
	}
}

// BenchmarkBigIntEquals benchmarks determining equality between two unsigned
// 256-bit big-endian integers modulo the group order with stdlib big integers.
func BenchmarkBigIntEquals(b *testing.B) {
	bufs := benchmarkVals()
	v1 := new(big.Int).SetBytes(bufs[0])
	v2 := new(big.Int).SetBytes(bufs[1])

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		v1.Cmp(v2)
	}
}

// BenchmarkModNScalarEquals benchmarks determining equality between two
// unsigned 256-bit big-endian integers modulo the group order with the
// specialized type.
func BenchmarkModNScalarEquals(b *testing.B) {
	bufs := benchmarkVals()
	var s1, s2 ModNScalar
	s1.SetByteSlice(bufs[0])
	s2.SetByteSlice(bufs[1])

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		s1.Equals(&s2)
	}
}

// BenchmarkBigIntAddModN benchmarks adding two unsigned 256-bit big-endian
// integers modulo the group order with stdlib big integers.
func BenchmarkBigIntAddModN(b *testing.B) {
	bufs := benchmarkVals()
	v1 := new(big.Int).SetBytes(bufs[0])
	v2 := new(big.Int).SetBytes(bufs[1])

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		result := new(big.Int).Add(v1, v2)
		result.Mod(result, curveParams.N)
	}
}

// BenchmarkModNScalarAdd benchmarks adding two unsigned 256-bit big-endian
// integers modulo the group order with the specialized type.
func BenchmarkModNScalarAdd(b *testing.B) {
	bufs := benchmarkVals()
	var s1, s2 ModNScalar
	s1.SetByteSlice(bufs[0])
	s2.SetByteSlice(bufs[1])

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = new(ModNScalar).Add2(&s1, &s2)
	}
}

// BenchmarkBigIntMulModN benchmarks multiplying two unsigned 256-bit big-endian
// integers modulo the group order with stdlib big integers.
func BenchmarkBigIntMulModN(b *testing.B) {
	bufs := benchmarkVals()
	v1 := new(big.Int).SetBytes(bufs[0])
	v2 := new(big.Int).SetBytes(bufs[1])

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		result := new(big.Int).Mul(v1, v2)
		result.Mod(result, curveParams.N)
	}
}

// BenchmarkModNScalarMul benchmarks multiplying two unsigned 256-bit big-endian
// integers modulo the group order with the specialized type.
func BenchmarkModNScalarMul(b *testing.B) {
	bufs := benchmarkVals()
	var s1, s2 ModNScalar
	s1.SetByteSlice(bufs[0])
	s2.SetByteSlice(bufs[1])

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = new(ModNScalar).Mul2(&s1, &s2)
	}
}

// BenchmarkBigIntSquareModN benchmarks squaring an unsigned 256-bit big-endian
// integer modulo the group order is zero with stdlib big integers.
func BenchmarkBigIntSquareModN(b *testing.B) {
	v1 := new(big.Int).SetBytes(benchmarkVals()[0])

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		result := new(big.Int).Mul(v1, v1)
		result.Mod(result, curveParams.N)
	}
}

// BenchmarkModNScalarSquare benchmarks squaring an unsigned 256-bit big-endian
// integer modulo the group order is zero with the specialized type.
func BenchmarkModNScalarSquare(b *testing.B) {
	var s1 ModNScalar
	s1.SetByteSlice(benchmarkVals()[0])

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = new(ModNScalar).SquareVal(&s1)
	}
}

// BenchmarkBigIntNegateModN benchmarks negating an unsigned 256-bit big-endian
// integer modulo the group order is zero with stdlib big integers.
func BenchmarkBigIntNegateModN(b *testing.B) {
	v1 := new(big.Int).SetBytes(benchmarkVals()[0])

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		result := new(big.Int).Neg(v1)
		result.Mod(result, curveParams.N)
	}
}

// BenchmarkModNScalarNegate benchmarks negating an unsigned 256-bit big-endian
// integer modulo the group order is zero with the specialized type.
func BenchmarkModNScalarNegate(b *testing.B) {
	var s1 ModNScalar
	s1.SetByteSlice(benchmarkVals()[0])

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = new(ModNScalar).NegateVal(&s1)
	}
}

// BenchmarkBigIntInverseModN benchmarks calculating the multiplicative inverse
// of an unsigned 256-bit big-endian integer modulo the group order is zero with
// stdlib big integers.
func BenchmarkBigIntInverseModN(b *testing.B) {
	v1 := new(big.Int).SetBytes(benchmarkVals()[0])

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		new(big.Int).ModInverse(v1, curveParams.N)
	}
}

// BenchmarkModNScalarInverse benchmarks calculating the multiplicative inverse
// of an unsigned 256-bit big-endian integer modulo the group order is zero with
// the specialized type.
func BenchmarkModNScalarInverse(b *testing.B) {
	var s1 ModNScalar
	s1.SetByteSlice(benchmarkVals()[0])

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = new(ModNScalar).InverseValNonConst(&s1)
	}
}

// BenchmarkBigIntIsOverHalfOrder benchmarks determining if an unsigned 256-bit
// big-endian integer modulo the group order exceeds half the group order with
// stdlib big integers.
func BenchmarkBigIntIsOverHalfOrder(b *testing.B) {
	v1 := new(big.Int).SetBytes(benchmarkVals()[0])
	bigHalfOrder := new(big.Int).Rsh(curveParams.N, 1)

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = v1.Cmp(bigHalfOrder)
	}
}

// BenchmarkModNScalarIsOverHalfOrder benchmarks determining if an unsigned
// 256-bit big-endian integer modulo the group order exceeds half the group
// order with the specialized type.
func BenchmarkModNScalarIsOverHalfOrder(b *testing.B) {
	var s1 ModNScalar
	s1.SetByteSlice(benchmarkVals()[0])

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		s1.IsOverHalfOrder()
	}
}
