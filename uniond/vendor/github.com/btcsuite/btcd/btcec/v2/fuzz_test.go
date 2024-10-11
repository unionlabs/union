//go:build gofuzz || go1.18

// Copyright (c) 2013-2017 The btcsuite developers
// Copyright (c) 2015-2022 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package btcec

import (
	"encoding/hex"
	"testing"
)

func FuzzParsePubKey(f *testing.F) {
	// 1. Seeds from pubkey tests.
	for _, test := range pubKeyTests {
		if test.isValid {
			f.Add(test.key)
		}
	}

	// 2. Seeds from recovery tests.
	var recoveryTestPubKeys = []string{
		"04E32DF42865E97135ACFB65F3BAE71BDC86F4D49150AD6A440B6F15878109880A0A2B2667F7E725CEEA70C673093BF67663E0312623C8E091B13CF2C0F11EF652",
		"04A7640409AA2083FDAD38B2D8DE1263B2251799591D840653FB02DBBA503D7745FCB83D80E08A1E02896BE691EA6AFFB8A35939A646F1FC79052A744B1C82EDC3",
	}
	for _, pubKey := range recoveryTestPubKeys {
		seed, err := hex.DecodeString(pubKey)
		if err != nil {
			f.Fatal(err)
		}
		f.Add(seed)
	}

	// Now run the fuzzer.
	f.Fuzz(func(t *testing.T, input []byte) {
		key, err := ParsePubKey(input)
		if key == nil && err == nil {
			panic("key==nil && err==nil")
		}
		if key != nil && err != nil {
			panic("key!=nil yet err!=nil")
		}
	})
}
