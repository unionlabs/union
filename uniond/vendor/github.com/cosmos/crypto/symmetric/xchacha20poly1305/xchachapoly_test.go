package xchacha20poly1305

import (
	"bytes"
	cr "crypto/rand"
	mr "math/rand"
	"testing"
)

// The following test is taken from
// https://github.com/golang/crypto/blob/master/chacha20poly1305/chacha20poly1305_test.go#L69
// It requires the below copyright notice, where "this source code" refers to the following function.
// Copyright 2016 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found at the bottom of this file.
func TestRandom(t *testing.T) {
	// Some random tests to verify Open(Seal) == Plaintext
	for i := 0; i < 256; i++ {
		var nonce [24]byte
		var key [32]byte

		al := mr.Intn(128)
		pl := mr.Intn(16384)
		ad := make([]byte, al)
		plaintext := make([]byte, pl)
		_, err := cr.Read(key[:])
		if err != nil {
			t.Errorf("error on read: %v", err)
		}
		_, err = cr.Read(nonce[:])
		if err != nil {
			t.Errorf("error on read: %v", err)
		}
		_, err = cr.Read(ad)
		if err != nil {
			t.Errorf("error on read: %v", err)
		}
		_, err = cr.Read(plaintext)
		if err != nil {
			t.Errorf("error on read: %v", err)
		}

		aead, err := New(key[:])
		if err != nil {
			t.Fatal(err)
		}

		ct := aead.Seal(nil, nonce[:], plaintext, ad)

		plaintext2, err := aead.Open(nil, nonce[:], ct, ad)
		if err != nil {
			t.Errorf("random #%d: Open failed", i)
			continue
		}

		if !bytes.Equal(plaintext, plaintext2) {
			t.Errorf("random #%d: plaintext's don't match: got %x vs %x", i, plaintext2, plaintext)
			continue
		}

		if len(ad) > 0 {
			alterAdIdx := mr.Intn(len(ad))
			ad[alterAdIdx] ^= 0x80
			if _, err := aead.Open(nil, nonce[:], ct, ad); err == nil {
				t.Errorf("random #%d: Open was successful after altering additional data", i)
			}
			ad[alterAdIdx] ^= 0x80
		}

		alterNonceIdx := mr.Intn(aead.NonceSize())
		nonce[alterNonceIdx] ^= 0x80
		if _, err := aead.Open(nil, nonce[:], ct, ad); err == nil {
			t.Errorf("random #%d: Open was successful after altering nonce", i)
		}
		nonce[alterNonceIdx] ^= 0x80

		alterCtIdx := mr.Intn(len(ct))
		ct[alterCtIdx] ^= 0x80
		if _, err := aead.Open(nil, nonce[:], ct, ad); err == nil {
			t.Errorf("random #%d: Open was successful after altering ciphertext", i)
		}
		ct[alterCtIdx] ^= 0x80
	}
}

// AFOREMENTIONED LICENSE
// Copyright (c) 2009 The Go Authors. All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//    * Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//    * Redistributions in binary form must reproduce the above
// copyright notice, this list of conditions and the following disclaimer
// in the documentation and/or other materials provided with the
// distribution.
//    * Neither the name of Google Inc. nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
