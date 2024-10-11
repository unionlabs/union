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

package scalar

import (
	"crypto/rand"
	"strconv"
	"testing"

	"github.com/oasisprotocol/curve25519-voi/internal/testhelpers"
)

var testConstants = map[string]*Scalar{
	// x = 2238329342913194256032495932344128051776374960164957527413114840482143558222
	"X": newRawScalar([]byte{
		0x4e, 0x5a, 0xb4, 0x34, 0x5d, 0x47, 0x08, 0x84,
		0x59, 0x13, 0xb4, 0x64, 0x1b, 0xc2, 0x7d, 0x52,
		0x52, 0xa5, 0x85, 0x10, 0x1b, 0xcc, 0x42, 0x44,
		0xd4, 0x49, 0xf4, 0xa8, 0x79, 0xd9, 0xf2, 0x04,
	}),

	// 1/x = 6859937278830797291664592131120606308688036382723378951768035303146619657244
	"XINV": newRawScalar([]byte{
		0x1c, 0xdc, 0x17, 0xfc, 0xe0, 0xe9, 0xa5, 0xbb,
		0xd9, 0x24, 0x7e, 0x56, 0xbb, 0x01, 0x63, 0x47,
		0xbb, 0xba, 0x31, 0xed, 0xd5, 0xa9, 0xbb, 0x96,
		0xd5, 0x0b, 0xcd, 0x7a, 0x3f, 0x96, 0x2a, 0x0f,
	}),

	// y = 2592331292931086675770238855846338635550719849568364935475441891787804997264
	"Y": newRawScalar([]byte{
		0x90, 0x76, 0x33, 0xfe, 0x1c, 0x4b, 0x66, 0xa4,
		0xa2, 0x8d, 0x2d, 0xd7, 0x67, 0x83, 0x86, 0xc3,
		0x53, 0xd0, 0xde, 0x54, 0x55, 0xd4, 0xfc, 0x9d,
		0xe8, 0xef, 0x7a, 0xc3, 0x1f, 0x35, 0xbb, 0x05,
	}),

	// x*y = 5690045403673944803228348699031245560686958845067437804563560795922180092780
	"XY": newRawScalar([]byte{
		0x6c, 0x33, 0x74, 0xa1, 0x89, 0x4f, 0x62, 0x21,
		0x0a, 0xaa, 0x2f, 0xe1, 0x86, 0xa6, 0xf9, 0x2c,
		0xe0, 0xaa, 0x75, 0xc2, 0x77, 0x95, 0x81, 0xc2,
		0x95, 0xfc, 0x08, 0x17, 0x9a, 0x73, 0x94, 0x0c,
	}),

	"LARGEST_ED25519_S": newRawScalar([]byte{
		0xf8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f,
	}),

	"CANONICAL_LARGEST_ED25519_S_PLUS_ONE": newRawScalar([]byte{
		0x7e, 0x34, 0x47, 0x75, 0x47, 0x4a, 0x7f, 0x97,
		0x23, 0xb6, 0x3a, 0x8b, 0xe9, 0x2a, 0xe7, 0x6d,
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0f,
	}),

	"CANONICAL_LARGEST_ED25519_S_MINUS_ONE": newRawScalar([]byte{
		0x7c, 0x34, 0x47, 0x75, 0x47, 0x4a, 0x7f, 0x97,
		0x23, 0xb6, 0x3a, 0x8b, 0xe9, 0x2a, 0xe7, 0x6d,
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0f,
	}),
}

var benchBatchSizes = []int{1, 2, 4, 8, 16}

func TestScalar(t *testing.T) {
	t.Run("FuzzerTestcaseReduction", testFuzzerTestcaseReduction)
	t.Run("NonAdjacentForm/TestVector", testNonAdjacentFormTestVector)
	t.Run("NonAdjacentForm/Random", testNonAdjacentFormRandom)
	t.Run("QuarkslabScalarOverflowDoesNotOccur", testOverflowDoesNotOccur)
	t.Run("Add", testAdd)
	t.Run("Add/Reduces", testAddReduces)
	t.Run("Sub/Reduces", testSubReduces)
	t.Run("Neg/TwiceIsIdentity", testNegTwiceIsIdentity)
	t.Run("Mul", testMul)
	t.Run("Mul/ByOne", testMulByOne)
	t.Run("Product", testProduct)
	t.Run("Sum", testSum)
	t.Run("Square", testSquare)
	t.Run("Reduce", testReduce)
	t.Run("FromUint64", testFromUint64)
	t.Run("FromBytesModOrderWide", testFromBytesModOrderWide)
	t.Run("CanonicalDecoding", testCanonicalDecoding)
	t.Run("Invert", testInvert)
	t.Run("BatchInvert/Empty", testBatchInvertEmpty)
	t.Run("BatchInvert/Consistency", testBatchInvertConsistency)
	t.Run("PippengerRadix", testPippengerRadix)
	t.Run("ScMinimalVartime", testScMinimalVartime)
}

func testFuzzerTestcaseReduction(t *testing.T) {
	// LE bytes of 24519928653854221733733552434404946937899825954937634815
	aBytes := []byte{
		255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	}
	// LE bytes of 4975441334397345751130612518500927154628011511324180036903450236863266160640
	bBytes := []byte{
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 210, 210, 210, 255, 255, 255, 255, 10,
	}
	// LE bytes of 6432735165214683820902750800207468552549813371247423777071615116673864412038
	cBytes := []byte{
		134, 171, 119, 216, 180, 128, 178, 62, 171, 132, 32, 62, 34, 119, 104, 193, 47, 215, 181, 250, 14, 207, 172, 93, 75, 207, 211, 103, 144, 204, 56, 14,
	}

	a, err := NewFromBytesModOrder(aBytes)
	if err != nil {
		t.Fatalf("NewFromBytesModOrder(aBytes): %v", err)
	}
	b, err := NewFromBytesModOrder(bBytes)
	if err != nil {
		t.Fatalf("NewFromBytesModOrder(bBytes): %v", err)
	}
	c, err := NewFromBytesModOrder(cBytes)
	if err != nil {
		t.Fatalf("NewFromBytesModOrder(cBytes): %v", err)
	}

	var tmp [64]byte

	// also_a = (a mod l)
	copy(tmp[0:32], aBytes)
	alsoA, err := NewFromBytesModOrderWide(tmp[:])
	if err != nil {
		t.Fatalf("NewFromBytesModOrderWide(tmp) (alsoA): %v", err)
	}

	// also_b = (b mod l)
	copy(tmp[0:32], bBytes)
	alsoB, err := NewFromBytesModOrderWide(tmp[:])
	if err != nil {
		t.Fatalf("NewFromBytesModOrderWide(tmp) (alsoB): %v", err)
	}

	expectedC := New().Mul(a, b)
	alsoExpectedC := New().Mul(alsoA, alsoB)

	if c.Equal(expectedC) != 1 {
		t.Fatalf("C != expectedC (Got %v)", expectedC)
	}
	if c.Equal(alsoExpectedC) != 1 {
		t.Fatalf("C != alsoExpectedC (Got %v)", alsoExpectedC)
	}
}

func testNonAdjacentFormTestVector(t *testing.T) {
	aScalar := newRawScalar([]byte{
		0x1a, 0x0e, 0x97, 0x8a, 0x90, 0xf6, 0x62, 0x2d,
		0x37, 0x47, 0x02, 0x3f, 0x8a, 0xd8, 0x26, 0x4d,
		0xa7, 0x58, 0xaa, 0x1b, 0x88, 0xe0, 0x40, 0xd1,
		0x58, 0x9e, 0x7b, 0x7f, 0x23, 0x76, 0xef, 0x09,
	})
	aNaf := [256]int8{
		0, 13, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, -9, 0, 0, 0, 0, -11, 0, 0, 0, 0, 3, 0, 0, 0, 0, 1,
		0, 0, 0, 0, 9, 0, 0, 0, 0, -5, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 11, 0, 0, 0, 0, 11, 0, 0, 0, 0, 0,
		-9, 0, 0, 0, 0, 0, -3, 0, 0, 0, 0, 9, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 9, 0,
		0, 0, 0, -15, 0, 0, 0, 0, -7, 0, 0, 0, 0, -9, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 13, 0, 0, 0, 0, 0, -3, 0,
		0, 0, 0, -11, 0, 0, 0, 0, -7, 0, 0, 0, 0, -13, 0, 0, 0, 0, 11, 0, 0, 0, 0, -9, 0, 0, 0, 0, 0, 1, 0, 0,
		0, 0, 0, -15, 0, 0, 0, 0, 1, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 13, 0, 0, 0,
		0, 0, 0, 11, 0, 0, 0, 0, 0, 15, 0, 0, 0, 0, 0, -9, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 7,
		0, 0, 0, 0, 0, -15, 0, 0, 0, 0, 0, 15, 0, 0, 0, 0, 15, 0, 0, 0, 0, 15, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0,
	}

	naf := aScalar.NonAdjacentForm(5)
	if naf != aNaf {
		t.Fatalf("aScalar.NonAdjacentForm(5) != aNaf (Got %v)", naf)
	}
}

func testNonAdjacentFormRandom(t *testing.T) {
	for i := 0; i < 1000; i++ {
		var x Scalar
		if _, err := x.SetRandom(nil); err != nil {
			t.Fatalf("x.Random(nil) failed: %v", err)
		}
		for _, w := range []uint{5, 6, 7, 8} {
			testNonAdjacentFormIter(t, w, &x)
		}
	}
}

func testNonAdjacentFormIter(t *testing.T, w uint, x *Scalar) {
	naf := x.NonAdjacentForm(w)

	// Reconstruct the scalar from the computed NAF
	var y Scalar
	for i := 255; i >= 0; i-- {
		var digit Scalar
		y.Add(&y, &y)
		if naf[i] < 0 {
			digit.SetUint64(uint64(-naf[i]))
			digit.Neg(&digit)
		} else {
			digit.SetUint64(uint64(naf[i]))
		}
		y.Add(&y, &digit)
	}

	if x.Equal(&y) != 1 {
		t.Fatalf("x != y (Got %v, %v)", x, y)
	}
}

func testFromUint64(t *testing.T) {
	const val uint64 = 0xdeadbeeffeedface
	expectedBytes := [32]byte{0xce, 0xfa, 0xed, 0xfe, 0xef, 0xbe, 0xad, 0xde}

	s := NewFromUint64(val)
	if s.inner != expectedBytes {
		t.Fatalf("s.FromUint64(%x) (Got %v)", val, s)
	}
}

func testMulByOne(t *testing.T) {
	x, one := testConstants["X"], One()

	testScalar := New().Mul(x, one)
	if testScalar.Equal(x) != 1 {
		t.Fatalf("x * 1 != x (Got %v)", testScalar)
	}
}

func testAddReduces(t *testing.T) {
	s, sPlusOne, one := testConstants["LARGEST_ED25519_S"],
		testConstants["CANONICAL_LARGEST_ED25519_S_PLUS_ONE"],
		One()

	// Check that the addition works
	res := New().Add(s, one)
	res.Reduce(res)
	if res.Equal(sPlusOne) != 1 {
		t.Fatalf("Reduce(s + 1) != s + 1 (Got %v)", res)
	}

	// Check that the addition reduces
	res.Add(s, one)
	if res.Equal(sPlusOne) != 1 {
		t.Fatalf("s + 1 != s  + 1 (Got %v)", res)
	}
}

func testSubReduces(t *testing.T) {
	s, sMinusOne, one := testConstants["LARGEST_ED25519_S"],
		testConstants["CANONICAL_LARGEST_ED25519_S_MINUS_ONE"],
		One()

	// Check that the subtraction works
	res := New().Sub(s, one)
	res.Reduce(res)
	if res.Equal(sMinusOne) != 1 {
		t.Fatalf("Reduce(s - 1) != s - 1 (Got %v)", res)
	}

	// Check that the subtraction reduces
	res.Sub(s, one)
	if res.Equal(sMinusOne) != 1 {
		t.Fatalf("s - 1 != s  + 1 (Got %v)", res)
	}
}

func testOverflowDoesNotOccur(t *testing.T) {
	// Check that manually-constructing large Scalars with
	// FromBits cannot produce incorrect results.
	//
	// The FromBits function is required to implement X/Ed25519,
	// while all other methods of constructing a Scalar produce
	// reduced Scalars.  However, this "invariant loophole" allows
	// constructing large scalars which are not reduced mod l.
	//
	// This issue was discovered independently by both Jack
	// "str4d" Grigg (issue #238), who noted that reduction was
	// not performed on addition, and Laurent Grémy & Nicolas
	// Surbayrole of Quarkslab, who noted that it was possible to
	// cause an overflow and compute incorrect results.
	//
	// This test is adapted from the one suggested by Quarkslab.

	largeBytes := []byte{
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f,
	}

	a, err := NewFromBytesModOrder(largeBytes)
	if err != nil {
		t.Fatalf("NewFromBytesModOrder(largeBytes): %v", err)
	}
	b, err := NewFromBits(largeBytes)
	if err != nil {
		t.Fatalf("NewFromBits(largeBytes): %v", err)
	}

	bReduced := New().Reduce(b)
	if a.Equal(bReduced) != 1 {
		t.Fatalf("a != Reduce(b) (Got %v, %v)", a, bReduced)
	}

	add3x := func(v *Scalar) *Scalar {
		res := New().Add(v, v)
		res.Add(res, v)
		return res
	}
	a3 := add3x(a)
	b3 := add3x(b)
	if a3.Equal(b3) != 1 {
		t.Fatalf("a + a + a != b + b + b (Got %v, %v)", a3, b3)
	}

	negA := New().Neg(a)
	negB := New().Neg(b)
	if negA.Equal(negB) != 1 {
		t.Fatalf("-a != -b (Got %v, %v)", negA, negB)
	}

	sub3x := func(v *Scalar) *Scalar {
		res := New().Sub(New(), v)
		res.Sub(res, v)
		res.Sub(res, v)
		return res
	}
	minusA3 := sub3x(a)
	minusB3 := sub3x(b)
	if minusA3.Equal(minusB3) != 1 {
		t.Fatalf("- a - a - a != - b - b - b (Got %v, %v)", minusA3, minusB3)
	}

	negA3 := New().Neg(a3)
	negB3 := New().Neg(b3)
	if minusA3.Equal(negA3) != 1 {
		t.Fatalf("- a -a -a != -(a + a + a) (Got %v, %v)", minusA3, negA3)
	}
	if minusB3.Equal(negB3) != 1 {
		t.Fatalf("- b -b -b != -(b + b + b) (Got %v, %v)", minusB3, negB3)
	}
}

func testAdd(t *testing.T) {
	one, two := One(), NewFromUint64(2)

	shouldBeTwo := New().Add(one, one)
	if shouldBeTwo.Equal(two) != 1 {
		t.Fatalf("1 + 1 != 2 (Got %v)", shouldBeTwo)
	}
}

func testMul(t *testing.T) {
	x, y, xy := testConstants["X"], testConstants["Y"], testConstants["XY"]

	var shouldBeXTimesY Scalar
	shouldBeXTimesY.Mul(x, y)
	if shouldBeXTimesY.Equal(xy) != 1 {
		t.Fatalf("x * y != xy (Got %v)", shouldBeXTimesY)
	}
}

func testProduct(t *testing.T) {
	// Test that product works for non-empty slices
	xy := testConstants["XY"]

	shouldBeXTimesY := New().Product([]*Scalar{
		testConstants["X"],
		testConstants["Y"],
	})
	if shouldBeXTimesY.Equal(xy) != 1 {
		t.Fatalf("Product([x, y]) != xy (Got %v)", shouldBeXTimesY)
	}

	// Test that product works for the empty slice
	shouldBeOne := New().Product([]*Scalar{})
	if shouldBeOne.Equal(One()) != 1 {
		t.Fatalf("Product([]) != 1 (Got %v)", shouldBeOne)
	}
	shouldBeOne.Product(nil)
	if shouldBeOne.Equal(One()) != 1 {
		t.Fatalf("Product(nil) != 1 (Got %v)", shouldBeOne)
	}
}

func testSum(t *testing.T) {
	// Test that sum works for non-empty slices
	one, two := One(), NewFromUint64(2)

	shouldBeTwo := New().Sum([]*Scalar{one, one})
	if shouldBeTwo.Equal(two) != 1 {
		t.Fatalf("Sum([1, 1]) != 2 (Got %v)", shouldBeTwo)
	}

	// Test that sum works for empty slices
	shouldBeZero := New().Sum([]*Scalar{})
	if shouldBeZero.Equal(New()) != 1 {
		t.Fatalf("Sum([]) != 0 (Got %v)", shouldBeZero)
	}
	shouldBeZero.Sum(nil)
	if shouldBeZero.Equal(New()) != 1 {
		t.Fatalf("Sum(nil) != 0 (Got %v)", shouldBeZero)
	}
}

func testSquare(t *testing.T) {
	x := testConstants["X"]

	expected := New().Mul(x, x)

	actual := New().pack(newUnpackedScalar().Square(x.unpack()))
	if expected.Equal(actual) != 1 {
		t.Fatalf("x.Square() != x * x (Got %v)", actual)
	}
}

func testReduce(t *testing.T) {
	// sage: l = 2^252 + 27742317777372353535851937790883648493
	// sage: big = 2^256 - 1
	// sage: repr((big % l).digits(256))
	canonical_2_256_minus_1 := newRawScalar([]byte{
		28, 149, 152, 141, 116, 49, 236, 214,
		112, 207, 125, 115, 244, 91, 239, 198,
		254, 255, 255, 255, 255, 255, 255, 255,
		255, 255, 255, 255, 255, 255, 255, 15,
	})

	biggest, err := NewFromBytesModOrder([]byte{
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
	})
	if err != nil {
		t.Fatalf("NewFromBytesModOrder([0xff...]): %v", err)
	}
	if biggest.Equal(canonical_2_256_minus_1) != 1 {
		t.Fatalf("unexpected biggest: %v", biggest)
	}
}

func testFromBytesModOrderWide(t *testing.T) {
	x := testConstants["X"]
	var bignum [ScalarWideSize]byte
	// set bignum = x + 2^256x
	for i, v := range x.inner {
		bignum[i] = v
		bignum[32+i] = v
	}
	// 3958878930004874126169954872055634648693766179881526445624823978500314864344
	// = x + 2^256x (mod l)
	expected := newRawScalar([]byte{
		216, 154, 179, 139, 210, 121, 2, 71,
		69, 99, 158, 216, 23, 173, 63, 100,
		204, 0, 91, 50, 219, 153, 57, 249,
		28, 82, 31, 197, 100, 165, 192, 8,
	})
	reduced, err := NewFromBytesModOrderWide(bignum[:])
	if err != nil {
		t.Fatalf("NewFromBytesModOrderWide(bignum): %v", err)
	}
	if reduced.Equal(expected) != 1 {
		t.Fatalf("unexpected reduced: %v", reduced)
	}

	// Also test that FromBytesModOrderWide matches the montgomery reduction.
	// The original code has this split into two tests, but the latter test
	// replicates the former.

	unpackedBignum, err := newUnpackedScalar().SetBytesWide(bignum[:])
	if err != nil {
		t.Fatalf("SetBytesWide(bignum): %v", err)
	}

	//  (x + 2^256x) * R
	interim := scalarMulInternal(unpackedBignum, &constR)

	// ((x + 2^256x) * R) / R  (mod l)
	montgomeryReduced := newUnpackedScalar().MontgomeryReduce(&interim)

	// The Montgomery reduced scalar should match the reduced one, as well as the expected
	unpackedReduced, unpackedExpected := reduced.unpack(), expected.unpack()
	if *montgomeryReduced != *unpackedReduced {
		t.Fatalf("montgomery_reduced != reduced (Got: %v)", montgomeryReduced)
	}
	if *montgomeryReduced != *unpackedExpected {
		t.Fatalf("montgomery_reduced != expected (Got: %v)", montgomeryReduced)
	}
}

func testInvert(t *testing.T) {
	x, xinv, one := testConstants["X"], testConstants["XINV"], One()

	tmp := New().Invert(x)
	if tmp.Equal(xinv) != 1 {
		t.Fatalf("x.Invert() != 1/x (Got: %v)", tmp)
	}

	tmp.Mul(tmp, x)
	if tmp.Equal(one) != 1 {
		t.Fatalf("x * 1/x != 1 (Got : %v)", tmp)
	}
}

func testNegTwiceIsIdentity(t *testing.T) {
	x := testConstants["X"]

	tmp := New().Neg(x)
	if tmp.Equal(x) == 1 {
		t.Fatalf("-x = x")
	}

	tmp.Neg(tmp)
	if tmp.Equal(x) != 1 {
		t.Fatalf("-(-x) != x (Got: %v)", tmp)
	}
}

func testCanonicalDecoding(t *testing.T) {
	// canonical encoding of 1667457891
	canonical_bytes := []byte{99, 99, 99, 99, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}

	// encoding of
	//   7265385991361016183439748078976496179028704920197054998554201349516117938192
	// = 28380414028753969466561515933501938171588560817147392552250411230663687203 (mod l)
	// non_canonical because unreduced mod l
	non_canonical_bytes_because_unreduced := []byte{
		16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16,
		16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16,
	}

	// encoding with high bit set, to check that the parser isn't pre-masking the high bit
	non_canonical_bytes_because_highbit := []byte{
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128,
	}

	var tmp Scalar
	if _, err := tmp.SetCanonicalBytes(canonical_bytes); err != nil {
		t.Fatalf("SetCanonicalBytes(canonical_bytes): %v", err)
	}
	if _, err := tmp.SetCanonicalBytes(non_canonical_bytes_because_unreduced); err == nil {
		t.Fatalf("SetCanonicalBytes(non_canonical_bytes_because_unreduced)")
	}
	if _, err := tmp.SetCanonicalBytes(non_canonical_bytes_because_highbit); err == nil {
		t.Fatalf("SetCanonicalBytes(non_canonical_bytes_because_highbit)")
	}

	// While we're here, test IsCanonical().
	copy(tmp.inner[:], non_canonical_bytes_because_unreduced)
	if tmp.IsCanonical() {
		t.Fatalf("non_canonical_bytes_because_unreduced.IsCanonical() == true")
	}
	copy(tmp.inner[:], non_canonical_bytes_because_highbit)
	if tmp.IsCanonical() {
		t.Fatalf("non_canonical_bytes_because_highbit.IsCanonical() == true")
	}
}

func testBatchInvertEmpty(t *testing.T) {
	one := One()

	tmp := New().BatchInvert(nil)
	if tmp.Equal(one) != 1 {
		t.Fatalf("BatchInvert(nil) != 1 (Got: %v)", tmp)
	}

	tmp.BatchInvert([]*Scalar{})
	if tmp.Equal(one) != 1 {
		t.Fatalf("BatchInvert([]Scalar{}) != 1 (Got: %v)", tmp)
	}
}

func testBatchInvertConsistency(t *testing.T) {
	one, x := One(), One()

	var v1, v2 []*Scalar
	for i := 0; i < 16; i++ {
		tmp1, tmp2 := *x, *x
		x.Add(x, x)

		v1 = append(v1, &tmp1)
		v2 = append(v2, &tmp2)
	}

	var expected Scalar
	expected.Product(v1)
	expected.Invert(&expected)

	var ret Scalar
	ret.BatchInvert(v1)
	if ret.Equal(&expected) != 1 {
		t.Fatalf("BatchInvert(v1) (Got: %v)", ret)
	}

	for i := range v1 {
		var tmp Scalar
		tmp.Mul(v1[i], v2[i])
		if tmp.Equal(one) != 1 {
			t.Fatalf("a * b != 1 (Got %v)", tmp)
		}
	}
}

func testPippengerRadixIter(t *testing.T, s *Scalar, w uint) {
	digitsCount := ToRadix2wSizeHint(w)
	digits := s.ToRadix2w(w)

	radix, term := NewFromUint64(uint64(1<<w)), One()

	var recovered Scalar
	for _, digit := range digits[0:digitsCount] {
		if digit != 0 {
			var sDigit Scalar
			if digit < 0 {
				sDigit.SetUint64(uint64(-int64(digit)))
				sDigit.Neg(&sDigit)
			} else {
				sDigit.SetUint64(uint64(digit))
			}
			tmp := New().Mul(term, &sDigit)
			recovered.Add(&recovered, tmp)
		}
		term.Mul(term, radix)
	}

	scalar := New().Reduce(s)
	if recovered.Equal(scalar) != 1 {
		t.Fatalf("recovered != scalar (Got %v, %v)", recovered, scalar)
	}
}

func testPippengerRadix(t *testing.T) {
	var cases []*Scalar
	for i := uint64(2); i < 100; i++ {
		s := NewFromUint64(i)
		s.Invert(s)
		cases = append(cases, s)
	}

	biggest, err := NewFromBits([]byte{
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
	})
	if err != nil {
		t.Fatalf("NewFromBits([0xff..]): %v", err)
	}
	cases = append(cases, biggest)

	for _, s := range cases {
		for _, w := range []uint{6, 7, 8} {
			testPippengerRadixIter(t, s, w)
		}
	}
}

func testScMinimalVartime(t *testing.T) {
	// At this point I could have just left this hardcoded as in the
	// Go standard library, but parsing out BASEPOINT_ORDER is probably
	// better.
	expectedOrder := [4]uint64{0x5812631a5cf5d3ed, 0x14def9dea2f79cd6, 0, 0x1000000000000000}
	if order != expectedOrder {
		t.Fatalf("invalid deserialized BASEPOINT_ORDER: Got %v", order)
	}

	// External review caught an early version of the fail-fast check
	// that had a bug due to an incorrect constant.  Run through a few
	// test cases to make sure that everything is ok.
	for _, v := range []struct {
		scalarHex string
		expected  bool
	}{
		{"0000000000000000000000000000000000000000000000000000000000000010", true},  // From review
		{"ddd3f55c1a631258d69cf7a2def9de1400000000000000000000000000000010", true},  // Order - 1
		{"edd3f55c1a631258d69cf7a2def9de1400000000000000000000000000000010", false}, // Order
		{"0000000000000000000000000000000000000000000000000000000000000020", false},
		{"0000000000000000000000000000000000000000000000000000000000000040", false},
		{"0000000000000000000000000000000000000000000000000000000000000080", false},
	} {
		b := testhelpers.MustUnhex(t, v.scalarHex)
		if ScMinimalVartime(b) != v.expected {
			t.Fatalf("ScMinimalVartime(%s) != %v", v.scalarHex, v.expected)
		}
	}
}

func BenchmarkScalar(b *testing.B) {
	b.Run("Invert", benchScalarInvert)
	b.Run("BatchInvert", benchScalarBatchInvert)
}

func benchScalarInvert(b *testing.B) {
	s := NewFromUint64(897987897)

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		s.Invert(s)
	}
}

func benchScalarBatchInvertIter(b *testing.B, n int) {
	scalars := func() []*Scalar {
		v := make([]*Scalar, 0, n)
		for i := 0; i < n; i++ {
			s, err := New().SetRandom(rand.Reader)
			if err != nil {
				b.Fatalf("New().Random(): %v", err)
			}
			v = append(v, s)
		}
		return v
	}()

	b.ResetTimer()

	var s Scalar
	for i := 0; i < b.N; i++ {
		s.BatchInvert(scalars)
	}
}

func benchScalarBatchInvert(b *testing.B) {
	for _, n := range benchBatchSizes {
		b.Run(strconv.Itoa(n), func(b *testing.B) {
			b.ReportAllocs()
			benchScalarBatchInvertIter(b, n)
		})
	}
}

func newRawScalar(vec []byte) *Scalar {
	if len(vec) != ScalarSize {
		panic("curve/scalar: invalid constant vector")
	}

	// Note: This will happily create non-canonical scalars, which is fine
	// because this is only used to define constants (e.g. L), and for
	// testing.
	s := New()
	copy(s.inner[:], vec)
	return s
}
