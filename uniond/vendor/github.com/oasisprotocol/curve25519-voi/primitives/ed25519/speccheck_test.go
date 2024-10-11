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

package ed25519

// Validate the implementation behavior against the tests presented in
// the paper "Taming the many EdDSAs" by Chalkias, Garillot, and
// Nikolaenko.
//
// Test data taken at commit 336651ba7f1c1ae90b7deac7d175290863a00b66 from
// https://github.com/novifinancial/ed25519-speccheck/blob/master/scripts/cases.json

import (
	"compress/gzip"
	"crypto/rand"
	"encoding/json"
	"fmt"
	"os"
	"testing"

	"github.com/oasisprotocol/curve25519-voi/internal/testhelpers"
)

type verificationImpl int

const (
	implVanilla verificationImpl = iota
	implBatch
	implExpanded
	implExpandedBatch
)

var speccheckExpectedResults = []bool{
	false, // 0: small order A, small order R
	false, // 1: small order A, mixed order R
	true,  // 2: mixed order A, small order R
	true,  // 3: mixed order A, mixed order R
	true,  // 4: cofactored verify
	true,  // 5: cofactored verify computes 8(hA) instead of (8h mod L)A
	false, // 6: non-canonical S (S > L)
	false, // 7: non-canonical S (S >> L)
	false, // 8: mixed order A, non-canonical small order R (accepted if R reduced before hashing)
	false, // 9: mixed order A, non-canonical small order R (accepted if R not reduced before hashing)
	false, // 10: non-canonical small order A, mixed order R (accepted if cofactored or A reduced before hashing)
	false, // 11: non-canonical small order A, mixed order R (accepted if cofactored or A not reduced before hashing)
}

var speccheckExpectedResultsStdLib = []bool{
	true,  // 0: small order A, small order R
	true,  // 1: small order A, mixed order R
	true,  // 2: mixed order A, small order R
	true,  // 3: mixed order A, mixed order R
	false, // 4: cofactored verify
	false, // 5: cofactored verify computes 8(hA) instead of (8h mod L)A
	false, // 6: non-canonical S (S > L)
	false, // 7: non-canonical S (S >> L)
	false, // 8: mixed order A, non-canonical small order R (accepted if R reduced before hashing)
	false, // 9: mixed order A, non-canonical small order R (accepted if R not reduced before hashing)
	false, // 10: non-canonical small order A, mixed order R (accepted if cofactored or cofactor-less and A reduced before hashing)
	true,  // 11: non-canonical small order A, mixed order R (accepted if cofactored or cofactor-less and A not reduced before hashing)
}

var speccheckExpectedResultsFIPS_186_5 = []bool{
	true,  // 0: small order A, small order R
	true,  // 1: small order A, mixed order R
	true,  // 2: mixed order A, small order R
	true,  // 3: mixed order A, mixed order R
	true,  // 4: cofactored verify
	true,  // 5: cofactored verify computes 8(hA) instead of (8h mod L)A
	false, // 6: non-canonical S (S > L)
	false, // 7: non-canonical S (S >> L)
	false, // 8: mixed order A, non-canonical small order R (accepted if R reduced before hashing)
	false, // 9: mixed order A, non-canonical small order R (accepted if R not reduced before hashing)
	false, // 10: non-canonical small order A, mixed order R (accepted if cofactored or cofactor-less and A reduced before hashing)
	false, // 11: non-canonical small order A, mixed order R (accepted if cofactored or cofactor-less and A not reduced before hashing)
}

var speccheckExpectedResultsZIP_215 = []bool{
	true,  // 0: small order A, small order R
	true,  // 1: small order A, mixed order R
	true,  // 2: mixed order A, small order R
	true,  // 3: mixed order A, mixed order R
	true,  // 4: cofactored verify
	true,  // 5: cofactored verify computes 8(hA) instead of (8h mod L)A
	false, // 6: non-canonical S (S > L)
	false, // 7: non-canonical S (S >> L)
	false, // 8: mixed order A, non-canonical small order R (accepted if R reduced before hashing)
	true,  // 9: mixed order A, non-canonical small order R (accepted if R not reduced before hashing)
	true,  // 10: non-canonical small order A, mixed order R (accepted if cofactored or cofactor-less and A reduced before hashing)
	true,  // 11: non-canonical small order A, mixed order R (accepted if cofactored or cofactor-less and A not reduced before hashing)
}

type speccheckTestVector struct {
	Message   string `json:"message"`
	PublicKey string `json:"pub_key"`
	Signature string `json:"signature"`
}

func (v *speccheckTestVector) toComponents(t *testing.T) ([]byte, PublicKey, []byte, error) {
	var pk PublicKey

	msg := testhelpers.MustUnhex(t, v.Message)
	rawPk := testhelpers.MustUnhex(t, v.PublicKey)
	sig := testhelpers.MustUnhex(t, v.Signature)
	if len(rawPk) != PublicKeySize {
		return nil, pk, nil, fmt.Errorf("invalid public key size")
	}
	if len(sig) != SignatureSize {
		return nil, pk, nil, fmt.Errorf("invalid signature size")
	}

	pk = PublicKey(rawPk)

	return msg, pk, sig, nil
}

func (v *speccheckTestVector) Run(t *testing.T, impl verificationImpl, opts *Options) bool {
	msg, pk, sig, err := v.toComponents(t)
	if err != nil {
		t.Fatal(err)
	}

	expPub, err := NewExpandedPublicKey(pk)
	if err != nil {
		t.Fatalf("NewExpandedPublicKey: %v", err)
	}

	var sigOk bool
	switch impl {
	case implVanilla:
		sigOk = VerifyWithOptions(pk, msg, sig, opts)
	case implBatch, implExpandedBatch:
		v := NewBatchVerifier()
		for i := 0; i < testBatchSize; i++ {
			switch impl {
			case implBatch:
				v.AddWithOptions(pk, msg, sig, opts)
			case implExpandedBatch:
				v.AddExpandedWithOptions(expPub, msg, sig, opts)
			}
		}

		var valid []bool
		sigOk, valid = v.Verify(rand.Reader)
		for i, v := range valid {
			if v != sigOk {
				t.Fatalf("sigOk != valid[%d]", i)
			}
		}

		if len(valid) != testBatchSize {
			t.Fatalf("len(valid) != testBatchSize: %v", len(valid))
		}
	case implExpanded:
		sigOk = VerifyExpandedWithOptions(expPub, msg, sig, opts)
	default:
		return false
	}

	return sigOk
}

func TestSpeccheck(t *testing.T) {
	f, err := os.Open("testdata/speccheck_cases.json.gz")
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()
	rd, err := gzip.NewReader(f)
	if err != nil {
		t.Fatal(err)
	}
	defer rd.Close()

	var testVectors []speccheckTestVector

	dec := json.NewDecoder(rd)
	if err = dec.Decode(&testVectors); err != nil {
		t.Fatal(err)
	}

	doTestCases := func(n string, opts *Options, expectedResults []bool) {
		for idx, tc := range testVectors {
			expected := expectedResults[idx]
			t.Run(fmt.Sprintf("%s/%d", n, idx), func(t *testing.T) {
				if sigOk := tc.Run(t, implVanilla, opts); sigOk != expected {
					t.Fatalf("behavior mismatch: %v (expected %v)", sigOk, expected)
				}
			})
			t.Run(fmt.Sprintf("%s/Batch/%d", n, idx), func(t *testing.T) {
				if sigOk := tc.Run(t, implBatch, opts); sigOk != expected {
					t.Fatalf("behavior mismatch: %v (expected %v)", sigOk, expected)
				}
			})
			t.Run(fmt.Sprintf("%s/Expanded/%d", n, idx), func(t *testing.T) {
				if sigOk := tc.Run(t, implExpanded, opts); sigOk != expected {
					t.Fatalf("behavior mismatch: %v (expected %v)", sigOk, expected)
				}
			})
			t.Run(fmt.Sprintf("%s/Expanded_Batch/%d", n, idx), func(t *testing.T) {
				if sigOk := tc.Run(t, implExpandedBatch, opts); sigOk != expected {
					t.Fatalf("behavior mismatch: %v (expected %v)", sigOk, expected)
				}
			})
		}
	}

	t.Run("Default", func(t *testing.T) {
		doTestCases("Default", optionsDefault, speccheckExpectedResults)
	})
	t.Run("StdLib", func(t *testing.T) {
		doTestCases("StdLib", &Options{Verify: VerifyOptionsStdLib}, speccheckExpectedResultsStdLib)
	})
	t.Run("FIPS-186-5", func(t *testing.T) {
		doTestCases("FIPS-186-5", &Options{Verify: VerifyOptionsFIPS_186_5}, speccheckExpectedResultsFIPS_186_5)
	})
	t.Run("ZIP-215", func(t *testing.T) {
		doTestCases("ZIP-215", &Options{Verify: VerifyOptionsZIP_215}, speccheckExpectedResultsZIP_215)
	})
}
