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

package curve

import (
	"crypto/rand"
	"testing"

	"github.com/oasisprotocol/curve25519-voi/curve/scalar"
)

func testEdwardsMulAbglsvPorninVartime(t *testing.T) {
	// Returns what C should be to satisfy the equation.
	deriveC := func(out *EdwardsPoint, a *scalar.Scalar, A *EdwardsPoint, b *scalar.Scalar) {
		var aA EdwardsPoint
		aA.Mul(A, a)

		var bB EdwardsPoint
		bB.Mul(ED25519_BASEPOINT_POINT, b)

		out.Add(&aA, &bB)
	}

	// Returns the results of the equation, calculated the hard way.
	slowMul := func(a *scalar.Scalar, A *EdwardsPoint, b *scalar.Scalar, C *EdwardsPoint) *EdwardsPoint {
		var aA_plus_bB EdwardsPoint
		deriveC(&aA_plus_bB, a, A, b)

		var ret EdwardsPoint
		ret.Sub(&aA_plus_bB, C)

		return &ret
	}

	var (
		a, b scalar.Scalar
		A, C EdwardsPoint

		actual EdwardsPoint
	)

	// The equation evaluates to the identity, so will be unaffected by delta.
	a.SetUint64(2)
	A.double(ED25519_BASEPOINT_POINT)
	b.SetUint64(4)
	C.double(C.double(&A))

	edwardsMulAbglsvPorninVartime(&actual, &a, &A, &b, &C)
	expected := slowMul(&a, &A, &b, &C)
	if !expected.IsIdentity() {
		t.Fatalf("slowMul(2, B, 4, 8B) != identity (Got: %v)", expected)
	}
	if expected.Equal(&actual) != 1 || !actual.IsIdentity() {
		t.Fatalf("mul(2, B, 4, 8B) != identity (Got: %v)", actual)
	}

	expandedA := NewExpandedEdwardsPoint(&A)
	expandedEdwardsMulAbglsvPorninVartime(&actual, &a, expandedA, &b, &C)
	if expected.Equal(&actual) != 1 || !actual.IsIdentity() {
		t.Fatalf("expandedMul(2, B , 4, 4, 8B) != identity (Got: %v)", actual)
	}

	for i := 0; i < 100; i++ {
		if _, err := a.SetRandom(rand.Reader); err != nil {
			t.Fatalf("a.SetRandom(): %v", err)
		}
		A.Mul(ED25519_BASEPOINT_POINT, newTestBenchRandomScalar(t))
		if _, err := b.SetRandom(rand.Reader); err != nil {
			t.Fatalf("b.SetRandom(): %v", err)
		}

		// With a correctly-constructed C, we get the identity.
		deriveC(&C, &a, &A, &b)
		expected = slowMul(&a, &A, &b, &C)
		if !expected.IsIdentity() {
			t.Fatalf("slowMul(a, A, b, C) != identity (Got: %v)", expected)
		}
		edwardsMulAbglsvPorninVartime(&actual, &a, &A, &b, &C)
		if expected.Equal(&actual) != 1 || !actual.IsIdentity() {
			t.Fatalf("mul(a, A, b, C) != identity (Got: %v)", actual)
		}

		expandedA = NewExpandedEdwardsPoint(&A)
		expandedEdwardsMulAbglsvPorninVartime(&actual, &a, expandedA, &b, &C)
		if expected.Equal(&actual) != 1 || !actual.IsIdentity() {
			t.Fatalf("expandedMul(a, A, b, C) != identity (Got: %v)", actual)
		}

		// With a random C, with high probability we do not get the identity.
		for {
			// Loop till we get a C that is sufficiently random.
			C.Mul(ED25519_BASEPOINT_POINT, newTestBenchRandomScalar(t))
			expected = slowMul(&a, &A, &b, &C)
			if !expected.IsIdentity() {
				break
			}
		}
		edwardsMulAbglsvPorninVartime(&actual, &a, &A, &b, &C)
		if actual.IsIdentity() {
			t.Fatalf("mul(a, A, b, random C) = identity")
		}
		expandedEdwardsMulAbglsvPorninVartime(&actual, &a, expandedA, &b, &C)
		if actual.IsIdentity() {
			t.Fatalf("expandedMul(a, A, b, random C) = identity")
		}
	}
}
