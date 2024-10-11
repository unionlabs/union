// Copyright 2022 Google LLC.
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

//go:build darwin && cgo
// +build darwin,cgo

package keychain

import (
	"bytes"
	"crypto"
	"crypto/rsa"
	"testing"
	"unsafe"
)

const testIssuer = "TestIssuer"

func TestKeychainError(t *testing.T) {
	tests := []struct {
		e    keychainError
		want string
	}{
		{e: keychainError(0), want: "No error."},
		{e: keychainError(-4), want: "Function or operation not implemented."},
	}

	for i, test := range tests {
		if got := test.e.Error(); got != test.want {
			t.Errorf("test %d: %#v.Error() = %q, want %q", i, test.e, got, test.want)
		}
	}
}

func TestBytesToCFDataRoundTrip(t *testing.T) {
	want := []byte("an arbitrary and yet coherent byte slice!")
	d := bytesToCFData(want)
	defer cfRelease(unsafe.Pointer(d))
	if got := cfDataToBytes(d); !bytes.Equal(got, want) {
		t.Errorf("bytesToCFData -> cfDataToBytes\ngot  %x\nwant %x", got, want)
	}
}

func TestEncrypt(t *testing.T) {
	key, err := Cred(testIssuer)
	if err != nil {
		t.Errorf("Cred: got %v, want nil err", err)
		return
	}
	plaintext := []byte("Plain text to encrypt")
	_, err = key.Encrypt(plaintext, crypto.SHA256)
	if err != nil {
		t.Errorf("Encrypt: got %v, want nil err", err)
		return
	}
}

func BenchmarkEncrypt(b *testing.B) {
	key, err := Cred(testIssuer)
	if err != nil {
		b.Errorf("Cred: got %v, want nil err", err)
		return
	}
	plaintext := []byte("Plain text to encrypt")
	for i := 0; i < b.N; i++ {
		_, err := key.Encrypt(plaintext, crypto.SHA256)
		if err != nil {
			b.Errorf("Encrypt: got %v, want nil err", err)
		}
	}
}

func TestDecrypt(t *testing.T) {
	key, err := Cred(testIssuer)
	if err != nil {
		t.Errorf("Cred: got %v, want nil err", err)
		return
	}
	byteSlice := []byte("Plain text to encrypt")
	ciphertext, _ := key.Encrypt(byteSlice, crypto.SHA256)
	plaintext, err := key.Decrypt(ciphertext, &rsa.OAEPOptions{Hash: crypto.SHA256})
	if err != nil {
		t.Errorf("Decrypt: got %v, want nil err", err)
		return
	}
	if !bytes.Equal(byteSlice, plaintext) {
		t.Errorf("Decryption message does not match original: got %v, want %v", plaintext, byteSlice)
	}
}

func BenchmarkDecrypt(b *testing.B) {
	key, err := Cred(testIssuer)
	if err != nil {
		b.Errorf("Cred: got %v, want nil err", err)
		return
	}
	byteSlice := []byte("Plain text to encrypt")
	ciphertext, _ := key.Encrypt(byteSlice, crypto.SHA256)
	for i := 0; i < b.N; i++ {
		_, err := key.Decrypt(ciphertext, &rsa.OAEPOptions{Hash: crypto.SHA256})
		if err != nil {
			b.Errorf("Decrypt: got %v, want nil err", err)
		}
	}
}
