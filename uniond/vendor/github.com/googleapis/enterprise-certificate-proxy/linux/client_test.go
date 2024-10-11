// Copyright 2023 Google LLC.
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package linux

import (
	"bytes"
	"crypto"
	"crypto/rsa"
	"flag"
	"testing"
)

const (
	testModule  = "/usr/local/lib/softhsm/libsofthsm2.so"
	testLabel   = "Demo Object"
	testUserPin = "0000"
)

var testSlot = *flag.String("testSlot", "", "libsofthsm2 slot location")

func TestEncrypt(t *testing.T) {
	sk, err := NewSecureKey(testModule, testSlot, testLabel, testUserPin)
	if err != nil {
		t.Errorf("Client Encrypt: error generating secure key, %q", err)
	}
	message := "Plain text to encrypt"
	bMessage := []byte(message)
	//Softhsm only supports SHA1
	_, err = sk.Encrypt(nil, bMessage, crypto.SHA1)
	if err != nil {
		t.Errorf("Client Encrypt error: %q", err)
	}
}

func TestDecrypt(t *testing.T) {
	sk, err := NewSecureKey(testModule, testSlot, testLabel, testUserPin)
	if err != nil {
		t.Errorf("Client Decrypt: error generating secure key, %q", err)
	}
	message := "Plain text to encrypt"
	bMessage := []byte(message)
	//Softhsm only supports SHA1
	cipher, err := sk.Encrypt(nil, bMessage, crypto.SHA1)
	if err != nil {
		t.Errorf("Client Encrypt error: %q", err)
	}
	decrypted, err := sk.Decrypt(nil, cipher, &rsa.OAEPOptions{Hash: crypto.SHA1})
	if err != nil {
		t.Fatalf("Client Decrypt error: %v", err)
	}
	decrypted = bytes.Trim(decrypted, "\x00")
	if string(decrypted) != message {
		t.Errorf("Client Decrypt error: expected %q, got %q", message, string(decrypted))
	}
}
