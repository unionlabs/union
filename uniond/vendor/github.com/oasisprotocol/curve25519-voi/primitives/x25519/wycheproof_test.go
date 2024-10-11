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

package x25519

import (
	"bytes"
	"compress/gzip"
	"encoding/json"
	"fmt"
	"os"
	"testing"

	"github.com/oasisprotocol/curve25519-voi/internal/testhelpers"
)

type wycheproofXDHFlags int

const (
	flagLowOrderPublic wycheproofXDHFlags = iota
	flagNonCanonicalPublic
	flagSmallPublicKey
	flagTwist
	flagZeroSharedSecret
)

type wycheproofTestVectors struct {
	Algorithm  string                `json:"algorithm"`
	Version    string                `json:"generatorVersion"`
	NumTests   int                   `json:"numberOfTests"`
	TestGroups []wycheproofTestGroup `json:"TestGroups"`
}

type wycheproofTestGroup struct {
	Curve string               `json:"string"`
	Type  string               `json:"type"`
	Tests []wycheproofTestCase `json:"tests"`
}

type wycheproofTestCase struct {
	ID      int      `json:"tcId"`
	Comment string   `json:"comment"`
	Public  string   `json:"public"`
	Private string   `json:"private"`
	Shared  string   `json:"shared"`
	Result  string   `json:"result"`
	Flags   []string `json:"flags"`
}

func (tc *wycheproofTestCase) Run(t *testing.T) {
	if tc.Comment != "" {
		t.Logf("%s", tc.Comment)
	}

	publicKey := testhelpers.MustUnhex(t, tc.Public)
	privateKey := testhelpers.MustUnhex(t, tc.Private)
	sharedKey := testhelpers.MustUnhex(t, tc.Shared)

	flags := make(map[wycheproofXDHFlags]bool)
	for _, s := range tc.Flags {
		switch s {
		case "LowOrderPublic":
			flags[flagLowOrderPublic] = true
		case "NonCanonicalPublic":
			flags[flagNonCanonicalPublic] = true
		case "SmallPublicKey":
			flags[flagSmallPublicKey] = true
		case "Twist":
			flags[flagTwist] = true
		case "ZeroSharedSecret":
			flags[flagZeroSharedSecret] = true
		}
	}

	// First test the raw "Deprecated" ScalarMult routine.
	var dst, in, base [32]byte
	copy(in[:], privateKey)
	copy(base[:], publicKey)
	ScalarMult(&dst, &in, &base)

	if !bytes.Equal(dst[:], sharedKey) {
		t.Fatalf("failed ScalarMult(dst, priv, pub): %x (expected %x)", dst[:], sharedKey)
	}

	// The "new" X25519 routine enforces contributory behavior, with an
	// error message that appears as if it is rejecting low order public
	// keys.  The check is for an all zero shared secret, so use that flag
	// though both are excluseively set as a pair.
	shouldFail := flags[flagZeroSharedSecret]

	out, err := X25519(privateKey, publicKey)
	switch shouldFail {
	case true:
		if err == nil {
			t.Fatalf("X25519(priv, pub) returned no error when it should fail")
		}
	case false:
		if err != nil {
			t.Fatalf("failed X25519(priv, pub): %v", err)
		}
		if !bytes.Equal(out, sharedKey) {
			t.Fatalf("failed X25519(priv, pub): %x (expected %x)", out, sharedKey)
		}
	}
}

func TestWycheproof(t *testing.T) {
	f, err := os.Open("testdata/x25519_test.json.gz")
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()
	rd, err := gzip.NewReader(f)
	if err != nil {
		t.Fatal(err)
	}
	defer rd.Close()

	var testVectors wycheproofTestVectors

	dec := json.NewDecoder(rd)
	if err = dec.Decode(&testVectors); err != nil {
		t.Fatal(err)
	}

	t.Logf("Wycheproof Version: %s", testVectors.Version)

	var numTests int
	for _, group := range testVectors.TestGroups {
		for _, testCase := range group.Tests {
			n := fmt.Sprintf("TestCase/%d", testCase.ID)
			t.Run(n, func(t *testing.T) {
				testCase.Run(t)
			})
			numTests++
		}
	}
	if numTests != testVectors.NumTests {
		t.Errorf("unexpected number of tests ran: %d (expected %d)", numTests, testVectors.NumTests)
	}
}
