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

package darwin

import (
	"bytes"
	"crypto"
	"crypto/rsa"
	"testing"
)

const testIssuer = "TestIssuer"

func TestClientEncrypt(t *testing.T) {
	secureKey, err := NewSecureKey(testIssuer)
	if err != nil {
		t.Errorf("Cred: got %v, want nil err", err)
		return
	}
	plaintext := []byte("Plain text to encrypt")
	_, err = secureKey.Encrypt(nil, plaintext, crypto.SHA256)
	if err != nil {
		t.Errorf("Client API encryption: got %v, want nil err", err)
		return
	}
}

func TestClientDecrypt(t *testing.T) {
	secureKey, err := NewSecureKey(testIssuer)
	if err != nil {
		t.Errorf("Cred: got %v, want nil err", err)
		return
	}
	byteSlice := []byte("Plain text to encrypt")
	ciphertext, _ := secureKey.Encrypt(nil, byteSlice, crypto.SHA256)
	plaintext, err := secureKey.Decrypt(nil, ciphertext, &rsa.OAEPOptions{Hash: crypto.SHA256})
	if err != nil {
		t.Errorf("Client API decryption: got %v, want nil err", err)
		return
	}
	if !bytes.Equal(byteSlice, plaintext) {
		t.Errorf("Decryption message does not match original: got %v, want %v", plaintext, byteSlice)
	}
}
