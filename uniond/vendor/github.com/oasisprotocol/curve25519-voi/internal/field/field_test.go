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

package field

import "testing"

func mustFeFromBytes(b []byte) *Element {
	var fe Element
	if _, err := fe.SetBytes(b); err != nil {
		panic("mustFeFromBytes: " + err.Error())
	}
	return &fe
}

var testConstants = map[string]*Element{
	// Random element a of GF(2^255-19), from Sage
	// a = 1070314506888354081329385823235218444233221\
	//     2228051251926706380353716438957572
	"A": mustFeFromBytes([]byte{
		0x04, 0xfe, 0xdf, 0x98, 0xa7, 0xfa, 0x0a, 0x68,
		0x84, 0x92, 0xbd, 0x59, 0x08, 0x07, 0xa7, 0x03,
		0x9e, 0xd1, 0xf6, 0xf2, 0xe1, 0xd9, 0xe2, 0xa4,
		0xa4, 0x51, 0x47, 0x36, 0xf3, 0xc3, 0xa9, 0x17,
	}),

	"ASQ": mustFeFromBytes([]byte{
		0x75, 0x97, 0x24, 0x9e, 0xe6, 0x06, 0xfe, 0xab,
		0x24, 0x04, 0x56, 0x68, 0x07, 0x91, 0x2d, 0x5d,
		0x0b, 0x0f, 0x3f, 0x1c, 0xb2, 0x6e, 0xf2, 0xe2,
		0x63, 0x9c, 0x12, 0xba, 0x73, 0x0b, 0xe3, 0x62,
	}),

	"AINV": mustFeFromBytes([]byte{
		0x96, 0x1b, 0xcd, 0x8d, 0x4d, 0x5e, 0xa2, 0x3a,
		0xe9, 0x36, 0x37, 0x93, 0xdb, 0x7b, 0x4d, 0x70,
		0xb8, 0x0d, 0xc0, 0x55, 0xd0, 0x4c, 0x1d, 0x7b,
		0x90, 0x71, 0xd8, 0xe9, 0xb6, 0x18, 0xe6, 0x30,
	}),

	"AP58": mustFeFromBytes([]byte{
		0x6a, 0x4f, 0x24, 0x89, 0x1f, 0x57, 0x60, 0x36,
		0xd0, 0xbe, 0x12, 0x3c, 0x8f, 0xf5, 0xb1, 0x59,
		0xe0, 0xf0, 0xb8, 0x1b, 0x20, 0xd2, 0xb5, 0x1f,
		0x15, 0x21, 0xf9, 0xe3, 0xe1, 0x61, 0x21, 0x55,
	}),

	"AP16": mustFeFromBytes([]byte{
		0x3d, 0x5e, 0x76, 0x49, 0x96, 0xf0, 0x80, 0x2f,
		0xe6, 0xfd, 0xea, 0x1f, 0x1a, 0x5a, 0xe6, 0xb8,
		0xc9, 0xee, 0xf2, 0x30, 0xc0, 0x76, 0x15, 0x61,
		0xb0, 0x5b, 0xd0, 0xf3, 0x47, 0x8c, 0x72, 0x07,
	}),
}

func TestElement(t *testing.T) {
	t.Run("Mul", testMul)
	t.Run("Square", testSquare)
	t.Run("Square2", testSquare2)
	t.Run("Invert", testInvert)
	t.Run("BatchInvert/Consistency", testBatchInvertConsistency)
	t.Run("BatchInvert/Empty", testBatchInvertEmpty)
	t.Run("SqrtRatioI", testSqrtRatioI)
	t.Run("PowP58", testPowP58)
	t.Run("Equal", testEqual)
	t.Run("SetBytes/HighBitIsIgnored", testSetBytesHighBitIsIgnored)
	t.Run("ConditionalNegate", testConditionalNegate)
	t.Run("ToBytes/EncodingIsCanonical", testToBytesEncodingIsCanonical)
	t.Run("Constants/SqrtM1", testConstantsSqrtMinusOne)
	t.Run("Constants/SqrtM1/Sign", testConstantsSqrtConstantsSign)
}

func testMul(t *testing.T) {
	a, asq := testConstants["A"], testConstants["ASQ"]

	var shouldBeAsq Element
	shouldBeAsq.Mul(a, a)

	if shouldBeAsq.Equal(asq) != 1 {
		t.Fatalf("a * a != asq (Got: %v)", shouldBeAsq)
	}
}

func testSquare(t *testing.T) {
	a, asq := testConstants["A"], testConstants["ASQ"]

	var shouldBeAsq Element
	shouldBeAsq.Square(a)

	if shouldBeAsq.Equal(asq) != 1 {
		t.Fatalf("a.Square() != asq (Got: %v)", shouldBeAsq)
	}
}

func testSquare2(t *testing.T) {
	a, asq := testConstants["A"], testConstants["ASQ"]

	var asq2 Element
	asq2.Add(asq, asq)

	var shouldBeAsq2 Element
	shouldBeAsq2.Square2(a)

	if shouldBeAsq2.Equal(&asq2) != 1 {
		t.Fatalf("a.Square2() != asq + asq (Got: %v)", shouldBeAsq2)
	}
}

func testInvert(t *testing.T) {
	a, ainv := testConstants["A"], testConstants["AINV"]

	var shouldBeInverse Element
	shouldBeInverse.Invert(a)

	if shouldBeInverse.Equal(ainv) != 1 {
		t.Fatalf("a.Invert() != ainv (Got: %v)", shouldBeInverse)
	}

	var shouldBeOne Element
	shouldBeOne.Mul(a, &shouldBeInverse)

	if shouldBeOne.Equal(&One) != 1 {
		t.Fatalf("a.Invert() * a != 1 (Got: %v)", shouldBeOne)
	}
}

func testBatchInvertConsistency(t *testing.T) {
	a := testConstants["A"]

	var a0 Element
	a0.Sub(a, a)

	var a2 Element
	a2.Add(a, a)

	aList := []*Element{
		a,
		testConstants["AP58"],
		testConstants["ASQ"],
		testConstants["AINV"],
		&a0,
		&a2,
	}
	var ainvList []*Element
	for _, v := range aList {
		tmp := *v
		ainvList = append(ainvList, &tmp)
	}

	BatchInvert(ainvList)
	for i, v := range aList {
		var expected Element
		expected.Invert(v)
		if ainvList[i].Equal(&expected) != 1 {
			t.Fatalf("aList[%d].Invert() != ainvList[%d] (Got: %v, %v)", i, i, expected, ainvList[i])
		}
	}
}

func testBatchInvertEmpty(t *testing.T) {
	BatchInvert([]*Element{})
	BatchInvert(nil)
}

func testSqrtRatioI(t *testing.T) {
	var zero, two_i, four, sqrt Element
	two_i.Mul(&Two, &SQRT_M1)
	four.Add(&Two, &Two)

	// 0/0 should return (0, 1) since u is 0
	_, choice := sqrt.SqrtRatioI(&zero, &zero)
	if choice != 1 {
		t.Fatalf("sqrt.RatioI(0, 0) choice != 1")
	}
	if sqrt.Equal(&zero) != 1 {
		t.Fatalf("sqrtRatioI(0, 0) sqrt != 0 (Got: %v)", sqrt)
	}
	if sqrt.IsNegative() != 0 {
		t.Fatalf("sqrt.IsNegative() != 0")
	}

	// 1/0 should return (0, 0) since v is 0, u is nonzero
	_, choice = sqrt.SqrtRatioI(&One, &zero)
	if choice != 0 {
		t.Fatalf("sqrt.RatioI(1, 0) choice != 0")
	}
	if sqrt.Equal(&zero) != 1 {
		t.Fatalf("sqrtRatioI(1, 0) sqrt != 0 (Got: %v)", sqrt)
	}
	if sqrt.IsNegative() != 0 {
		t.Fatalf("sqrt.IsNegative() != 0")
	}

	// 2/1 is nonsquare, so we expect (sqrt(i*2), 0)
	_, choice = sqrt.SqrtRatioI(&Two, &One)
	if choice != 0 {
		t.Fatalf("sqrt.RatioI(2, 1) choice != 0")
	}
	var sqrtSquared Element
	sqrtSquared.Square(&sqrt)
	if sqrtSquared.Equal(&two_i) != 1 {
		t.Fatalf("sqrtRatioI(2, 1) sqrt^2 != 2 * i (Got: %v)", sqrtSquared)
	}
	if sqrt.IsNegative() != 0 {
		t.Fatalf("sqrt.IsNegative() != 0")
	}

	// 4/1 is square, so we expect (sqrt(4), 1)
	_, choice = sqrt.SqrtRatioI(&four, &One)
	if choice != 1 {
		t.Fatalf("sqrt.RatioI(4, 1) choice != 1")
	}
	sqrtSquared.Square(&sqrt)
	if sqrtSquared.Equal(&four) != 1 {
		t.Fatalf("sqrtRatioI(4, 1) sqrt^2 != 4 * i (Got: %v)", sqrtSquared)
	}
	if sqrt.IsNegative() != 0 {
		t.Fatalf("sqrt.IsNegative() != 0")
	}

	// 1/4 is square, so we expect (1/sqrt(4), 1)
	_, choice = sqrt.SqrtRatioI(&One, &four)
	if choice != 1 {
		t.Fatalf("sqrt.RatioI(4, 1) choice != 1")
	}
	var tmp Element
	tmp.Square(&sqrt)
	tmp.Mul(&tmp, &four)
	if tmp.Equal(&One) != 1 {
		t.Fatalf("sqrtRatioI(4, 1) sqrt^2 * 4 != 1 (Got: %v)", tmp)
	}
	if sqrt.IsNegative() != 0 {
		t.Fatalf("sqrt.IsNegative() != 0")
	}
}

func testPowP58(t *testing.T) {
	a, ap58 := testConstants["A"], testConstants["AP58"]

	shouldBeAp58 := *a
	shouldBeAp58.pow_p58()

	if shouldBeAp58.Equal(ap58) != 1 {
		t.Fatalf("a.pow_p58() != ap58 (Got: %v)", shouldBeAp58)
	}
}

func testEqual(t *testing.T) {
	a, ainv := testConstants["A"], testConstants["AINV"]

	if a.Equal(a) != 1 {
		t.Fatalf("a != a")
	}
	if a.Equal(ainv) != 0 {
		t.Fatalf("a == ainv")
	}
}

func testSetBytesHighBitIsIgnored(t *testing.T) {
	// Notice that the last element has the high bit set, which
	// should be ignored.
	bBytes := [ElementSize]byte{
		113, 191, 169, 143, 91, 234, 121, 15,
		241, 131, 217, 36, 230, 101, 92, 234,
		8, 208, 170, 251, 97, 127, 70, 210,
		58, 23, 166, 87, 240, 169, 184, 178,
	}

	var withHighBitSet, withoutHighBitSet Element
	if _, err := withHighBitSet.SetBytes(bBytes[:]); err != nil {
		t.Fatalf("withHighBitSet SetBytes(): %v", err)
	}

	clearedBytes := bBytes
	clearedBytes[31] &= 127

	if _, err := withoutHighBitSet.SetBytes(clearedBytes[:]); err != nil {
		t.Fatalf("withoutHighBitSet SetBytes(): %v", err)
	}

	if withHighBitSet.Equal(&withoutHighBitSet) != 1 {
		t.Fatalf("withoutHighBitSet != withHighBitSet (Got: %v)", withHighBitSet)
	}
}

func testConditionalNegate(t *testing.T) {
	x := One
	x.ConditionalNegate(1)
	if x.Equal(&MinusOne) != 1 {
		t.Fatalf("x.ConditionalNegate(1) != -1 (Got: %v)", x)
	}
	x.ConditionalNegate(0)
	if x.Equal(&MinusOne) != 1 {
		t.Fatalf("x.ConditionalNegate(0) != -1 (Got: %v)", x)
	}
	x.ConditionalNegate(1)
	if x.Equal(&One) != 1 {
		t.Fatalf("x.ConditionalNegate(1) != 1 (Got: %v)", x)
	}
}

func testToBytesEncodingIsCanonical(t *testing.T) {
	// Encode 1 wrongly as 1 + (2^255 - 19) = 2^255 - 18
	oneEncodedWronglyBytes := [ElementSize]byte{
		0xee, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f,
	}

	var one Element
	if _, err := one.SetBytes(oneEncodedWronglyBytes[:]); err != nil {
		t.Fatalf("one SetBytes(): %v", err)
	}

	var oneBytes, expectedOneBytes [ElementSize]byte
	if err := one.ToBytes(oneBytes[:]); err != nil {
		t.Fatalf("one.ToBytes(): %v", err)
	}

	expectedOneBytes[0] = 1
	if oneBytes != expectedOneBytes {
		t.Fatalf("oneBytes != [1, 0, 0, ...] (Got: %v)", oneBytes)
	}
}

func testConstantsSqrtMinusOne(t *testing.T) {
	var sqrtM1Sq Element
	sqrtM1Sq.Mul(&SQRT_M1, &SQRT_M1)
	if MinusOne.Equal(&sqrtM1Sq) != 1 {
		t.Fatalf("SQRT_M1 * SQRT_M1 != -1")
	}
	if SQRT_M1.IsNegative() != 0 {
		t.Fatalf("SQRT_M1.IsNegative() != 0")
	}
}

func testConstantsSqrtConstantsSign(t *testing.T) {
	var signTestSqrt Element
	invSqrtM1 := MinusOne
	_, wasNonZeroSquare := invSqrtM1.InvSqrt()
	if wasNonZeroSquare != 1 {
		t.Fatalf("-1.InvSqrt() wasNonZeroSquare != 1")
	}
	signTestSqrt.Mul(&invSqrtM1, &SQRT_M1)
	if signTestSqrt.Equal(&MinusOne) != 1 {
		t.Fatalf("invSqrtM1 * SQRT_M1 != -1 (Got: %v)", signTestSqrt)
	}
}

func BenchmarkElement(b *testing.B) {
	b.Run("Mul", benchMul)
	b.Run("Square", benchSquare)
	b.Run("Invert", benchInvert)
}

func benchMul(b *testing.B) {
	var x, y Element
	x.One()
	y.Add(&x, &x)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		x.Mul(&x, &y)
	}
}

func benchSquare(b *testing.B) {
	var x Element
	x.One()
	x.Add(&x, &x)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		x.Square(&x)
	}
}

func benchInvert(b *testing.B) {
	var x Element
	x.One()
	x.Add(&x, &x)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		x.Invert(&x)
	}
}
