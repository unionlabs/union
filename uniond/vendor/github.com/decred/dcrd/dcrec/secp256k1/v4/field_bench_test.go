// Copyright (c) 2020-2023 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package secp256k1

import (
	"math/big"
	"testing"
)

// BenchmarkFieldNormalize benchmarks how long it takes the internal field
// to perform normalization (which includes modular reduction).
func BenchmarkFieldNormalize(b *testing.B) {
	// The function is constant time so any value is fine.
	f := &FieldVal{n: [10]uint32{
		0x000148f6, 0x03ffffc0, 0x03ffffff, 0x03ffffff, 0x03ffffff,
		0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x00000007,
	}}

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		f.Normalize()
	}
}

// BenchmarkFieldSqrt benchmarks calculating the square root of an unsigned
// 256-bit big-endian integer modulo the field prime  with the specialized type.
func BenchmarkFieldSqrt(b *testing.B) {
	// The function is constant time so any value is fine.
	valHex := "16fb970147a9acc73654d4be233cc48b875ce20a2122d24f073d29bd28805aca"
	f := new(FieldVal).SetHex(valHex).Normalize()

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		var result FieldVal
		_ = result.SquareRootVal(f)
	}
}

// BenchmarkBigSqrt benchmarks calculating the square root of an unsigned
// 256-bit big-endian integer modulo the field prime with stdlib big integers.
func BenchmarkBigSqrt(b *testing.B) {
	// The function is constant time so any value is fine.
	valHex := "16fb970147a9acc73654d4be233cc48b875ce20a2122d24f073d29bd28805aca"
	val, ok := new(big.Int).SetString(valHex, 16)
	if !ok {
		b.Fatalf("failed to parse hex %s", valHex)
	}

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = new(big.Int).ModSqrt(val, curveParams.P)
	}
}

// BenchmarkFieldIsGtOrEqPrimeMinusOrder benchmarks determining whether a value
// is greater than or equal to the field prime minus the group order with the
// specialized type.
func BenchmarkFieldIsGtOrEqPrimeMinusOrder(b *testing.B) {
	// The function is constant time so any value is fine.
	valHex := "16fb970147a9acc73654d4be233cc48b875ce20a2122d24f073d29bd28805aca"
	f := new(FieldVal).SetHex(valHex).Normalize()

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = f.IsGtOrEqPrimeMinusOrder()
	}
}

// BenchmarkBigIsGtOrEqPrimeMinusOrder benchmarks determining whether a value
// is greater than or equal to the field prime minus the group order with stdlib
// big integers.
func BenchmarkBigIsGtOrEqPrimeMinusOrder(b *testing.B) {
	// Same value used in field val version.
	valHex := "16fb970147a9acc73654d4be233cc48b875ce20a2122d24f073d29bd28805aca"
	val, ok := new(big.Int).SetString(valHex, 16)
	if !ok {
		b.Fatalf("failed to parse hex %s", valHex)
	}
	bigPMinusN := new(big.Int).Sub(curveParams.P, curveParams.N)

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		// In practice, the internal value to compare would have to be converted
		// to a big integer from bytes, so it's a fair comparison to allocate a
		// new big int here and set all bytes.
		_ = new(big.Int).SetBytes(val.Bytes()).Cmp(bigPMinusN) >= 0
	}
}
