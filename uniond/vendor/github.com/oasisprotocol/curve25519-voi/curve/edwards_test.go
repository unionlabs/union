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
	"crypto/rand"
	"reflect"
	"testing"

	"github.com/oasisprotocol/curve25519-voi/curve/scalar"
	"github.com/oasisprotocol/curve25519-voi/internal/field"
)

var edwardsPointTestIdentity = func() *EdwardsPoint {
	var p EdwardsPoint
	return p.Identity()
}()

var edwardsPointTestPoints = map[string]*CompressedEdwardsY{
	// Compressed Edwards Y form of 2*basepoint.
	"BASE2": {
		0xc9, 0xa3, 0xf8, 0x6a, 0xae, 0x46, 0x5f, 0xe,
		0x56, 0x51, 0x38, 0x64, 0x51, 0x0f, 0x39, 0x97,
		0x56, 0x1f, 0xa2, 0xc9, 0xe8, 0x5e, 0xa2, 0x1d,
		0xc2, 0x29, 0x23, 0x09, 0xf3, 0xcd, 0x60, 0x22,
	},

	// Compressed Edwards Y form of 16*basepoint.
	"BASE16": {
		0xeb, 0x27, 0x67, 0xc1, 0x37, 0xab, 0x7a, 0xd8,
		0x27, 0x9c, 0x07, 0x8e, 0xff, 0x11, 0x6a, 0xb0,
		0x78, 0x6e, 0xad, 0x3a, 0x2e, 0x0f, 0x98, 0x9f,
		0x72, 0xc3, 0x7f, 0x82, 0xf2, 0x96, 0x96, 0x70,
	},

	// A_SCALAR * basepoint, computed with ed25519.py
	"A_TIMES_BASEPOINT": {
		0xea, 0x27, 0xe2, 0x60, 0x53, 0xdf, 0x1b, 0x59,
		0x56, 0xf1, 0x4d, 0x5d, 0xec, 0x3c, 0x34, 0xc3,
		0x84, 0xa2, 0x69, 0xb7, 0x4c, 0xc3, 0x80, 0x3e,
		0xa8, 0xe2, 0xe7, 0xc9, 0x42, 0x5e, 0x40, 0xa5,
	},

	// A_SCALAR * (A_TIMES_BASEPOINT) + B_SCALAR * BASEPOINT
	// computed with ed25519.py
	"DOUBLE_SCALAR_MULT_RESULT": {
		0x7d, 0xfd, 0x6c, 0x45, 0xaf, 0x6d, 0x6e, 0x0e,
		0xba, 0x20, 0x37, 0x1a, 0x23, 0x64, 0x59, 0xc4,
		0xc0, 0x46, 0x83, 0x43, 0xde, 0x70, 0x4b, 0x85,
		0x09, 0x6f, 0xfe, 0x35, 0x4f, 0x13, 0x2b, 0x42,
	},

	"ED25519_BASEPOINT": ED25519_BASEPOINT_COMPRESSED,

	"COMPRESSED_IDENTITY": func() *CompressedEdwardsY {
		var cp CompressedEdwardsY
		cp.Identity()
		return &cp
	}(),
}

var edwardsPointTestScalars = map[string]*scalar.Scalar{
	// 4493907448824000747700850167940867464579944529806937181821189941592931634714
	"A": func() *scalar.Scalar {
		s, _ := scalar.NewFromCanonicalBytes([]byte{
			0x1a, 0x0e, 0x97, 0x8a, 0x90, 0xf6, 0x62, 0x2d,
			0x37, 0x47, 0x02, 0x3f, 0x8a, 0xd8, 0x26, 0x4d,
			0xa7, 0x58, 0xaa, 0x1b, 0x88, 0xe0, 0x40, 0xd1,
			0x58, 0x9e, 0x7b, 0x7f, 0x23, 0x76, 0xef, 0x09,
		})
		return s
	}(),

	// 2506056684125797857694181776241676200180934651973138769173342316833279714961
	"B": func() *scalar.Scalar {
		s, _ := scalar.NewFromCanonicalBytes([]byte{
			0x91, 0x26, 0x7a, 0xcf, 0x25, 0xc2, 0x09, 0x1b,
			0xa2, 0x17, 0x74, 0x7b, 0x66, 0xf0, 0xb3, 0x2e,
			0x9d, 0xf2, 0xa5, 0x67, 0x41, 0xcf, 0xda, 0xc4,
			0x56, 0xa7, 0xd4, 0xaa, 0xb8, 0x60, 0x8a, 0x05,
		})
		return s
	}(),
}

func TestEdwards(t *testing.T) {
	t.Run("Decompression/Compression", testEdwardsDecompressionCompression)
	t.Run("Decompression/SignHandling", testEdwardsDecompressionSignHandling)
	t.Run("Add", testEdwardsAdd)
	t.Run("Add/ProjectiveNiels", testEdwardsAddProjectiveNiels)
	t.Run("Add/AffineNiels", testEdwardsAddAffineNiels)
	t.Run("Equals/HandlesScaling", testEdwardsEqualsHandlesScaling)
	t.Run("Mul", testEdwardsMul)
	t.Run("Sum", testEdwardsSum)
	t.Run("IsSmallOrder", testEdwardsIsSmallOrder)
	t.Run("IsTorsionFree", testEdwardsIsTorsionFree)
	t.Run("IsIdentity", testEdwardsIsIdentity)
	t.Run("CompressedIdentity", testEdwardsCompressedIdentity)
	t.Run("BasepointTable/New", testEdwardsBasepointTableNew)
	t.Run("BasepointTable/Basepoint", testEdwardsBasepointTableBasepoint)
	t.Run("BasepointTable/Mul", testEdwardsBasepointTableMul)
	t.Run("BasepointTable/Mul/One", testEdwardsBasepointTableMulOne)
	t.Run("BasepointTable/Mul/Two", testEdwardsBasepointTableMulTwo)
	t.Run("BasepointTable/Mul/VsEd25519py", testEdwardsBasepointTableMulVsEd25519py)
	t.Run("BasepointTable/Mul/ByBasepointOrder", testEdwardsBasepointTableMulByBasepointOrder)
	t.Run("BasepointPoint/DoubleVsConstant", testEdwardsBasepointPointDoubleVsConstant)
	t.Run("BasepointPoint/ProjectiveExtendedRoundTrip", testEdwardsBasepointPointProjectiveExtendedRoundTrip)
	t.Run("BasepointPoint/16VsMulByPow2_4", testEdwardsBasepointPoint16VsMulByPow2_4)
	t.Run("DoubleScalarMulBasepointVartime", testEdwardsDoubleScalarMulBasepointVartime)
	t.Run("TripleScalarMulBasepointVartime", testEdwardsMulAbglsvPorninVartime)
	t.Run("MultiscalarMul", testEdwardsMultiscalarMul)
	t.Run("MultiscalarMul/Consistency", testEdwardsMultiscalarConsistency)
	t.Run("MultiscalarMulVartime", testEdwardsMultiscalarMulVartime)
	t.Run("MultiscalarMulPippengerVartime", testEdwardsMultiscalarMulPippengerVartime)
	t.Run("AffineNielsPoint/ConditionalAssign", testAffineNielsConditionalAssign)
	t.Run("AffineNielsPoint/ConversionClearsDenominators", testAffineNielsConversionClearsDenominators)
	t.Run("IsCanonicalVartime", testIsCanonicalVartime)
}

func testEdwardsDecompressionCompression(t *testing.T) {
	// X coordinate of the basepoint.
	// = 15112221349535400772501151409588531511454012693041857206046113283949847762202
	BASE_X_COORD_BYTES := []byte{
		0x1a, 0xd5, 0x25, 0x8f, 0x60, 0x2d, 0x56, 0xc9, 0xb2, 0xa7, 0x25, 0x95, 0x60, 0xc7, 0x2c, 0x69,
		0x5c, 0xdc, 0xd6, 0xfd, 0x31, 0xe2, 0xa4, 0xc0, 0xfe, 0x53, 0x6e, 0xcd, 0xd3, 0x36, 0x69, 0x21,
	}

	var baseX field.Element
	if _, err := baseX.SetBytes(BASE_X_COORD_BYTES); err != nil {
		t.Fatalf("baseX.FromBytes(): %v", err)
	}

	var bp EdwardsPoint
	if _, err := bp.SetCompressedY(ED25519_BASEPOINT_COMPRESSED); err != nil {
		t.Fatalf("bp.SetCompressedY(): %v", err)
	}
	if !bp.debugIsValid() {
		t.Fatalf("bp.isDebugValid() != true")
	}

	// Check that decompression actually gives the correct X coordinate.
	if baseX.Equal(&bp.inner.X) != 1 {
		t.Fatalf("baseX != bp.X (Got: %v)", bp.inner.X)
	}
	var recompressed CompressedEdwardsY
	recompressed.SetEdwardsPoint(&bp)
	if recompressed.Equal(ED25519_BASEPOINT_COMPRESSED) != 1 {
		t.Fatalf("recompressed != ED25519_BASEPOINT_COMPRESSED")
	}
}

func testEdwardsDecompressionSignHandling(t *testing.T) {
	// Manually set the high bit of the last byte to flip the sign.
	var minusBasepointBytes CompressedEdwardsY
	copy(minusBasepointBytes[:], ED25519_BASEPOINT_COMPRESSED[:])
	minusBasepointBytes[31] |= 1 << 7
	var minusBasepoint EdwardsPoint
	if _, err := minusBasepoint.SetCompressedY(&minusBasepointBytes); err != nil {
		t.Fatalf("minusBasepoint.SetCompressedY(): %v", err)
	}

	// Test projective coordinates exactly since we know they should
	// only differ by a flipped sign.
	var negX, negT field.Element
	negX.Neg(&ED25519_BASEPOINT_POINT.inner.X)
	negT.Neg(&ED25519_BASEPOINT_POINT.inner.T)
	if minusBasepoint.inner.X.Equal(&negX) != 1 {
		t.Fatalf("minusBasepoint.X != -ED25519_BASEPOINT_POINT.X (Got: %v)", minusBasepoint.inner.X)
	}
	if minusBasepoint.inner.Y.Equal(&ED25519_BASEPOINT_POINT.inner.Y) != 1 {
		t.Fatalf("minusBasepoint.Y != -ED25519_BASEPOINT_POINT.Y (Got: %v)", minusBasepoint.inner.Y)
	}
	if minusBasepoint.inner.Z.Equal(&ED25519_BASEPOINT_POINT.inner.Z) != 1 {
		t.Fatalf("minusBasepoint.Z != -ED25519_BASEPOINT_POINT.Z (Got: %v)", minusBasepoint.inner.Z)
	}
	if minusBasepoint.inner.T.Equal(&negT) != 1 {
		t.Fatalf("minusBasepoint.T != -ED25519_BASEPOINT_COMPRESSED.T (Got: %v)", minusBasepoint.inner.T)
	}
}

func testEdwardsAdd(t *testing.T) {
	bp := ED25519_BASEPOINT_POINT
	var sum EdwardsPoint
	sum.Add(bp, bp)
	if !sum.testEqualCompressedY("BASE2") {
		t.Fatalf("bp + bp != BASE2 (Got: %v)", sum)
	}
}

func testEdwardsAddProjectiveNiels(t *testing.T) {
	var (
		bpPNiels     projectiveNielsPoint
		sumCompleted completedPoint
		sum          EdwardsPoint
	)
	bp := ED25519_BASEPOINT_POINT
	sum.setCompleted(sumCompleted.AddEdwardsProjectiveNiels(bp, bpPNiels.SetEdwards(bp)))
	if !sum.testEqualCompressedY("BASE2") {
		t.Fatalf("bp + toProjectiveNiels(bp) != BASE2 (Got: %v)", sum)
	}
}

func testEdwardsAddAffineNiels(t *testing.T) {
	var (
		bpANiels     affineNielsPoint
		sumCompleted completedPoint
		sum          EdwardsPoint
	)
	bp := ED25519_BASEPOINT_POINT
	sum.setCompleted(sumCompleted.AddEdwardsAffineNiels(bp, bpANiels.SetEdwards(bp)))
	if !sum.testEqualCompressedY("BASE2") {
		t.Fatalf("bp + toAffineNiels(bp) != BASE2 (Got: %v)", sum)
	}
}

func testEdwardsEqualsHandlesScaling(t *testing.T) {
	var id1 EdwardsPoint
	id1.Identity()

	var id2 EdwardsPoint
	id2.inner.Y.Set(&field.Two)
	id2.inner.Z.Set(&field.Two)
	if id1.Equal(&id2) != 1 {
		t.Fatalf("id1 != id2")
	}
}

func testEdwardsMul(t *testing.T) {
	var aB EdwardsPoint
	aB.Mul(ED25519_BASEPOINT_POINT, edwardsPointTestScalars["A"])
	if !aB.testEqualCompressedY("A_TIMES_BASEPOINT") {
		t.Fatalf("a * B != A_TIMES_BASEPOINT (Got: %v)", aB)
	}
}

func testEdwardsSum(t *testing.T) {
	base := ED25519_BASEPOINT_POINT

	s1, s2 := scalar.NewFromUint64(999), scalar.NewFromUint64(333)

	var p1, p2, expected EdwardsPoint
	p1.Mul(base, s1)
	p2.Mul(base, s2)
	expected.Add(&p1, &p2)

	var sum EdwardsPoint
	sum.Sum([]*EdwardsPoint{&p1, &p2})

	if sum.Equal(&expected) != 1 {
		t.Fatalf("Sum({p1, p2}) != expected (Got: %v)", sum)
	}

	// Test that sum works with an empty slice.
	expected.Identity()
	sum.Sum([]*EdwardsPoint{})
	if sum.Equal(&expected) != 1 {
		t.Fatalf("Sum({}) != identity (Got: %v)", sum)
	}

	// Test that sum works with a nil slice.
	sum.Sum(nil)
	if sum.Equal(&expected) != 1 {
		t.Fatalf("Sum({}) != identity (Got: %v)", sum)
	}
}

func testEdwardsIsSmallOrder(t *testing.T) {
	// The basepoint has large prime order.
	if ED25519_BASEPOINT_POINT.IsSmallOrder() {
		t.Fatalf("ED25519_BASEPOINT_POINT.IsSmallOrder() != false")
	}
	// EIGHT_TORSION has all points of small order.
	for _, torsionPoint := range EIGHT_TORSION {
		if !torsionPoint.IsSmallOrder() {
			t.Fatalf("EIGHT_TORSION.IsSmallOrder() != true")
		}
	}
}

func testEdwardsIsTorsionFree(t *testing.T) {
	// The basepoint is torsion-free.
	if !ED25519_BASEPOINT_POINT.IsTorsionFree() {
		t.Fatalf("ED25519_BASEPOINT_POINT.IsTorsionFree() != true")
	}

	// The basepoint + generator of the torsion subgroup is not torsion-free.
	var sum EdwardsPoint
	sum.Add(ED25519_BASEPOINT_POINT, EIGHT_TORSION[1])
	if sum.IsTorsionFree() {
		t.Fatalf("(P + Q).IsTorsionFree() != false")
	}
}

func testEdwardsIsIdentity(t *testing.T) {
	if !edwardsPointTestIdentity.IsIdentity() {
		t.Fatalf("p.IsIdentity() != true")
	}
	if ED25519_BASEPOINT_POINT.IsIdentity() {
		t.Fatalf("ED25519_BASEPOINT_POINT.IsIdentity() == true")
	}
}

func testEdwardsCompressedIdentity(t *testing.T) {
	if !edwardsPointTestIdentity.testEqualCompressedY("COMPRESSED_IDENTITY") {
		t.Fatalf("p.Identity().compress() != COMPRESSED_IDENTITY (Got: %v)", edwardsPointTestIdentity)
	}
}

func testEdwardsBasepointTableNew(t *testing.T) {
	// Test table creation by regenerating the hard coded basepoint table.
	// This also serves to sanity-check that the hardcoded table is correct.
	tbl := NewEdwardsBasepointTable(ED25519_BASEPOINT_POINT)
	switch supportsVectorizedEdwards {
	case false:
		for i, subTbl := range tbl.inner {
			for ii, pt := range subTbl {
				expectedPt := ED25519_BASEPOINT_TABLE.inner[i][ii]
				if !pt.testEqual(&expectedPt) {
					t.Fatalf("tbl[%d][%d] != ED25519_BASEPOINT_TABLE[%d][%d] (Got: %v)", i, ii, i, ii, pt)
				}
			}
		}
	case true:
		if !reflect.DeepEqual(tbl, ED25519_BASEPOINT_TABLE) {
			t.Fatalf("NewEdwardsBasepointTable(ED25519_BASEPOINT_POINT) != ED25519_BASEPOINT_TABLE")
		}
	}
}

func testEdwardsBasepointTableBasepoint(t *testing.T) {
	bp := ED25519_BASEPOINT_TABLE.Basepoint()
	if !bp.testEqualCompressedY("ED25519_BASEPOINT") {
		t.Fatalf("ED25519_BASEPOINT_TABLE.Basepoint() != ED25519_BASEPOINT (Got: %v)", bp)
	}
}

func testEdwardsBasepointTableMul(t *testing.T) {
	var aB_1, aB_2 EdwardsPoint
	aB_1.MulBasepoint(ED25519_BASEPOINT_TABLE, edwardsPointTestScalars["A"])
	aB_2.Mul(ED25519_BASEPOINT_POINT, edwardsPointTestScalars["A"])
	if aB_1.Equal(&aB_2) != 1 {
		t.Fatalf("aB_1 != aB_2 (Got %v %v)", aB_1, aB_2)
	}
}

func testEdwardsBasepointTableMulOne(t *testing.T) {
	var bp EdwardsPoint
	bp.MulBasepoint(ED25519_BASEPOINT_TABLE, scalar.One())
	if !bp.testEqualCompressedY("ED25519_BASEPOINT") {
		t.Fatalf("ED25519_BASEPOINT_TABLE.Mul(1) != ED25519_BASEPOINT (Got: %v)", bp)
	}
}

func testEdwardsBasepointTableMulVsEd25519py(t *testing.T) {
	var aB EdwardsPoint
	aB.MulBasepoint(ED25519_BASEPOINT_TABLE, edwardsPointTestScalars["A"])
	if !aB.testEqualCompressedY("A_TIMES_BASEPOINT") {
		t.Fatalf("ED25519_BASEPOINT_TABLE.Mul(a) != A_TIMES_BASEPOINT (Got: %v)", aB)
	}
}

func testEdwardsBasepointTableMulByBasepointOrder(t *testing.T) {
	var shouldBeId EdwardsPoint
	shouldBeId.MulBasepoint(ED25519_BASEPOINT_TABLE, scalar.BASEPOINT_ORDER)
	if !shouldBeId.IsIdentity() {
		t.Fatalf("ED25519_BASEPOINT_TABLE.Mul(BASEPOINT_ORDER).IsIdentity() != true")
	}
}

func testEdwardsBasepointTableMulTwo(t *testing.T) {
	var bp2 EdwardsPoint
	bp2.MulBasepoint(ED25519_BASEPOINT_TABLE, scalar.NewFromUint64(2))
	if !bp2.testEqualCompressedY("BASE2") {
		t.Fatalf("ED25519_BASEPOINT_TABLE.Mul(2) != BASE2 (Got: %v)", bp2)
	}
}

func testEdwardsBasepointPointDoubleVsConstant(t *testing.T) {
	var p EdwardsPoint
	p.double(ED25519_BASEPOINT_POINT)
	if !p.testEqualCompressedY("BASE2") {
		t.Fatalf("bp.double() != BASE2 (Got: %v)", p)
	}
}

func testEdwardsBasepointPointProjectiveExtendedRoundTrip(t *testing.T) {
	var (
		pProjective projectivePoint
		pp          EdwardsPoint
	)
	pProjective.SetEdwards(ED25519_BASEPOINT_POINT)
	pp.setProjective(&pProjective)
	if !pp.testEqualCompressedY("ED25519_BASEPOINT") {
		t.Fatalf("bp->projective->extended != bp (Got: %v)", pp)
	}
}

func testEdwardsBasepointPoint16VsMulByPow2_4(t *testing.T) {
	var bp16 EdwardsPoint
	bp16.mulByPow2(ED25519_BASEPOINT_POINT, 4)
	if !bp16.testEqualCompressedY("BASE16") {
		t.Fatalf("bp.mulByPow2(4) != BASE16 (Got: %v)", bp16)
	}
}

func testEdwardsMultiscalarConsistencyIter(t *testing.T, n int) {
	// Construct random coefficients x0, ..., x_{n-1},
	// followed by some extra hardcoded ones.
	xs := newTestBenchRandomScalars(t, n)
	// The largest scalar allowed by the type system, 2^255-1
	biggest, err := scalar.NewFromBits([]byte{
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
		0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
	})
	if err != nil {
		t.Fatalf("scalar.NewFromBits([0xff..]): %v", err)
	}
	xs = append(xs, biggest)
	var check scalar.Scalar
	for _, xi := range xs {
		tmp := scalar.New().Mul(xi, xi)
		check.Add(&check, tmp)
	}

	// Construct points G_i = x_i * B
	Gs := make([]*EdwardsPoint, 0, n)
	for _, xi := range xs {
		var tmp EdwardsPoint
		Gs = append(Gs, tmp.MulBasepoint(ED25519_BASEPOINT_TABLE, xi))
	}
	expandedGs := make([]*ExpandedEdwardsPoint, 0, n)
	for _, gi := range Gs {
		expandedGs = append(expandedGs, NewExpandedEdwardsPoint(gi))
	}

	var H1, H2, H3, H4 EdwardsPoint
	// Compute H1 = <xs, Gs> (consttime)
	H1.MultiscalarMul(xs, Gs)
	// Compute H2 = <xs, Gs> (vartime)
	H2.MultiscalarMulVartime(xs, Gs)
	// Compute H3 = <xs, expandedGs> + <xs, Gs> (vartime)
	H3.ExpandedMultiscalarMulVartime(xs, expandedGs, xs, Gs)
	// Compute H4 = <xs, Gs> = sum(xi^2) * B
	H4.MulBasepoint(ED25519_BASEPOINT_TABLE, &check)

	if H1.Equal(&H4) != 1 {
		t.Fatalf("H1 != H4 (Got: %v)", H1)
	}
	if H2.Equal(&H4) != 1 {
		t.Fatalf("H2 != H4 (Got: %v)", H2)
	}
	if H3.Equal(H4.double(&H4)) != 1 {
		t.Fatalf("H3 != 2 * H4 (Got: %v)", H3)
	}
}

func testEdwardsMultiscalarConsistency(t *testing.T) {
	t.Run("100", func(t *testing.T) {
		testEdwardsMultiscalarConsistencyIter(t, 100)
	})
	t.Run("250", func(t *testing.T) {
		testEdwardsMultiscalarConsistencyIter(t, 250)
	})
	t.Run("500", func(t *testing.T) {
		testEdwardsMultiscalarConsistencyIter(t, 500)
	})
	t.Run("1000", func(t *testing.T) {
		testEdwardsMultiscalarConsistencyIter(t, 1000)
	})
}

func testEdwardsDoubleScalarMulBasepointVartime(t *testing.T) {
	var A EdwardsPoint
	if _, err := A.SetCompressedY(edwardsPointTestPoints["A_TIMES_BASEPOINT"]); err != nil {
		t.Fatalf("A.SetCompressedY(): %v", err)
	}

	var result EdwardsPoint
	result.DoubleScalarMulBasepointVartime(
		edwardsPointTestScalars["A"],
		&A,
		edwardsPointTestScalars["B"],
	)
	if !result.testEqualCompressedY("DOUBLE_SCALAR_MULT_RESULT") {
		t.Fatalf("A_SCALAR * A_TIMES_BASEPOINT + B_SCALAR * BASEPOINT != DOUBLE_SCALAR_MULT_RESULT (Got: %v)", result)
	}

	expandedA := NewExpandedEdwardsPoint(&A)
	result.ExpandedDoubleScalarMulBasepointVartime(
		edwardsPointTestScalars["A"],
		expandedA,
		edwardsPointTestScalars["B"],
	)
	if !result.testEqualCompressedY("DOUBLE_SCALAR_MULT_RESULT") {
		t.Fatalf("A_SCALAR * EXPANDED_A_TIMES_BASEPOINT + B_SCALAR * BASEPOINT != DOUBLE_SCALAR_MULT_RESULT (Got: %v)", result)
	}
}

func testEdwardsMultiscalarMul(t *testing.T) {
	var A EdwardsPoint
	if _, err := A.SetCompressedY(edwardsPointTestPoints["A_TIMES_BASEPOINT"]); err != nil {
		t.Fatalf("A.SetCompressedY(): %v", err)
	}

	var result EdwardsPoint
	result.MultiscalarMul(
		[]*scalar.Scalar{edwardsPointTestScalars["A"], edwardsPointTestScalars["B"]},
		[]*EdwardsPoint{&A, ED25519_BASEPOINT_POINT},
	)
	if !result.testEqualCompressedY("DOUBLE_SCALAR_MULT_RESULT") {
		t.Fatalf("A_SCALAR * A_TIMES_BASEPOINT + B_SCALAR * BASEPOINT != DOUBLE_SCALAR_MULT_RESULT (Got: %v)", result)
	}
}

func testEdwardsMultiscalarMulVartime(t *testing.T) {
	var A EdwardsPoint
	if _, err := A.SetCompressedY(edwardsPointTestPoints["A_TIMES_BASEPOINT"]); err != nil {
		t.Fatalf("A.SetCompressedY(): %v", err)
	}

	var result EdwardsPoint
	result.MultiscalarMulVartime(
		[]*scalar.Scalar{edwardsPointTestScalars["A"], edwardsPointTestScalars["B"]},
		[]*EdwardsPoint{&A, ED25519_BASEPOINT_POINT},
	)
	if !result.testEqualCompressedY("DOUBLE_SCALAR_MULT_RESULT") {
		t.Fatalf("A_SCALAR * A_TIMES_BASEPOINT + B_SCALAR * BASEPOINT != DOUBLE_SCALAR_MULT_RESULT (Got: %v)", result)
	}
}

func testAffineNielsConditionalAssign(t *testing.T) {
	var id, p1, bp affineNielsPoint
	id.Identity()
	p1.Identity()
	bp.SetEdwards(ED25519_BASEPOINT_POINT)

	p1.ConditionalAssign(&bp, 0)
	if !p1.testEqual(&id) {
		t.Fatalf("p1.conditionalAssign(bp, 0) != id")
	}
	p1.ConditionalAssign(&bp, 1)
	if !p1.testEqual(&bp) {
		t.Fatalf("p1.conditionalAssign(bp, 1) != bp")
	}
}

func testAffineNielsConversionClearsDenominators(t *testing.T) {
	var (
		aB, also_aB   EdwardsPoint
		aBAffineNiels affineNielsPoint
		sum           completedPoint
	)
	aB.MulBasepoint(ED25519_BASEPOINT_TABLE, edwardsPointTestScalars["A"])
	also_aB.setCompleted(sum.AddEdwardsAffineNiels(edwardsPointTestIdentity, aBAffineNiels.SetEdwards(&aB)))

	if aB.Equal(&also_aB) != 1 {
		t.Fatalf("aB != also_aB (Got %v %v)", aB, also_aB)
	}
}

func testIsCanonicalVartime(t *testing.T) {
	// Check to see that the hard-coded compressed points with non-canonical
	// sign bits are indeed non-canonical.
	for _, p := range noncanonicalSignBits {
		ep, err := NewEdwardsPoint().SetCompressedY(&p)
		if err != nil {
			t.Fatalf("ep.SetCompressedY(): %v", err)
		}
		pCheck := NewCompressedEdwardsY().SetEdwardsPoint(ep)
		if pCheck.Equal(&p) == 1 {
			t.Fatalf("pCheck == p (p: %v pCheck: %v)", p, pCheck)
		}

		if p.IsCanonicalVartime() {
			t.Fatalf("p.IsCanonicalVartime() == true (p: %v)", p)
		}
	}
}

func (p *EdwardsPoint) testEqualCompressedY(s string) bool {
	pCompressed := NewCompressedEdwardsY().SetEdwardsPoint(p)
	return pCompressed.Equal(edwardsPointTestPoints[s]) == 1
}

func (p *EdwardsPoint) debugIsValid() bool {
	var pProjective projectivePoint
	pProjective.SetEdwards(p)
	pointOnCurve := pProjective.debugIsValid()

	var XY, ZT field.Element
	XY.Mul(&p.inner.X, &p.inner.Y)
	ZT.Mul(&p.inner.Z, &p.inner.T)
	onSegreImage := XY.Equal(&ZT) == 1

	return pointOnCurve && onSegreImage
}

func (p *projectivePoint) debugIsValid() bool {
	// Curve equation is    -x^2 + y^2 = 1 + d*x^2*y^2,
	// homogenized as (-X^2 + Y^2)*Z^2 = Z^4 + d*X^2*Y^2
	var XX, YY, ZZ, ZZZZ field.Element
	XX.Square(&p.X)
	YY.Square(&p.Y)
	ZZ.Square(&p.Z)
	ZZZZ.Square(&ZZ)

	var lhs, rhs field.Element
	lhs.Sub(&YY, &XX)
	lhs.Mul(&lhs, &ZZ)
	rhs.Mul(&XX, &YY)
	rhs.Mul(&rhs, &constEDWARDS_D)
	rhs.Add(&rhs, &ZZZZ)

	return lhs.Equal(&rhs) == 1
}

func (p *affineNielsPoint) testEqual(other *affineNielsPoint) bool {
	res := p.y_plus_x.Equal(&other.y_plus_x) & p.y_minus_x.Equal(&other.y_minus_x) & p.xy2d.Equal(&other.xy2d)

	return res == 1
}

func newTestBenchRandomScalar(tb testing.TB) *scalar.Scalar {
	s, err := scalar.New().SetRandom(rand.Reader)
	if err != nil {
		tb.Fatalf("scalar.New().Random(): %v", err)
	}
	return s
}

func newTestBenchRandomScalars(tb testing.TB, n int) []*scalar.Scalar {
	v := make([]*scalar.Scalar, 0, n)
	for i := 0; i < n; i++ {
		v = append(v, newTestBenchRandomScalar(tb))
	}
	return v
}

func newTestBenchRandomPoint(tb testing.TB) *EdwardsPoint {
	var p EdwardsPoint
	return p.MulBasepoint(ED25519_BASEPOINT_TABLE, newTestBenchRandomScalar(tb))
}
