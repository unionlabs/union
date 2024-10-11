/*
 *
 * Copyright 2021 Google LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package aeadcrypter

import (
	"bytes"
	"testing"

	"github.com/google/s2a-go/internal/record/internal/aeadcrypter/testutil"
)

// fakeAEAD is a fake implementation of an AEAD interface used for testing.
type fakeAEAD struct{}

func (*fakeAEAD) NonceSize() int                                  { return NonceSize }
func (*fakeAEAD) Overhead() int                                   { return TagSize }
func (*fakeAEAD) Seal(_, _, plaintext, _ []byte) []byte           { return plaintext }
func (*fakeAEAD) Open(_, _, ciphertext, _ []byte) ([]byte, error) { return ciphertext, nil }

type encryptDecryptTestVector struct {
	desc   string
	nonce  []byte
	outErr bool
}

func TestSliceForAppend(t *testing.T) {
	for _, tc := range []struct {
		desc  string
		inBuf []byte
		n     int
	}{
		{
			desc: "nil buf and zero length",
		},
		{
			desc: "nil buf and non-zero length",
			n:    5,
		},
		{
			desc:  "non-empty buf and zero length",
			inBuf: testutil.Dehex("1111111111"),
		},
		{
			desc:  "non-empty buf and non-zero length",
			inBuf: testutil.Dehex("1111111111"),
			n:     5,
		},
		{
			desc:  "test slice capacity pre allocated",
			inBuf: make([]byte, 0, 5),
			n:     5,
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			head, tail := sliceForAppend(tc.inBuf, tc.n)
			// Check that the resulting head buffer starts with the same byte
			// sequence as the input buffer.
			if got, want := head, tc.inBuf; !bytes.HasPrefix(head, tc.inBuf) {
				t.Errorf("sliceForAppend(%v, %v).head = %v, want %v", tc.inBuf, tc.n, got, want)
			}
			// Check that the length of the resulting head buffer is equal
			// to the initial buffer + the additional length requested.
			if got, want := len(head), len(tc.inBuf)+tc.n; got != want {
				t.Errorf("sliceForAppend(%v, %v).tail = %v, want %v", tc.inBuf, tc.n, got, want)
			}
			// Check that the length of the resulting tail buffer is what was
			// requested.
			if got, want := len(tail), tc.n; got != want {
				t.Errorf("sliceForAppend(%v, %v).tail = %v, want %v", tc.inBuf, tc.n, got, want)
			}
		})
	}
}

func TestEncrypt(t *testing.T) {
	plaintext := []byte("test")
	for _, tc := range []encryptDecryptTestVector{
		{
			desc:  "valid nonce size",
			nonce: make([]byte, NonceSize),
		},
		{
			desc:   "invalid nonce size",
			nonce:  make([]byte, 1),
			outErr: true,
		},
	} {
		ciphertext, err := encrypt(&fakeAEAD{}, nil, plaintext, tc.nonce, nil)
		if got, want := err == nil, !tc.outErr; got != want {
			t.Fatalf("encrypt(&fakeAEAD{}, nil, %v, %v, nil)=(err=nil)=%v, want %v", plaintext, tc.nonce, got, want)
		}
		if got, want := ciphertext, plaintext; err == nil && !bytes.Equal(got, want) {
			t.Fatalf("encrypt(&fakeAEAD{}, nil, %v, %v, nil) = %v, want %v", plaintext, tc.nonce, got, want)
		}
	}
}

func TestDecrypt(t *testing.T) {
	ciphertext := []byte("test")
	for _, tc := range []encryptDecryptTestVector{
		{
			desc:  "valid nonce size",
			nonce: make([]byte, NonceSize),
		},
		{
			desc:   "invalid nonce size",
			nonce:  make([]byte, 1),
			outErr: true,
		},
	} {
		plaintext, err := decrypt(&fakeAEAD{}, nil, ciphertext, tc.nonce, nil)
		if got, want := err == nil, !tc.outErr; got != want {
			t.Fatalf("decrypt(&fakeAEAD{}, nil, %v, %v, nil)=(err=nil)=%v, want %v", ciphertext, tc.nonce, got, want)
		}
		if got, want := plaintext, ciphertext; err == nil && !bytes.Equal(got, want) {
			t.Fatalf("decrypt(&fakeAEAD{}, nil, %v, %v, nil) = %v, want %v", ciphertext, tc.nonce, got, want)
		}
	}
}
