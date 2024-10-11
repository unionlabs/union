// Copyright (c) 2016-2019 isis agora lovecruft. All rights reserved.
// Copyright (c) 2016-2019 Henry de Valence. All rights reserved.
// Copyright (c) 2020-2021 Oasis Labs Inc. All rights reserved.
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
	"testing"

	"github.com/oasisprotocol/curve25519-voi/internal/field"
)

func TestMontgomery(t *testing.T) {
	t.Run("EdwardsPointFromMontgomery", testMontgomeryEdwardsPointFromMontgomery)
	t.Run("EdwardsPointFromMontgomery/RejectsTwist", testMontgomeryEdwardsPointFromMontgomeryRejectsTwist)
	t.Run("FromEdwards", testMontgomeryFromEdwards)
	t.Run("Equal", testMontgomeryEqual)
	t.Run("Mul", testMontgomeryMul)
}

func testMontgomeryEdwardsPointFromMontgomery(t *testing.T) {
	var p EdwardsPoint
	if _, err := p.SetMontgomery(X25519_BASEPOINT, 0); err != nil {
		t.Fatalf("SetMontgomery(X25519_BASEPOINT, 0)")
	}
	if p.Equal(ED25519_BASEPOINT_POINT) != 1 {
		t.Fatalf("SetMontgomery(X25519_BASEPOINT, 0) != ED25519_BASEPOINT_POINT (Got: %v)", p)
	}

	var negBasepoint EdwardsPoint
	negBasepoint.Neg(ED25519_BASEPOINT_POINT)
	if _, err := p.SetMontgomery(X25519_BASEPOINT, 1); err != nil {
		t.Fatalf("SetMontgomery(X25519_BASEPOINT, 0)")
	}
	if p.Equal(&negBasepoint) != 1 {
		t.Fatalf("SetMontgomery-X25519_BASEPOINT, 1) != -ED25519_BASEPOINT_POINT (Got: %v)", p)
	}
}

func testMontgomeryEdwardsPointFromMontgomeryRejectsTwist(t *testing.T) {
	// u = 2 corresponds to a point on the twist.
	var pM MontgomeryPoint
	_ = field.Two.ToBytes(pM[:])

	var p EdwardsPoint
	if _, err := p.SetMontgomery(&pM, 0); err == nil {
		t.Fatalf("SetMontgomery(2, 0) != error (Got: %v)", p)
	}

	// u = -1 corresponds to a point on the twist, but should be
	// checked explicitly because it's an exceptional point for the
	// birational map.  For instance, libsignal will accept it.
	_ = field.MinusOne.ToBytes(pM[:])
	if _, err := p.SetMontgomery(&pM, 0); err == nil {
		t.Fatalf("SetMontgomery(-1, 0) != error (Got: %v)", p)
	}
}

func testMontgomeryFromEdwards(t *testing.T) {
	var p MontgomeryPoint
	p.SetEdwards(ED25519_BASEPOINT_POINT)
	if p.Equal(X25519_BASEPOINT) != 1 {
		t.Fatalf("FromEdwards(ED25519_BASEPOINT_POINT) != X25519_BASEPOINT (Got: %v)", p)
	}
}

func testMontgomeryEqual(t *testing.T) {
	u18Bytes := [MontgomeryPointSize]byte{18}
	u18UnreducedBytes := [MontgomeryPointSize]byte{
		255, 255, 255, 255, 255, 255, 255, 255,
		255, 255, 255, 255, 255, 255, 255, 255,
		255, 255, 255, 255, 255, 255, 255, 255,
		255, 255, 255, 255, 255, 255, 255, 255,
	}

	u18, u18Unreduced := MontgomeryPoint(u18Bytes), MontgomeryPoint(u18UnreducedBytes)
	if u18.Equal(&u18Unreduced) != 1 {
		t.Fatalf("u18 != u18Unreduced")
	}
}

func testMontgomeryMul(t *testing.T) {
	s := newTestBenchRandomScalar(t)

	var pEdwards EdwardsPoint
	pEdwards.MulBasepoint(ED25519_BASEPOINT_TABLE, s)

	var pMontgomery MontgomeryPoint
	pMontgomery.SetEdwards(&pEdwards)

	var expected EdwardsPoint
	expected.Mul(&pEdwards, s)

	var result MontgomeryPoint
	result.Mul(&pMontgomery, s)

	var expectedMontgomery MontgomeryPoint
	expectedMontgomery.SetEdwards(&expected)
	if result.Equal(&expectedMontgomery) != 1 {
		t.Fatalf("s * p_edwards != s * p_montgomery (Got: %v, %v)", expectedMontgomery, result)
	}
}
