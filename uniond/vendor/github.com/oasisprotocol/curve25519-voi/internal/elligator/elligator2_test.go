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
	"fmt"
	"testing"

	"github.com/oasisprotocol/curve25519-voi/curve"
	"github.com/oasisprotocol/curve25519-voi/internal/field"
	"github.com/oasisprotocol/curve25519-voi/internal/testhelpers"
)

const montgomeryUniformSize = 32

func montgomeryFromUniformBytes(in []byte) (*curve.MontgomeryPoint, error) {
	if len(in) != montgomeryUniformSize {
		return nil, fmt.Errorf("curve/montgomery: unexpected representative size")
	}

	var r field.Element
	if _, err := r.SetBytes(in[:]); err != nil {
		return nil, fmt.Errorf("curve/montgomery: failed to deserailize r: %w", err)
	}

	var p curve.MontgomeryPoint
	u, _ := montgomeryFlavor(&r)
	_ = u.ToBytes(p[:])

	return &p, nil
}

func TestMontgomeryElligator2(t *testing.T) {
	// Test vectors stolen from Monocypher's tis-ci-vectors.h
	type testVector struct {
		repr     []byte
		expected *curve.MontgomeryPoint
	}

	testVectors := []testVector{
		{
			testhelpers.MustUnhex(t, "0000000000000000000000000000000000000000000000000000000000000000"),
			mustUnhexMontgomery(t, "0000000000000000000000000000000000000000000000000000000000000000"),
		},
		{
			testhelpers.MustUnhex(t, "0000000000000000000000000000000000000000000000000000000000000040"),
			mustUnhexMontgomery(t, "0000000000000000000000000000000000000000000000000000000000000000"),
		},
		{
			testhelpers.MustUnhex(t, "0000000000000000000000000000000000000000000000000000000000000080"),
			mustUnhexMontgomery(t, "0000000000000000000000000000000000000000000000000000000000000000"),
		},
		{
			testhelpers.MustUnhex(t, "00000000000000000000000000000000000000000000000000000000000000c0"),
			mustUnhexMontgomery(t, "0000000000000000000000000000000000000000000000000000000000000000"),
		},
		{
			testhelpers.MustUnhex(t, "673a505e107189ee54ca93310ac42e4545e9e59050aaac6f8b5f64295c8ec02f"),
			mustUnhexMontgomery(t, "242ae39ef158ed60f20b89396d7d7eef5374aba15dc312a6aea6d1e57cacf85e"),
		},
		{
			testhelpers.MustUnhex(t, "922688fa428d42bc1fa8806998fbc5959ae801817e85a42a45e8ec25a0d7545a"),
			mustUnhexMontgomery(t, "696f341266c64bcfa7afa834f8c34b2730be11c932e08474d1a22f26ed82410b"),
		},
		{
			testhelpers.MustUnhex(t, "0d3b0eb88b74ed13d5f6a130e03c4ad607817057dc227152827c0506a538bbba"),
			mustUnhexMontgomery(t, "0b00df174d9fb0b6ee584d2cf05613130bad18875268c38b377e86dfefef177f"),
		},
		{
			testhelpers.MustUnhex(t, "01a3ea5658f4e00622eeacf724e0bd82068992fae66ed2b04a8599be16662ef5"),
			mustUnhexMontgomery(t, "7ae4c58bc647b5646c9f5ae4c2554ccbf7c6e428e7b242a574a5a9c293c21f7e"),
		},
		{
			testhelpers.MustUnhex(t, "69599ab5a829c3e9515128d368da7354a8b69fcee4e34d0a668b783b6cae550f"),
			mustUnhexMontgomery(t, "09024abaaef243e3b69366397e8dfc1fdc14a0ecc7cf497cbe4f328839acce69"),
		},
		{
			testhelpers.MustUnhex(t, "9172922f96d2fa41ea0daf961857056f1656ab8406db80eaeae76af58f8c9f50"),
			mustUnhexMontgomery(t, "beab745a2a4b4e7f1a7335c3ffcdbd85139f3a72b667a01ee3e3ae0e530b3372"),
		},
		{
			testhelpers.MustUnhex(t, "6850a20ac5b6d2fa7af7042ad5be234d3311b9fb303753dd2b610bd566983281"),
			mustUnhexMontgomery(t, "1287388eb2beeff706edb9cf4fcfdd35757f22541b61528570b86e8915be1530"),
		},
		{
			testhelpers.MustUnhex(t, "84417826c0e80af7cb25a73af1ba87594ff7048a26248b5757e52f2824e068f1"),
			mustUnhexMontgomery(t, "51acd2e8910e7d28b4993db7e97e2b995005f26736f60dcdde94bdf8cb542251"),
		},
		{
			testhelpers.MustUnhex(t, "b0fbe152849f49034d2fa00ccc7b960fad7b30b6c4f9f2713eb01c147146ad31"),
			mustUnhexMontgomery(t, "98508bb3590886af3be523b61c3d0ce6490bb8b27029878caec57e4c750f993d"),
		},
		{
			testhelpers.MustUnhex(t, "a0ca9ff75afae65598630b3b93560834c7f4dd29a557aa29c7becd49aeef3753"),
			mustUnhexMontgomery(t, "3c5fad0516bb8ec53da1c16e910c23f792b971c7e2a0ee57d57c32e3655a646b"),
		},
	}
	for i, v := range testVectors {
		// Monocypher explicitly ignores the 2 most significant bits,
		// but our implementation does not.  Mask them off.
		var clamped [curve.MontgomeryPointSize]byte
		copy(clamped[:], v.repr)
		clamped[31] &= 63

		p, err := montgomeryFromUniformBytes(clamped[:])
		if err != nil {
			t.Fatalf("p.SetUniformBytes(v[%d].repr): %v", i, err)
		}
		if p.Equal(v.expected) != 1 {
			t.Fatalf("p[%d] != vector[%d] (Got: %v)", i, i, p)
		}
	}
}

func TestSetEdwardsFromXY(t *testing.T) {
	// Ensure that the routine doesn't puke on the output from EdwardsFlavor
	// when t == 0 or s == -1.

	var p curve.EdwardsPoint
	SetEdwardsFromXY(&p, &constFieldZero, &field.One)
}

func mustUnhexMontgomery(t *testing.T, x string) *curve.MontgomeryPoint {
	b := testhelpers.MustUnhex(t, x)

	var p curve.MontgomeryPoint
	if _, err := p.SetBytes(b); err != nil {
		t.Fatalf("montgomery: failed to parse hex: %v", err)
	}

	return &p
}
