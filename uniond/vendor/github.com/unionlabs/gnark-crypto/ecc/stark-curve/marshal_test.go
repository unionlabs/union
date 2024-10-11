// Copyright 2020 ConsenSys Software Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// FOO

package starkcurve

import (
	"bytes"
	"io"
	"math/big"
	"math/rand"
	"testing"

	"github.com/leanovate/gopter"
	"github.com/leanovate/gopter/prop"

	"github.com/consensys/gnark-crypto/ecc/stark-curve/fp"
	"github.com/consensys/gnark-crypto/ecc/stark-curve/fr"
)

func TestEncoder(t *testing.T) {
	t.Parallel()
	// TODO need proper fuzz testing here

	var inA uint64
	var inB fr.Element
	var inC fp.Element
	var inD G1Affine
	var inE G1Affine
	var inG []G1Affine
	var inI []fp.Element
	var inJ []fr.Element

	// set values of inputs
	inA = rand.Uint64() //#nosec G404 weak rng is fine here
	inB.SetRandom()
	inC.SetRandom()
	inD.ScalarMultiplication(&g1GenAff, new(big.Int).SetUint64(rand.Uint64())) //#nosec G404 weak rng is fine here
	// inE --> infinity
	inG = make([]G1Affine, 2)
	inG[1] = inD
	inI = make([]fp.Element, 3)
	inI[2] = inD.X
	inJ = make([]fr.Element, 0)

	// encode them, compressed and raw
	var buf, bufRaw bytes.Buffer
	enc := NewEncoder(&buf)
	encRaw := NewEncoder(&bufRaw, RawEncoding())
	toEncode := []interface{}{inA, &inB, &inC, &inD, &inE, inG, inI, inJ}
	for _, v := range toEncode {
		if err := enc.Encode(v); err != nil {
			t.Fatal(err)
		}
		if err := encRaw.Encode(v); err != nil {
			t.Fatal(err)
		}
	}

	testDecode := func(t *testing.T, r io.Reader, n int64) {
		dec := NewDecoder(r)
		var outA uint64
		var outB fr.Element
		var outC fp.Element
		var outD G1Affine
		var outE G1Affine
		outE.X.SetOne()
		outE.Y.SetUint64(42)
		var outG []G1Affine
		var outI []fp.Element
		var outJ []fr.Element

		toDecode := []interface{}{&outA, &outB, &outC, &outD, &outE, &outG, &outI, &outJ}
		for _, v := range toDecode {
			if err := dec.Decode(v); err != nil {
				t.Fatal(err)
			}
		}

		// compare values
		if inA != outA {
			t.Fatal("didn't encode/decode uint64 value properly")
		}

		if !inB.Equal(&outB) || !inC.Equal(&outC) {
			t.Fatal("decode(encode(Element) failed")
		}
		if !inD.Equal(&outD) || !inE.Equal(&outE) {
			t.Fatal("decode(encode(G1Affine) failed")
		}
		for i := 0; i < len(inG); i++ {
			if !inG[i].Equal(&outG[i]) {
				t.Fatal("decode(encode(slice(points))) failed")
			}
		}
		if (len(inI) != len(outI)) || (len(inJ) != len(outJ)) {
			t.Fatal("decode(encode(slice(elements))) failed")
		}
		for i := 0; i < len(inI); i++ {
			if !inI[i].Equal(&outI[i]) {
				t.Fatal("decode(encode(slice(elements))) failed")
			}
		}
		if n != dec.BytesRead() {
			t.Fatal("bytes read don't match bytes written")
		}
	}

	// decode them
	testDecode(t, &buf, enc.BytesWritten())
	testDecode(t, &bufRaw, encRaw.BytesWritten())

}

func TestIsCompressed(t *testing.T) {
	t.Parallel()
	var g1Inf, g1 G1Affine

	g1 = g1GenAff

	{
		b := g1Inf.Bytes()
		if !isCompressed(b[0]) {
			t.Fatal("g1Inf.Bytes() should be compressed")
		}
	}

	{
		b := g1Inf.RawBytes()
		if isCompressed(b[0]) {
			t.Fatal("g1Inf.RawBytes() should be uncompressed")
		}
	}

	{
		b := g1.Bytes()
		if !isCompressed(b[0]) {
			t.Fatal("g1.Bytes() should be compressed")
		}
	}

	{
		b := g1.RawBytes()
		if isCompressed(b[0]) {
			t.Fatal("g1.RawBytes() should be uncompressed")
		}
	}

}

func TestG1AffineSerialization(t *testing.T) {
	t.Parallel()
	// test round trip serialization of infinity
	{
		// compressed
		{
			var p1, p2 G1Affine
			p2.X.SetRandom()
			p2.Y.SetRandom()
			buf := p1.Bytes()
			n, err := p2.SetBytes(buf[:])
			if err != nil {
				t.Fatal(err)
			}
			if n != SizeOfG1AffineCompressed {
				t.Fatal("invalid number of bytes consumed in buffer")
			}
			if !(p2.X.IsZero() && p2.Y.IsZero()) {
				t.Fatal("deserialization of uncompressed infinity point is not infinity")
			}
		}

		// uncompressed
		{
			var p1, p2 G1Affine
			p2.X.SetRandom()
			p2.Y.SetRandom()
			buf := p1.RawBytes()
			n, err := p2.SetBytes(buf[:])
			if err != nil {
				t.Fatal(err)
			}
			if n != SizeOfG1AffineUncompressed {
				t.Fatal("invalid number of bytes consumed in buffer")
			}
			if !(p2.X.IsZero() && p2.Y.IsZero()) {
				t.Fatal("deserialization of uncompressed infinity point is not infinity")
			}
		}
	}

	parameters := gopter.DefaultTestParameters()
	if testing.Short() {
		parameters.MinSuccessfulTests = nbFuzzShort
	} else {
		parameters.MinSuccessfulTests = nbFuzz
	}

	properties := gopter.NewProperties(parameters)

	properties.Property("[G1] Affine SetBytes(RawBytes) should stay the same", prop.ForAll(
		func(a fp.Element) bool {
			var start, end G1Affine
			var ab big.Int
			a.BigInt(&ab)
			start.ScalarMultiplication(&g1GenAff, &ab)

			buf := start.RawBytes()
			n, err := end.SetBytes(buf[:])
			if err != nil {
				return false
			}
			if n != SizeOfG1AffineUncompressed {
				return false
			}
			return start.X.Equal(&end.X) && start.Y.Equal(&end.Y)
		},
		GenFp(),
	))

	properties.Property("[G1] Affine SetBytes(Bytes()) should stay the same", prop.ForAll(
		func(a fp.Element) bool {
			var start, end G1Affine
			var ab big.Int
			a.BigInt(&ab)
			start.ScalarMultiplication(&g1GenAff, &ab)

			buf := start.Bytes()
			n, err := end.SetBytes(buf[:])
			if err != nil {
				return false
			}
			if n != SizeOfG1AffineCompressed {
				return false
			}
			return start.X.Equal(&end.X) && start.Y.Equal(&end.Y)
		},
		GenFp(),
	))

	properties.TestingRun(t, gopter.ConsoleReporter(false))
}
