// Copyright 2019 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package wycheproof

import (
	"crypto/ecdsa"
	"math/big"
	"testing"

	"golang.org/x/crypto/cryptobyte"
	"golang.org/x/crypto/cryptobyte/asn1"
)

func TestECDSA(t *testing.T) {
	type ASNSignatureTestVector struct {
		// A brief description of the test case
		Comment string `json:"comment"`
		// A list of flags
		Flags []string `json:"flags"`
		// The message to sign
		Msg string `json:"msg"`
		// Test result
		Result string `json:"result"`
		// An ASN.1 encoded signature for msg
		Sig string `json:"sig"`
		// Identifier of the test case
		TcID int `json:"tcId"`
	}

	type ECPublicKey struct {
		// The EC group used by this public key
		Curve interface{} `json:"curve"`
	}

	type ECDSATestGroup struct {
		// Unencoded EC public key
		Key *ECPublicKey `json:"key"`
		// DER encoded public key
		KeyDER string `json:"keyDer"`
		// the hash function used for ECDSA
		SHA   string                    `json:"sha"`
		Tests []*ASNSignatureTestVector `json:"tests"`
	}

	type Root struct {
		TestGroups []*ECDSATestGroup `json:"testGroups"`
	}

	flagsShouldPass := map[string]bool{
		// An encoded ASN.1 integer missing a leading zero is invalid, but
		// accepted by some implementations.
		"MissingZero": false,
		// A signature using a weaker hash than the EC params is not a security
		// risk, as long as the hash is secure.
		// https://www.imperialviolet.org/2014/05/25/strengthmatching.html
		"WeakHash": true,
	}

	// supportedCurves is a map of all elliptic curves supported
	// by crypto/elliptic, which can subsequently be parsed and tested.
	supportedCurves := map[string]bool{
		"secp224r1": true,
		"secp256r1": true,
		"secp384r1": true,
		"secp521r1": true,
	}

	var root Root
	readTestVector(t, "ecdsa_test.json", &root)
	for _, tg := range root.TestGroups {
		curve := tg.Key.Curve.(string)
		if !supportedCurves[curve] {
			continue
		}
		pub := decodePublicKey(tg.KeyDER).(*ecdsa.PublicKey)
		h := parseHash(tg.SHA).New()
		for _, sig := range tg.Tests {
			h.Reset()
			h.Write(decodeHex(sig.Msg))
			hashed := h.Sum(nil)
			sigBytes := decodeHex(sig.Sig)
			got := ecdsa.VerifyASN1(pub, hashed, sigBytes)
			if want := shouldPass(sig.Result, sig.Flags, flagsShouldPass); got != want {
				t.Errorf("tcid: %d, type: %s, comment: %q, VerifyASN1 wanted success: %t", sig.TcID, sig.Result, sig.Comment, want)
			}

			var r, s big.Int
			var inner cryptobyte.String
			input := cryptobyte.String(sigBytes)
			if !input.ReadASN1(&inner, asn1.SEQUENCE) ||
				!input.Empty() ||
				!inner.ReadASN1Integer(&r) ||
				!inner.ReadASN1Integer(&s) ||
				!inner.Empty() {
				continue
			}
			got = ecdsa.Verify(pub, hashed, &r, &s)
			if want := shouldPass(sig.Result, sig.Flags, flagsShouldPass); got != want {
				t.Errorf("tcid: %d, type: %s, comment: %q, Verify wanted success: %t", sig.TcID, sig.Result, sig.Comment, want)
			}
		}
	}
}
