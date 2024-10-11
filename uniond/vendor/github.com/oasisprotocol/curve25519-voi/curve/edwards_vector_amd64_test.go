// Copyright (c) 2016-2019 isis agora lovecruft. All rights reserved.
// Copyright (c) 2016-2019 Henry de Valence. All rights reserved.
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

//go:build amd64 && !purego && !force32bit

package curve

import (
	"testing"

	"github.com/oasisprotocol/curve25519-voi/internal/field"
)

func TestAVX2(t *testing.T) {
	if !supportsVectorizedEdwards {
		t.Skipf("AVX2 not supported")
	}

	t.Run("FieldElement2625x4", func(t *testing.T) {
		t.Run("ConditionalSelect", testVecConditionalSelect)
		t.Run("Neg", testVecNeg)
		t.Run("SquareAndNegateD", testVecSquareAndNegateD)
		t.Run("Mul", testVecMul)
		t.Run("NewSplit", testVecNewSplit)
	})
}

func TestSSE2(t *testing.T) {
	// It is increasingly difficult to find hardware that doesn't
	// support AVX2, but there is specific assembly to accelerate
	// table lookups.
	//
	// Make sure that the SSE2 code is tested, even if the AVX2
	// backend is enabled.
	if !supportsVectorizedEdwards {
		t.Skipf("SSE2 covered by generic tests")
	}

	// Temporarily replace the vector table with the generic one
	// and disable vector support.
	oldBasepointTable := ED25519_BASEPOINT_TABLE
	defer func() {
		supportsVectorizedEdwards = true
		ED25519_BASEPOINT_TABLE = oldBasepointTable
	}()

	ED25519_BASEPOINT_TABLE = &EdwardsBasepointTable{
		inner: newEdwardsBasepointTableGeneric(ED25519_BASEPOINT_POINT),
	}
	supportsVectorizedEdwards = false

	t.Run("BasepointTable/Basepoint", testEdwardsBasepointTableBasepoint)
	t.Run("BasepointTable/Mul", testEdwardsBasepointTableMul)
	t.Run("BasepointTable/Mul/One", testEdwardsBasepointTableMulOne)
	t.Run("BasepointTable/Mul/Two", testEdwardsBasepointTableMulTwo)
	t.Run("BasepointTable/Mul/VsEd25519py", testEdwardsBasepointTableMulVsEd25519py)
	t.Run("BasepointTable/Mul/ByBasepointOrder", testEdwardsBasepointTableMulByBasepointOrder)
}

func testVecConditionalSelect(t *testing.T) {
	a := testFieldElement2625x4()
	var out, b fieldElement2625x4

	// Fill out up with crap.
	for i := range out.inner {
		for j := range out.inner[i] {
			out.inner[i][j] = 0xdeadbeef
		}
	}

	out.ConditionalSelect(&a, &b, 0)
	if out.inner != a.inner {
		t.Fatalf("ConditionalSelect(a, b, 0) != a (Got: %v)", out)
	}

	out.ConditionalSelect(&a, &b, 1)
	if out.inner != b.inner {
		t.Fatalf("ConditionalSelect(a, b, 1) != b (Got: %v)", out)
	}
}

func testVecNeg(t *testing.T) {
	x0, x1, x2, x3 := testFieldElementComponents()
	vec := newFieldElement2625x4(x0, x1, x2, x3)

	vec.Neg()

	var y0, y1, y2, y3 field.Element
	vec.Split(&y0, &y1, &y2, &y3)

	var neg_x0, neg_x1, neg_x2, neg_x3 field.Element
	neg_x0.Neg(x0)
	neg_x1.Neg(x1)
	neg_x2.Neg(x2)
	neg_x3.Neg(x3)

	if neg_x0.Equal(&y0) != 1 {
		t.Fatalf("vec[0] != -x0 (Got: %v)", y0)
	}
	if neg_x1.Equal(&y1) != 1 {
		t.Fatalf("vec[1] != -x1 (Got: %v)", y1)
	}
	if neg_x2.Equal(&y2) != 1 {
		t.Fatalf("vec[2] != -x2 (Got: %v)", y2)
	}
	if neg_x3.Equal(&y3) != 1 {
		t.Fatalf("vec[3] != -x3 (Got: %v)", y3)
	}
}

func testVecSquareAndNegateD(t *testing.T) {
	x0, x1, x2, x3 := testFieldElementComponents()
	vec := newFieldElement2625x4(x0, x1, x2, x3)

	vec.SquareAndNegateD()

	var y0, y1, y2, y3 field.Element
	vec.Split(&y0, &y1, &y2, &y3)

	var x0sq, x1sq, x2sq, neg_x3sq field.Element
	x0sq.Mul(x0, x0)
	x1sq.Mul(x1, x1)
	x2sq.Mul(x2, x2)
	neg_x3sq.Mul(x3, x3)
	neg_x3sq.Neg(&neg_x3sq)

	if x0sq.Equal(&y0) != 1 {
		t.Fatalf("vec[0] != x0 * x0 (Got: %v)", y0)
	}
	if x1sq.Equal(&y1) != 1 {
		t.Fatalf("vec[1] != x1 * x1 (Got: %v)", y1)
	}
	if x2sq.Equal(&y2) != 1 {
		t.Fatalf("vec[2] != x2 * x2 (Got: %v)", y2)
	}
	if neg_x3sq.Equal(&y3) != 1 {
		t.Fatalf("vec[3] != -(x3 * x3) (Got: %v)", y3)
	}
}

func testVecMul(t *testing.T) {
	x0, x1, x2, x3 := testFieldElementComponents()
	vec := newFieldElement2625x4(x0, x1, x2, x3)

	vec.Mul(&vec, &vec)

	var y0, y1, y2, y3 field.Element
	vec.Split(&y0, &y1, &y2, &y3)

	var x0sq, x1sq, x2sq, x3sq field.Element
	x0sq.Mul(x0, x0)
	x1sq.Mul(x1, x1)
	x2sq.Mul(x2, x2)
	x3sq.Mul(x3, x3)

	if x0sq.Equal(&y0) != 1 {
		t.Fatalf("vec[0] != x0 * x0 (Got: %v %v)", y0, x0sq)
	}
	if x1sq.Equal(&y1) != 1 {
		t.Fatalf("vec[1] != x1 * x1 (Got: %v)", y1)
	}
	if x2sq.Equal(&y2) != 1 {
		t.Fatalf("vec[2] != x2 * x2 (Got: %v)", y2)
	}
	if x3sq.Equal(&y3) != 1 {
		t.Fatalf("vec[3] != x3 * x3 (Got: %v)", y3)
	}
}

func testVecNewSplit(t *testing.T) {
	x0, x1, x2, x3 := testFieldElementComponents()
	vec := newFieldElement2625x4(x0, x1, x2, x3)

	var y0, y1, y2, y3 field.Element
	vec.Split(&y0, &y1, &y2, &y3)

	if x0.Equal(&y0) != 1 {
		t.Fatalf("Split(0) != x0 (Got: %v)", y0)
	}
	if x1.Equal(&y1) != 1 {
		t.Fatalf("Split(1) != x1 (Got: %v)", y1)
	}
	if x2.Equal(&y2) != 1 {
		t.Fatalf("Split(2) != x2 (Got: %v)", y2)
	}
	if x3.Equal(&y3) != 1 {
		t.Fatalf("Split(3) != x3 (Got: %v)", y3)
	}
}

func testFieldElementComponents() (x0, x1, x2, x3 *field.Element) {
	// Just make a fieldElement2625x4 out of 2*B.
	compressedY := edwardsPointTestPoints["BASE2"]
	_ = compressedY

	var p EdwardsPoint
	_, _ = p.SetCompressedY(compressedY)

	return &p.inner.X, &p.inner.Y, &p.inner.Z, &p.inner.T
}

func testFieldElement2625x4() fieldElement2625x4 {
	x0, x1, x2, x3 := testFieldElementComponents()
	return newFieldElement2625x4(x0, x1, x2, x3)
}
