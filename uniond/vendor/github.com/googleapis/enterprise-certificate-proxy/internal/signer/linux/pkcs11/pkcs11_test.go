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

package pkcs11

import (
	"bytes"
	"crypto"
	"crypto/rsa"
	"flag"
	"testing"
)

const (
	testModule  = "/usr/lib/softhsm/libsofthsm2.so"
	testLabel   = "Demo Object"
	testUserPin = "0000"
)

var testSlot = flag.String("testSlot", "", "libsofthsm2 slot location")

func makeTestKey() (*Key, error) {
	key, err := Cred(testModule, *testSlot, testLabel, testUserPin)
	return key, err
}

func TestParseHexString(t *testing.T) {
	got, err := ParseHexString("0x1739427")
	if err != nil {
		t.Fatalf("ParseHexString error: %v", err)
	}
	want := uint32(0x1739427)
	if got != want {
		t.Errorf("Expected result is %v, got: %v", want, got)
	}
}

func TestParseHexStringFailure(t *testing.T) {
	_, err := ParseHexString("abcdefgh")
	if err == nil {
		t.Error("Expected error but got nil")
	}
}

func TestCredLinux(t *testing.T) {
	key, err := makeTestKey()
	if err != nil {
		t.Errorf("Cred error: %q", err)
	}
	defer key.Close()
}

func BenchmarkEncryptRSA(b *testing.B) {
	msg := "Plain text to encrypt"
	bMsg := []byte(msg)
	key, errCred := makeTestKey()
	if errCred != nil {
		b.Errorf("Cred error: %q", errCred)
		return
	}
	defer key.Close()
	b.Run("encryptRSA Crypto", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			_, errEncrypt := key.encryptRSA(bMsg)
			if errEncrypt != nil {
				b.Errorf("EncryptRSA error: %q", errEncrypt)
				return
			}
		}
	})
}

func TestEncrypt(t *testing.T) {
	key, errCred := makeTestKey()
	if errCred != nil {
		t.Errorf("Cred error: %q", errCred)
		return
	}
	defer key.Close()
	msg := "Plain text to encrypt"
	bMsg := []byte(msg)
	_, err := key.Encrypt(bMsg, crypto.SHA1)
	if err != nil {
		t.Errorf("Encrypt error: %q", err)
	}
}

func TestDecrypt(t *testing.T) {
	key, errCred := makeTestKey()
	if errCred != nil {
		t.Errorf("Cred error: %q", errCred)
		return
	}
	defer key.Close()
	msg := "Plain text to encrypt"
	bMsg := []byte(msg)
	// Softhsm only supports SHA1
	ciphertext, err := key.Encrypt(bMsg, crypto.SHA1)
	if err != nil {
		t.Errorf("Encrypt error: %q", err)
	}
	decrypted, err := key.Decrypt(ciphertext, &rsa.OAEPOptions{Hash: crypto.SHA1})
	if err != nil {
		t.Fatalf("Decrypt error: %v", err)
	}
	decrypted = bytes.Trim(decrypted, "\x00")
	if string(decrypted) != msg {
		t.Errorf("Decrypt error: expected %q, got %q", msg, string(decrypted))
	}
}
