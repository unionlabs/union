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
	"bytes"
	"testing"

	"github.com/oasisprotocol/curve25519-voi/curve/scalar"
	"github.com/oasisprotocol/curve25519-voi/internal/field"
)

func (p *RistrettoPoint) coset4() [4]EdwardsPoint {
	var ret [4]EdwardsPoint

	ret[0] = p.inner
	ret[1].Add(&p.inner, EIGHT_TORSION[2])
	ret[2].Add(&p.inner, EIGHT_TORSION[4])
	ret[3].Add(&p.inner, EIGHT_TORSION[6])

	return ret
}

func TestRistretto(t *testing.T) {
	t.Run("Ristretto/Sum", testRistrettoSum)
	t.Run("Ristretto/Decompress/NegativeS", testRistrettoDecompressNegativeSFails)
	t.Run("Ristretto/Decompress/Id", testRistrettoDecompressId)
	t.Run("Ristretto/Compress/Id", testRistrettoCompressId)
	t.Run("Ristretto/Roundtrip/Basepoint", testRistrettoBasepointRoundtrip)
	t.Run("Ristretto/Roundtrip/Random", testRistrettoRandomRoundtrip)
	t.Run("Ristretto/FourTorsion/Basepoint", testRistrettoFourTorsionBasepoint)
	t.Run("Ristretto/FourTorsion/Random", testRistrettoFourTorsionRandom)
	t.Run("Ristretto/Elligator", testRistrettoElligator)
	t.Run("Ristretto/TestVectors", testRistrettoVectors)
	t.Run("Ristretto/Serialization", testRistrettoSerialization)
}

func testRistrettoSum(t *testing.T) {
	base := RISTRETTO_BASEPOINT_POINT

	s1, s2 := scalar.NewFromUint64(999), scalar.NewFromUint64(333)

	var p1, p2, expected RistrettoPoint
	p1.Mul(base, s1)
	p2.Mul(base, s2)
	expected.Add(&p1, &p2)

	var sum RistrettoPoint
	sum.Sum([]*RistrettoPoint{&p1, &p2})

	if sum.Equal(&expected) != 1 {
		t.Fatalf("Sum({p1, p2}) != expected (Got: %v)", sum)
	}

	// Test that sum works with an empty slice.
	expected.Identity()
	sum.Sum([]*RistrettoPoint{})
	if sum.Equal(&expected) != 1 {
		t.Fatalf("Sum({}) != identity (Got: %v)", sum)
	}

	// Test that sum works with a nil slice.
	sum.Sum(nil)
	if sum.Equal(&expected) != 1 {
		t.Fatalf("Sum({}) != identity (Got: %v)", sum)
	}
}

func testRistrettoDecompressNegativeSFails(t *testing.T) {
	// constEDWARDS_D is neg, so decompression should fail as |d| != d.
	var bad CompressedRistretto
	_ = constEDWARDS_D.ToBytes(bad[:])

	var p RistrettoPoint
	if _, err := p.SetCompressed(&bad); err == nil {
		t.Fatalf("FromCompressed(constEDWARDS_D) succeeded")
	}
}

func testRistrettoDecompressId(t *testing.T) {
	var (
		compressedId CompressedRistretto
		id           RistrettoPoint
	)
	compressedId.Identity()
	if _, err := id.SetCompressed(&compressedId); err != nil {
		t.Fatalf("FromCompressed(compressedId): %v", err)
	}

	var compressedEdwardsId CompressedEdwardsY
	compressedEdwardsId.Identity()

	var identityInCoset bool
	for _, p := range id.coset4() {
		var tmp CompressedEdwardsY
		tmp.SetEdwardsPoint(&p)

		if tmp.Equal(&compressedEdwardsId) == 1 {
			identityInCoset = true
		}
	}

	if !identityInCoset {
		t.Fatalf("identity not in coset")
	}
}

func testRistrettoCompressId(t *testing.T) {
	var (
		id           RistrettoPoint
		compressedId CompressedRistretto
	)
	id.Identity()
	compressedId.Identity()

	var cp CompressedRistretto
	cp.SetRistrettoPoint(&id)

	if cp.Equal(&compressedId) != 1 {
		t.Fatalf("cp != compressedId (Got: %v)", cp)
	}
}

func testRistrettoBasepointRoundtrip(t *testing.T) {
	var bpCompressedRistretto CompressedRistretto
	bpCompressedRistretto.SetRistrettoPoint(RISTRETTO_BASEPOINT_POINT)

	var bpRecaf RistrettoPoint
	_, _ = bpRecaf.SetCompressed(&bpCompressedRistretto)

	// Check that bpRecaf differs from bp by a point of order 4.
	var (
		diff  RistrettoPoint
		diff4 EdwardsPoint
	)
	diff.Sub(RISTRETTO_BASEPOINT_POINT, &bpRecaf)
	diff4.mulByPow2(&diff.inner, 2)

	var compressedDiff4, compressedEdwardsId CompressedEdwardsY
	compressedEdwardsId.Identity()
	compressedDiff4.SetEdwardsPoint(&diff4)

	if compressedDiff4.Equal(&compressedEdwardsId) != 1 {
		t.Fatalf("diff4 != id (Got: %v)", compressedDiff4)
	}
}

func testRistrettoFourTorsionBasepoint(t *testing.T) {
	bp := RISTRETTO_BASEPOINT_POINT
	bpCoset := bp.coset4()
	for i, p := range bpCoset {
		if bp.Equal(&RistrettoPoint{inner: p}) != 1 {
			t.Fatalf("bp != bpCoset[%d] (Got: %v)", i, p)
		}
	}
}

func testRistrettoFourTorsionRandom(t *testing.T) {
	var p RistrettoPoint
	p.MulBasepoint(RISTRETTO_BASEPOINT_TABLE, newTestBenchRandomScalar(t))

	pCoset := p.coset4()
	for i, pp := range pCoset {
		if p.Equal(&RistrettoPoint{inner: pp}) != 1 {
			t.Fatalf("p != pCoset[%d] (Got: %v)", i, pp)
		}
	}
}

func testRistrettoElligator(t *testing.T) {
	// Test vectors extracted from ristretto.sage.
	//
	// Notice that all of the byte sequences have bit 255 set to 0;
	// this is because ristretto.sage does not mask the high bit of
	// a field element.  When the high bit is set, the ristretto.sage
	// elligator implementation gives different results, since it
	// takes a different field element as input.
	r0Bytes := [][field.ElementSize]byte{
		{184, 249, 135, 49, 253, 123, 89, 113, 67, 160, 6, 239, 7, 105, 211, 41, 192, 249, 185, 57, 9, 102, 70, 198, 15, 127, 7, 26, 160, 102, 134, 71},
		{229, 14, 241, 227, 75, 9, 118, 60, 128, 153, 226, 21, 183, 217, 91, 136, 98, 0, 231, 156, 124, 77, 82, 139, 142, 134, 164, 169, 169, 62, 250, 52},
		{115, 109, 36, 220, 180, 223, 99, 6, 204, 169, 19, 29, 169, 68, 84, 23, 21, 109, 189, 149, 127, 205, 91, 102, 172, 35, 112, 35, 134, 69, 186, 34},
		{16, 49, 96, 107, 171, 199, 164, 9, 129, 16, 64, 62, 241, 63, 132, 173, 209, 160, 112, 215, 105, 50, 157, 81, 253, 105, 1, 154, 229, 25, 120, 83},
		{156, 131, 161, 162, 236, 251, 5, 187, 167, 171, 17, 178, 148, 210, 90, 207, 86, 21, 79, 161, 167, 215, 234, 1, 136, 242, 182, 248, 38, 85, 79, 86},
		{251, 177, 124, 54, 18, 101, 75, 235, 245, 186, 19, 46, 133, 157, 229, 64, 10, 136, 181, 185, 78, 144, 254, 167, 137, 49, 107, 10, 61, 10, 21, 25},
		{232, 193, 20, 68, 240, 77, 186, 77, 183, 40, 44, 86, 150, 31, 198, 212, 76, 81, 3, 217, 197, 8, 126, 128, 126, 152, 164, 208, 153, 44, 189, 77},
		{173, 229, 149, 177, 37, 230, 30, 69, 61, 56, 172, 190, 219, 115, 167, 194, 71, 134, 59, 75, 28, 244, 118, 26, 162, 97, 64, 16, 15, 189, 30, 64},
		{106, 71, 61, 107, 250, 117, 42, 151, 91, 202, 212, 100, 52, 188, 190, 21, 125, 218, 31, 18, 253, 241, 160, 133, 57, 242, 3, 164, 189, 68, 111, 75},
		{112, 204, 182, 90, 220, 198, 120, 73, 173, 107, 193, 17, 227, 40, 162, 36, 150, 141, 235, 55, 172, 183, 12, 39, 194, 136, 43, 153, 244, 118, 91, 89},
		{111, 24, 203, 123, 254, 189, 11, 162, 51, 196, 163, 136, 204, 143, 10, 222, 33, 112, 81, 205, 34, 35, 8, 66, 90, 6, 164, 58, 170, 177, 34, 25},
		{225, 183, 30, 52, 236, 82, 6, 183, 109, 25, 227, 181, 25, 82, 41, 193, 80, 77, 161, 80, 242, 203, 79, 204, 136, 245, 131, 110, 237, 106, 3, 58},
		{207, 246, 38, 56, 30, 86, 176, 90, 27, 200, 61, 42, 221, 27, 56, 210, 79, 178, 189, 120, 68, 193, 120, 167, 77, 185, 53, 197, 124, 128, 191, 126},
		{1, 136, 215, 80, 240, 46, 63, 147, 16, 244, 230, 207, 82, 189, 74, 50, 106, 169, 138, 86, 30, 131, 214, 202, 166, 125, 251, 228, 98, 24, 36, 21},
		{210, 207, 228, 56, 155, 116, 207, 54, 84, 195, 251, 215, 249, 199, 116, 75, 109, 239, 196, 251, 194, 246, 252, 228, 70, 146, 156, 35, 25, 39, 241, 4},
		{34, 116, 123, 9, 8, 40, 93, 189, 9, 103, 57, 103, 66, 227, 3, 2, 157, 107, 134, 219, 202, 74, 230, 154, 78, 107, 219, 195, 214, 14, 84, 80},
	}
	encodedImages := []CompressedRistretto{
		{176, 157, 237, 97, 66, 29, 140, 166, 168, 94, 26, 157, 212, 216, 229, 160, 195, 246, 232, 239, 169, 112, 63, 193, 64, 32, 152, 69, 11, 190, 246, 86},
		{234, 141, 77, 203, 181, 225, 250, 74, 171, 62, 15, 118, 78, 212, 150, 19, 131, 14, 188, 238, 194, 244, 141, 138, 166, 162, 83, 122, 228, 201, 19, 26},
		{232, 231, 51, 92, 5, 168, 80, 36, 173, 179, 104, 68, 186, 149, 68, 40, 140, 170, 27, 103, 99, 140, 21, 242, 43, 62, 250, 134, 208, 255, 61, 89},
		{208, 120, 140, 129, 177, 179, 237, 159, 252, 160, 28, 13, 206, 5, 211, 241, 192, 218, 1, 97, 130, 241, 20, 169, 119, 46, 246, 29, 79, 80, 77, 84},
		{202, 11, 236, 145, 58, 12, 181, 157, 209, 6, 213, 88, 75, 147, 11, 119, 191, 139, 47, 142, 33, 36, 153, 193, 223, 183, 178, 8, 205, 120, 248, 110},
		{26, 66, 231, 67, 203, 175, 116, 130, 32, 136, 62, 253, 215, 46, 5, 214, 166, 248, 108, 237, 216, 71, 244, 173, 72, 133, 82, 6, 143, 240, 104, 41},
		{40, 157, 102, 96, 201, 223, 200, 197, 150, 181, 106, 83, 103, 126, 143, 33, 145, 230, 78, 6, 171, 146, 210, 143, 112, 5, 245, 23, 183, 138, 18, 120},
		{220, 37, 27, 203, 239, 196, 176, 131, 37, 66, 188, 243, 185, 250, 113, 23, 167, 211, 154, 243, 168, 215, 54, 171, 159, 36, 195, 81, 13, 150, 43, 43},
		{232, 121, 176, 222, 183, 196, 159, 90, 238, 193, 105, 52, 101, 167, 244, 170, 121, 114, 196, 6, 67, 152, 80, 185, 221, 7, 83, 105, 176, 208, 224, 121},
		{226, 181, 183, 52, 241, 163, 61, 179, 221, 207, 220, 73, 245, 242, 25, 236, 67, 84, 179, 222, 167, 62, 167, 182, 32, 9, 92, 30, 165, 127, 204, 68},
		{226, 119, 16, 242, 200, 139, 240, 87, 11, 222, 92, 146, 156, 243, 46, 119, 65, 59, 1, 248, 92, 183, 50, 175, 87, 40, 206, 53, 208, 220, 148, 13},
		{70, 240, 79, 112, 54, 157, 228, 146, 74, 122, 216, 88, 232, 62, 158, 13, 14, 146, 115, 117, 176, 222, 90, 225, 244, 23, 94, 190, 150, 7, 136, 96},
		{22, 71, 241, 103, 45, 193, 195, 144, 183, 101, 154, 50, 39, 68, 49, 110, 51, 44, 62, 0, 229, 113, 72, 81, 168, 29, 73, 106, 102, 40, 132, 24},
		{196, 133, 107, 11, 130, 105, 74, 33, 204, 171, 133, 221, 174, 193, 241, 36, 38, 179, 196, 107, 219, 185, 181, 253, 228, 47, 155, 42, 231, 73, 41, 78},
		{58, 255, 225, 197, 115, 208, 160, 143, 39, 197, 82, 69, 143, 235, 92, 170, 74, 40, 57, 11, 171, 227, 26, 185, 217, 207, 90, 185, 197, 190, 35, 60},
		{88, 43, 92, 118, 223, 136, 105, 145, 238, 186, 115, 8, 214, 112, 153, 253, 38, 108, 205, 230, 157, 130, 11, 66, 101, 85, 253, 110, 110, 14, 148, 112},
	}

	for i, b := range r0Bytes {
		var r_0 field.Element
		_, _ = r_0.SetBytes(b[:])

		var q RistrettoPoint
		q.elligatorRistrettoFlavor(&r_0)

		var qCompressed CompressedRistretto
		qCompressed.SetRistrettoPoint(&q)

		if qCompressed.Equal(&encodedImages[i]) != 1 {
			t.Fatalf("q != encodedImages[%d] (Got %v)", i, qCompressed)
		}
	}
}

func testRistrettoRandomRoundtrip(t *testing.T) {
	for i := 0; i < 100; i++ {
		var p RistrettoPoint
		p.MulBasepoint(RISTRETTO_BASEPOINT_TABLE, newTestBenchRandomScalar(t))

		var compressedP CompressedRistretto
		compressedP.SetRistrettoPoint(&p)

		var q RistrettoPoint
		_, _ = q.SetCompressed(&compressedP)
		if p.Equal(&q) != 1 {
			t.Fatalf("p != q (Got %v, %v)", p, q)
		}
	}
}

func testRistrettoSerialization(t *testing.T) {
	var p RistrettoPoint

	if _, err := p.SetRandom(nil); err != nil {
		t.Fatalf("p.SetRandom: %v", err)
	}
	if p.IsIdentity() {
		t.Fatalf("random point is identity???")
	}

	b, err := p.MarshalBinary()
	if err != nil {
		t.Fatalf("RistrettoPoint.MarshalBinary: %v", err)
	}

	// Check that RistrettoPoints round-trip.
	var pp RistrettoPoint
	if err = pp.UnmarshalBinary(b); err != nil {
		t.Fatalf("RistrettoPoint.UnmarshalBinary: %v", err)
	}
	if p.Equal(&pp) != 1 {
		t.Fatalf("p != pp (Got %v, %v)", p, pp)
	}

	// Check that CompressedRistrettos round-trip.
	var pc CompressedRistretto
	pp.Identity()
	if err = pc.UnmarshalBinary(b); err != nil {
		t.Fatalf("CompressedRistretto.UnmarshalBinary: %v", err)
	}
	if _, err = pp.SetCompressed(&pc); err != nil {
		t.Fatalf("RistrettoPoint.SetCompressed: %v", err)
	}
	if p.Equal(&pp) != 1 {
		t.Fatalf("compressed p != pp (Got %v, %v)", p, pp)
	}

	bb, err := pc.MarshalBinary()
	if err != nil {
		t.Fatalf("CompressedRistretto.MarshalBinary: %v", err)
	}
	if !bytes.Equal(bb, b) {
		t.Fatalf("b != bb (Got %v, %v)", b, bb)
	}
}
