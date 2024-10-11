// Copyright (c) 2020 Oasis Labs Inc. All rights reserved.
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
	"crypto/rand"
	"encoding/json"
	"fmt"
	"os"
	"testing"

	"github.com/oasisprotocol/curve25519-voi/internal/testhelpers"
)

type zip215TestVector [2]string

func (tc zip215TestVector) Run(t *testing.T, isBatch, isZIP215 bool) {
	msg := []byte("Zcash")
	rawPk := testhelpers.MustUnhex(t, tc[0])
	sig := testhelpers.MustUnhex(t, tc[1])

	pk := PublicKey(rawPk)
	var opts Options
	switch isZIP215 {
	case true:
		opts.Verify = VerifyOptionsZIP_215
	case false:
		opts.Verify = VerifyOptionsDefault
	}

	var sigOk bool
	switch isBatch {
	case false:
		sigOk = VerifyWithOptions(pk, msg, sig, &opts)
	case true:
		v := NewBatchVerifier()

		for i := 0; i < testBatchSize; i++ {
			v.AddWithOptions(pk, msg, sig, &opts)
		}

		var valid []bool
		sigOk, valid = v.Verify(rand.Reader)
		for i, v := range valid {
			if v != sigOk {
				t.Fatalf("sigOk != valid[%d]", i)
			}
		}
	}

	// The ZIP-215 test vectors are cases that a ZIP-215 verifier should
	// pass, and the default verification algorithm should reject.
	if sigOk != isZIP215 {
		t.Fatalf("failed to verify signature")
	}
}

func TestZIP215(t *testing.T) {
	f, err := os.Open("testdata/zip215.json.gz")
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()
	rd, err := gzip.NewReader(f)
	if err != nil {
		t.Fatal(err)
	}
	defer rd.Close()

	var testVectors []zip215TestVector
	dec := json.NewDecoder(rd)
	if err = dec.Decode(&testVectors); err != nil {
		t.Fatal(err)
	}

	for idx, tc := range testVectors {
		n := fmt.Sprintf("TestCase_%d", idx)
		t.Run(n, func(t *testing.T) {
			tc.Run(t, false, false)
		})
		t.Run(n+"_Batch", func(t *testing.T) {
			tc.Run(t, true, false)
		})

		n = n + "_ZIP215"
		t.Run(n, func(t *testing.T) {
			tc.Run(t, false, true)
		})
		t.Run(n+"_Batch", func(t *testing.T) {
			tc.Run(t, true, true)
		})
	}
}
