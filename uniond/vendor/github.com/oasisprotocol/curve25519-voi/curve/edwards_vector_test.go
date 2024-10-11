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

package curve

import (
	"testing"

	"github.com/oasisprotocol/curve25519-voi/curve/scalar"
)

func TestVector(t *testing.T) {
	if !supportsVectorizedEdwards {
		t.Skipf("Vector backend not supported")
	}

	t.Run("ExtendedPoint", func(t *testing.T) {
		t.Run("AddSubCached", testVecAddSubCached)
		t.Run("Double", testVecDoubleExtended)
	})
}

func testVecDoubleExtended(t *testing.T) {
	doubleEdwardsSerial := func(p *EdwardsPoint) *EdwardsPoint {
		var out EdwardsPoint
		return out.double(p)
	}

	doubleEdwardsVector := func(p *EdwardsPoint) *EdwardsPoint {
		var pExtended extendedPoint
		pExtended.Double(pExtended.SetEdwards(p))

		var out EdwardsPoint
		return out.setExtended(&pExtended)
	}

	for _, v := range []struct {
		p *EdwardsPoint
		n string
	}{
		{ED25519_BASEPOINT_POINT, "B"},
		{edwardsPointTestIdentity, "id"},
		{testPoint_kB(), "([k]B)"},
	} {
		pS := doubleEdwardsSerial(v.p)
		pV := doubleEdwardsVector(v.p)

		if pS.Equal(pV) != 1 {
			t.Fatalf("[2]%s incorrect (Got: %v)", v.n, pV)
		}
	}
}

func testVecAddSubCached(t *testing.T) {
	addSubEdwardsVector := func(a, b *EdwardsPoint, isSub bool) *EdwardsPoint {
		var (
			aExtended, bExtended, abExtended extendedPoint
			bCached                          cachedPoint
		)
		aExtended.SetEdwards(a)
		bCached.SetExtended(bExtended.SetEdwards(b))

		switch isSub {
		case false:
			abExtended.AddExtendedCached(&aExtended, &bCached)
		case true:
			abExtended.SubExtendedCached(&aExtended, &bCached)
		}

		var out EdwardsPoint
		return out.setExtended(&abExtended)
	}

	addEdwardsVector := func(a, b *EdwardsPoint) *EdwardsPoint {
		return addSubEdwardsVector(a, b, false)
	}

	subEdwardsVector := func(a, b *EdwardsPoint) *EdwardsPoint {
		return addSubEdwardsVector(a, b, true)
	}

	addEdwardsSerial := func(a, b *EdwardsPoint) *EdwardsPoint {
		var out EdwardsPoint
		out.Add(a, b)
		return &out
	}

	subEdwardsSerial := func(a, b *EdwardsPoint) *EdwardsPoint {
		var out EdwardsPoint
		out.Sub(a, b)
		return &out
	}

	for _, v := range []struct {
		a, b   *EdwardsPoint
		an, bn string
	}{
		{edwardsPointTestIdentity, edwardsPointTestIdentity, "id", "id"},
		{edwardsPointTestIdentity, ED25519_BASEPOINT_POINT, "id", "B"},
		{ED25519_BASEPOINT_POINT, ED25519_BASEPOINT_POINT, "B", "B"},
		{ED25519_BASEPOINT_POINT, testPoint_kB(), "B", "([k]B)"},
	} {
		sS := addEdwardsSerial(v.a, v.b)
		sV := addEdwardsVector(v.a, v.b)
		if sS.Equal(sV) != 1 {
			t.Fatalf("%s + %s incorrect (Got: %v)", v.an, v.bn, sV)
		}

		dS := subEdwardsSerial(v.a, v.b)
		dV := subEdwardsVector(v.a, v.b)
		if dS.Equal(dV) != 1 {
			t.Fatalf("%s - %s incorrect (Got: %v)", v.an, v.bn, dV)
		}
	}
}

func testPoint_kB() *EdwardsPoint {
	var p EdwardsPoint
	return p.MulBasepoint(ED25519_BASEPOINT_TABLE, scalar.NewFromUint64(8475983829))
}
