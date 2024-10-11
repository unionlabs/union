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

package elligator

import (
	"encoding/binary"
	"testing"

	"github.com/oasisprotocol/curve25519-voi/internal/field"
)

func TestConstants(t *testing.T) {
	t.Run("A", testConstantsA)
	t.Run("NegA", testConstantsNegA)
	t.Run("ASquared", testConstantsASquared)
	t.Run("SqrtNegAPlusTwo", testConstantsSqrtNegAPlusTwo)
	t.Run("UFactor", testConstantsUFactor)
	t.Run("VFactor", testConstantsVFactor)
}

func testConstantsA(t *testing.T) {
	expected := feFromUint64(486662)

	if constMONTGOMERY_A.Equal(expected) != 1 {
		t.Fatalf("A != 486662 (Got: %v)", constMONTGOMERY_A)
	}
}

func testConstantsNegA(t *testing.T) {
	expected := new(field.Element).Neg(&constMONTGOMERY_A)

	if constMONTGOMERY_NEG_A.Equal(expected) != 1 {
		t.Fatalf("NEG_A != -A (Got: %v)", constMONTGOMERY_NEG_A)
	}
}

func testConstantsASquared(t *testing.T) {
	var expected field.Element
	expected.Square(&constMONTGOMERY_A)

	if constMONTGOMERY_A_SQUARED.Equal(&expected) != 1 {
		t.Fatalf("A_SQUARED != A^2 (Got: %v)", constMONTGOMERY_A_SQUARED)
	}
}

func testConstantsSqrtNegAPlusTwo(t *testing.T) {
	var expected field.Element
	expected.Sub(&constMONTGOMERY_NEG_A, &field.Two)
	expected.Invert(&expected)
	expected.InvSqrt()

	if constMONTGOMERY_SQRT_NEG_A_PLUS_TWO.Equal(&expected) != 1 {
		t.Fatalf("SQRT_NEG_A_PLUS_TWO != sqrt(-(A+2)) (Got: %v)", constMONTGOMERY_SQRT_NEG_A_PLUS_TWO)
	}
}

func testConstantsUFactor(t *testing.T) {
	var expected field.Element
	expected.Neg(&field.Two)
	expected.Mul(&expected, &field.SQRT_M1)

	if constMONTGOMERY_U_FACTOR.Equal(&expected) != 1 {
		t.Fatalf("U_FACTOR != -2 * sqrt(-1) (Got: %v)", constMONTGOMERY_U_FACTOR)
	}
}

func testConstantsVFactor(t *testing.T) {
	var expected field.Element
	expected.Invert(&constMONTGOMERY_U_FACTOR)
	expected.InvSqrt()

	if constMONTGOMERY_V_FACTOR.Equal(&expected) != 1 {
		t.Fatalf("V_FACTOR != sqrt(u_factor) (Got: %v)", constMONTGOMERY_V_FACTOR)
	}
}

func feFromUint64(x uint64) *field.Element {
	var feBytes [field.ElementSize]byte
	binary.LittleEndian.PutUint64(feBytes[0:8], x)

	var fe field.Element
	_, _ = fe.SetBytes(feBytes[:])
	return &fe
}
