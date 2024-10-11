// Copyright (c) 2020 Jack Grigg. All rights reserved.
// Copyright (c) 2021 Oasis Labs Inc. All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
// 1. Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright
// notice, this list of conditions and the following disclaimer in the
// documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS
// IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED
// TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A
// PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED
// TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

package lattice

import (
	"crypto/rand"
	"testing"

	"github.com/oasisprotocol/curve25519-voi/curve/scalar"
)

func TestLatticeReduction(t *testing.T) {
	for i := 0; i < 500; i++ {
		k, err := scalar.New().SetRandom(rand.Reader)
		if err != nil {
			t.Fatalf("Failed to generate random scalar: %v", err)
		}

		d_0, d_1 := FindShortVector(k)

		// Ensure `d_0` and `d_1` are non-zero.
		if d_0.isZero() {
			t.Fatalf("Invariant violation, d_0 is 0")
		}
		if d_1.isZero() {
			t.Fatalf("Invariant violation, d_1 is 0")
		}

		// Ensure `d_0 = d_1 * k`.
		var s_0, s_1, should_be_d_0 scalar.Scalar
		d_0.ToScalar(&s_0)
		d_1.ToScalar(&s_1)
		should_be_d_0.Mul(k, &s_1)
		if s_0.Equal(&should_be_d_0) != 1 {
			t.Fatalf("d_0 != k * d_1 (Got: %v)", should_be_d_0)
		}
	}
}

func BenchmarkLatticeReduction(b *testing.B) {
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		// Rerandomize the scalar on each iteration, and hope that enough
		// iterations happen to give a good estimate of average runtime.
		b.StopTimer()
		k, err := scalar.New().SetRandom(rand.Reader)
		if err != nil {
			b.Fatalf("Failed to generate random scalar: %v", err)
		}
		b.StartTimer()

		_, _ = FindShortVector(k)
	}
}
