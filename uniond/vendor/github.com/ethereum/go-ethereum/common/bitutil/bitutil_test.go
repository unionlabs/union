// Copyright 2013 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Adapted from: https://golang.org/src/crypto/cipher/xor_test.go

package bitutil

import (
	"bytes"
	"testing"
)

// Tests that bitwise XOR works for various alignments.
func TestXOR(t *testing.T) {
	for alignP := 0; alignP < 2; alignP++ {
		for alignQ := 0; alignQ < 2; alignQ++ {
			for alignD := 0; alignD < 2; alignD++ {
				p := make([]byte, 1023)[alignP:]
				q := make([]byte, 1023)[alignQ:]

				for i := 0; i < len(p); i++ {
					p[i] = byte(i)
				}
				for i := 0; i < len(q); i++ {
					q[i] = byte(len(q) - i)
				}
				d1 := make([]byte, 1023+alignD)[alignD:]
				d2 := make([]byte, 1023+alignD)[alignD:]

				XORBytes(d1, p, q)
				safeXORBytes(d2, p, q)
				if !bytes.Equal(d1, d2) {
					t.Error("not equal", d1, d2)
				}
			}
		}
	}
}

// Tests that bitwise AND works for various alignments.
func TestAND(t *testing.T) {
	for alignP := 0; alignP < 2; alignP++ {
		for alignQ := 0; alignQ < 2; alignQ++ {
			for alignD := 0; alignD < 2; alignD++ {
				p := make([]byte, 1023)[alignP:]
				q := make([]byte, 1023)[alignQ:]

				for i := 0; i < len(p); i++ {
					p[i] = byte(i)
				}
				for i := 0; i < len(q); i++ {
					q[i] = byte(len(q) - i)
				}
				d1 := make([]byte, 1023+alignD)[alignD:]
				d2 := make([]byte, 1023+alignD)[alignD:]

				ANDBytes(d1, p, q)
				safeANDBytes(d2, p, q)
				if !bytes.Equal(d1, d2) {
					t.Error("not equal")
				}
			}
		}
	}
}

// Tests that bitwise OR works for various alignments.
func TestOR(t *testing.T) {
	for alignP := 0; alignP < 2; alignP++ {
		for alignQ := 0; alignQ < 2; alignQ++ {
			for alignD := 0; alignD < 2; alignD++ {
				p := make([]byte, 1023)[alignP:]
				q := make([]byte, 1023)[alignQ:]

				for i := 0; i < len(p); i++ {
					p[i] = byte(i)
				}
				for i := 0; i < len(q); i++ {
					q[i] = byte(len(q) - i)
				}
				d1 := make([]byte, 1023+alignD)[alignD:]
				d2 := make([]byte, 1023+alignD)[alignD:]

				ORBytes(d1, p, q)
				safeORBytes(d2, p, q)
				if !bytes.Equal(d1, d2) {
					t.Error("not equal")
				}
			}
		}
	}
}

// Tests that bit testing works for various alignments.
func TestTest(t *testing.T) {
	for align := 0; align < 2; align++ {
		// Test for bits set in the bulk part
		p := make([]byte, 1023)[align:]
		p[100] = 1

		if TestBytes(p) != safeTestBytes(p) {
			t.Error("not equal")
		}
		// Test for bits set in the tail part
		q := make([]byte, 1023)[align:]
		q[len(q)-1] = 1

		if TestBytes(q) != safeTestBytes(q) {
			t.Error("not equal")
		}
	}
}

// Benchmarks the potentially optimized XOR performance.
func BenchmarkFastXOR1KB(b *testing.B) { benchmarkFastXOR(b, 1024) }
func BenchmarkFastXOR2KB(b *testing.B) { benchmarkFastXOR(b, 2048) }
func BenchmarkFastXOR4KB(b *testing.B) { benchmarkFastXOR(b, 4096) }

func benchmarkFastXOR(b *testing.B, size int) {
	p, q := make([]byte, size), make([]byte, size)

	for i := 0; i < b.N; i++ {
		XORBytes(p, p, q)
	}
}

// Benchmarks the baseline XOR performance.
func BenchmarkBaseXOR1KB(b *testing.B) { benchmarkBaseXOR(b, 1024) }
func BenchmarkBaseXOR2KB(b *testing.B) { benchmarkBaseXOR(b, 2048) }
func BenchmarkBaseXOR4KB(b *testing.B) { benchmarkBaseXOR(b, 4096) }

func benchmarkBaseXOR(b *testing.B, size int) {
	p, q := make([]byte, size), make([]byte, size)

	for i := 0; i < b.N; i++ {
		safeXORBytes(p, p, q)
	}
}

// Benchmarks the potentially optimized AND performance.
func BenchmarkFastAND1KB(b *testing.B) { benchmarkFastAND(b, 1024) }
func BenchmarkFastAND2KB(b *testing.B) { benchmarkFastAND(b, 2048) }
func BenchmarkFastAND4KB(b *testing.B) { benchmarkFastAND(b, 4096) }

func benchmarkFastAND(b *testing.B, size int) {
	p, q := make([]byte, size), make([]byte, size)

	for i := 0; i < b.N; i++ {
		ANDBytes(p, p, q)
	}
}

// Benchmarks the baseline AND performance.
func BenchmarkBaseAND1KB(b *testing.B) { benchmarkBaseAND(b, 1024) }
func BenchmarkBaseAND2KB(b *testing.B) { benchmarkBaseAND(b, 2048) }
func BenchmarkBaseAND4KB(b *testing.B) { benchmarkBaseAND(b, 4096) }

func benchmarkBaseAND(b *testing.B, size int) {
	p, q := make([]byte, size), make([]byte, size)

	for i := 0; i < b.N; i++ {
		safeANDBytes(p, p, q)
	}
}

// Benchmarks the potentially optimized OR performance.
func BenchmarkFastOR1KB(b *testing.B) { benchmarkFastOR(b, 1024) }
func BenchmarkFastOR2KB(b *testing.B) { benchmarkFastOR(b, 2048) }
func BenchmarkFastOR4KB(b *testing.B) { benchmarkFastOR(b, 4096) }

func benchmarkFastOR(b *testing.B, size int) {
	p, q := make([]byte, size), make([]byte, size)

	for i := 0; i < b.N; i++ {
		ORBytes(p, p, q)
	}
}

// Benchmarks the baseline OR performance.
func BenchmarkBaseOR1KB(b *testing.B) { benchmarkBaseOR(b, 1024) }
func BenchmarkBaseOR2KB(b *testing.B) { benchmarkBaseOR(b, 2048) }
func BenchmarkBaseOR4KB(b *testing.B) { benchmarkBaseOR(b, 4096) }

func benchmarkBaseOR(b *testing.B, size int) {
	p, q := make([]byte, size), make([]byte, size)

	for i := 0; i < b.N; i++ {
		safeORBytes(p, p, q)
	}
}

var GloBool bool // Exported global will not be dead-code eliminated, at least not yet.

// Benchmarks the potentially optimized bit testing performance.
func BenchmarkFastTest1KB(b *testing.B) { benchmarkFastTest(b, 1024) }
func BenchmarkFastTest2KB(b *testing.B) { benchmarkFastTest(b, 2048) }
func BenchmarkFastTest4KB(b *testing.B) { benchmarkFastTest(b, 4096) }

func benchmarkFastTest(b *testing.B, size int) {
	p := make([]byte, size)
	a := false
	for i := 0; i < b.N; i++ {
		a = a != TestBytes(p)
	}
	GloBool = a // Use of benchmark "result" to prevent total dead code elimination.
}

// Benchmarks the baseline bit testing performance.
func BenchmarkBaseTest1KB(b *testing.B) { benchmarkBaseTest(b, 1024) }
func BenchmarkBaseTest2KB(b *testing.B) { benchmarkBaseTest(b, 2048) }
func BenchmarkBaseTest4KB(b *testing.B) { benchmarkBaseTest(b, 4096) }

func benchmarkBaseTest(b *testing.B, size int) {
	p := make([]byte, size)
	a := false
	for i := 0; i < b.N; i++ {
		a = a != safeTestBytes(p)
	}
	GloBool = a // Use of benchmark "result" to prevent total dead code elimination.
}
