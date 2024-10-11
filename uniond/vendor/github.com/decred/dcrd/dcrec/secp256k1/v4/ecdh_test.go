// Copyright (c) 2015-2016 The btcsuite developers
// Copyright (c) 2015-2017 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package secp256k1

import (
	"bytes"
	"testing"
)

func TestGenerateSharedSecret(t *testing.T) {
	privKey1, err := GeneratePrivateKey()
	if err != nil {
		t.Errorf("private key generation error: %s", err)
		return
	}
	privKey2, err := GeneratePrivateKey()
	if err != nil {
		t.Errorf("private key generation error: %s", err)
		return
	}

	pubKey1 := privKey1.PubKey()
	pubKey2 := privKey2.PubKey()
	secret1 := GenerateSharedSecret(privKey1, pubKey2)
	secret2 := GenerateSharedSecret(privKey2, pubKey1)
	if !bytes.Equal(secret1, secret2) {
		t.Errorf("ECDH failed, secrets mismatch - first: %x, second: %x",
			secret1, secret2)
	}
}
