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

package secp256k1

import (
	"math/big"
	"testing"

	"github.com/leanovate/gopter"
	"github.com/leanovate/gopter/prop"

	"github.com/consensys/gnark-crypto/ecc/secp256k1/fp"
)

func TestG1AffineSerialization(t *testing.T) {
	t.Parallel()
	// test round trip serialization of infinity
	{
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

	properties.TestingRun(t, gopter.ConsoleReporter(false))
}
