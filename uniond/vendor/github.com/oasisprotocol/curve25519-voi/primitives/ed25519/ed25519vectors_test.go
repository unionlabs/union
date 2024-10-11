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

package ed25519

import (
	"compress/gzip"
	"encoding/json"
	"fmt"
	"os"
	"testing"

	"github.com/oasisprotocol/curve25519-voi/curve"
	"github.com/oasisprotocol/curve25519-voi/internal/testhelpers"
)

var ed25519Vectors = func() []ed25519Vector {
	f, err := os.Open("./testdata/ed25519vectors.json.gz")
	if err != nil {
		panic("ed25519vectors: failed to open test data: " + err.Error())
	}
	defer f.Close()

	rd, err := gzip.NewReader(f)
	if err != nil {
		panic("ed25519vectors: failed to instantiate gzip: " + err.Error())
	}
	defer rd.Close()

	var ret []ed25519Vector

	dec := json.NewDecoder(rd)
	if err = dec.Decode(&ret); err != nil {
		panic("ed25519vectors: failed to parse test vectors: " + err.Error())
	}

	return ret
}()

type ed25519Vector struct {
	A     string
	Rhex  string `json:"R"`
	Shex  string `json:"S"`
	M     string
	Flags []string
}

func (v *ed25519Vector) PublicKey(t *testing.T) []byte {
	return testhelpers.MustUnhex(t, v.A)
}

func (v *ed25519Vector) R(t *testing.T) []byte {
	return testhelpers.MustUnhex(t, v.Rhex)
}

func (v *ed25519Vector) S(t *testing.T) []byte {
	return testhelpers.MustUnhex(t, v.Shex)
}

func (v *ed25519Vector) Signature(t *testing.T) []byte {
	return append(v.R(t), v.S(t)...)
}

func (v *ed25519Vector) Message() []byte {
	return []byte(v.M)
}

func shouldVectorVerifyWithOpts(v *ed25519Vector, vOpts *VerifyOptions) bool {
	shouldVerify := true
	for _, flag := range v.Flags {
		switch flag {
		case "LowOrderR":
			shouldVerify = shouldVerify && vOpts.AllowSmallOrderR
		case "LowOrderA":
			shouldVerify = shouldVerify && vOpts.AllowSmallOrderA
		case "LowOrderComponentR", "LowOrderComponentA":
			// None of the presets check for a torsion component.
		case "LowOrderResidue":
			shouldVerify = shouldVerify && !vOpts.CofactorlessVerify
		case "NonCanonicalA":
			shouldVerify = shouldVerify && vOpts.AllowNonCanonicalA
		case "NonCanonicalR":
			shouldVerify = shouldVerify && vOpts.AllowNonCanonicalR
		default:
			panic("unknown flag: " + flag)
		}
	}

	return shouldVerify
}

func TestEd25519Vectors(t *testing.T) {
	checkVectors := func(t *testing.T, vOpts *VerifyOptions) {
		for i, vec := range ed25519Vectors {
			expected := shouldVectorVerifyWithOpts(&vec, vOpts)

			t.Run(fmt.Sprintf("TestCase/%d", i), func(t *testing.T) {
				ok := VerifyWithOptions(
					PublicKey(vec.PublicKey(t)),
					vec.Message(),
					vec.Signature(t),
					&Options{
						Verify: vOpts,
					},
				)
				if expected != ok {
					t.Errorf("verification mismatch: %v (%v)", ok, vec.Flags)
				}
			})

			t.Run(fmt.Sprintf("TestCase/%d/Batch", i), func(t *testing.T) {
				v := NewBatchVerifier()
				v.AddWithOptions(
					PublicKey(vec.PublicKey(t)),
					vec.Message(),
					vec.Signature(t),
					&Options{
						Verify: vOpts,
					},
				)

				// Can't use VerifyBatchOnly, because it doesn't work with
				// the StdLib profile.
				ok, _ := v.Verify(nil)
				if expected != ok {
					t.Errorf("verification mismatch: %v (%v)", ok, vec.Flags)
				}
			})
		}
	}

	t.Run("Default", func(t *testing.T) {
		checkVectors(t, VerifyOptionsDefault)
	})
	t.Run("StdLib", func(t *testing.T) {
		checkVectors(t, VerifyOptionsStdLib)
	})
	t.Run("FIPS-186-5", func(t *testing.T) {
		checkVectors(t, VerifyOptionsFIPS_186_5)
	})
	t.Run("ZIP-215", func(t *testing.T) {
		checkVectors(t, VerifyOptionsZIP_215)
	})
}

func TestIsCanonicalVartime(t *testing.T) {
	// This should live in the curve package, but this is where the convenient
	// test vectors live.
	for i, vec := range ed25519Vectors {
		t.Run(fmt.Sprintf("TestCase/%d", i), func(t *testing.T) {
			canonicalA, canonicalR := true, true
			for _, flag := range vec.Flags {
				switch flag {
				case "NonCanonicalA":
					canonicalA = false
				case "NonCanonicalR":
					canonicalR = false
				}
			}

			var p curve.CompressedEdwardsY
			if _, err := p.SetBytes(vec.PublicKey(t)); err != nil {
				t.Fatalf("failed to deserialize A: %v", err)
			}
			if isCanonical := p.IsCanonicalVartime(); isCanonical != canonicalA {
				t.Fatalf("A.IsCanonicalVartime() mismatch: %v (%v)", isCanonical, canonicalA)
			}
			if _, err := p.SetBytes(vec.R(t)); err != nil {
				t.Fatalf("failed to deserialize R: %v", err)
			}
			if isCanonical := p.IsCanonicalVartime(); isCanonical != canonicalR {
				t.Fatalf("R.IsCanonicalVartime() mismatch: %v (%v)", isCanonical, canonicalR)
			}
		})
	}
}
