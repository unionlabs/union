// Copyright (c) 2019 Oasis Labs Inc. All rights reserved.
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
	"bytes"
	"compress/gzip"
	"encoding/json"
	"fmt"
	"os"
	"strings"
	"testing"

	"github.com/oasisprotocol/curve25519-voi/internal/testhelpers"
)

type wycheproofTestVectors struct {
	Algorithm  string                `json:"algorithm"`
	Version    string                `json:"generatorVersion"`
	NumTests   int                   `json:"numberOfTests"`
	TestGroups []wycheproofTestGroup `json:"TestGroups"`
}

type wycheproofTestGroup struct {
	Key   wycheproofTestKey    `json:"key"`
	Tests []wycheproofTestCase `json:"tests"`
}

type wycheproofTestKey struct {
	Curve      string `json:"curve"`
	KeySize    int    `json:"keySize"`
	PublicKey  string `json:"pk"`
	PrivateKey string `json:"sk"`
	Type       string `json:"type"`
}

func (k *wycheproofTestKey) Keys(t *testing.T) (PublicKey, PrivateKey, error) {
	rawPk := testhelpers.MustUnhex(t, k.PublicKey)
	if len(rawPk) != PublicKeySize {
		return nil, nil, fmt.Errorf("invalid public key size")
	}
	rawSeed := testhelpers.MustUnhex(t, k.PrivateKey)
	if len(rawSeed) != SeedSize {
		return nil, nil, fmt.Errorf("invalid private key seed size")
	}

	privKey := NewKeyFromSeed(rawSeed)
	pubKeyCheck := privKey.Public().(PublicKey)
	if !bytes.Equal(pubKeyCheck[:], rawPk) {
		return nil, nil, fmt.Errorf("derived public key does not match vectors")
	}

	return PublicKey(rawPk), privKey, nil
}

type wycheproofTestCase struct {
	ID        int    `json:"tcId"`
	Comment   string `json:"comment"`
	Message   string `json:"msg"`
	Signature string `json:"sig"`
	Result    string `json:"result"`
}

func (tc *wycheproofTestCase) Run(t *testing.T, pubKey PublicKey, privKey PrivateKey) {
	if tc.Comment != "" {
		t.Logf("%s", tc.Comment)
	}

	msg := testhelpers.MustUnhex(t, tc.Message)
	sig := testhelpers.MustUnhex(t, tc.Signature)

	var expectedResult bool
	switch strings.ToLower(tc.Result) {
	case "invalid":
	case "valid":
		expectedResult = true
	default:
		t.Fatalf("failed to parse expected result: '%v'", tc.Result)
	}

	// If the test case has a valid signature, check to see if we can
	// reproduce it.
	if expectedResult == true {
		derivedSig := Sign(privKey, msg)
		if !bytes.Equal(sig, derivedSig) {
			t.Errorf("failed to re-generate signature: %x (expected %x)",
				derivedSig,
				sig,
			)
		}
	}

	sigOk := Verify(pubKey, msg, sig)
	if sigOk != expectedResult {
		t.Errorf("signature validation result mismatch: %v (expected %v)",
			sigOk,
			expectedResult,
		)
	}

	// Ensure that the expanded version also does the right thing.
	expPub, err := NewExpandedPublicKey(pubKey)
	if err != nil {
		t.Fatalf("NewExpandedPublicKey: %v", err)
	}
	sigOk = VerifyExpandedWithOptions(expPub, msg, sig, optionsDefault)
	if sigOk != expectedResult {
		t.Errorf("expanded signature validation result mismatch: %v (expected %v)",
			sigOk,
			expectedResult,
		)
	}
}

func TestWycheproof(t *testing.T) {
	f, err := os.Open("testdata/eddsa_test.json.gz")
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
	for i, group := range testVectors.TestGroups {
		pubKey, privKey, err := group.Key.Keys(t)
		if err != nil {
			t.Errorf("failed to parse keys for test group %d: %v", i, err)
			continue
		}

		for _, testCase := range group.Tests {
			n := fmt.Sprintf("TestCase/%d", testCase.ID)
			t.Run(n, func(t *testing.T) {
				testCase.Run(t, pubKey, privKey)
			})
			numTests++
		}
	}
	if numTests != testVectors.NumTests {
		t.Errorf("unexpected number of tests ran: %d (expected %d)", numTests, testVectors.NumTests)
	}
}
