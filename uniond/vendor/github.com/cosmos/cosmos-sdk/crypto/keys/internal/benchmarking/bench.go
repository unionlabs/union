package benchmarking

import (
	"crypto/rand"
	"io"
	"testing"

	"github.com/cosmos/cosmos-sdk/crypto/types"
)

// The code in this file is adapted from agl/ed25519.
// As such it is under the following license.
// Copyright 2012 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found at the bottom of this file.

// BenchmarkKeyGeneration benchmarks the given key generation algorithm using
// a dummy reader.
func BenchmarkKeyGeneration(b *testing.B, generateKey func(reader io.Reader) types.PrivKey) {
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		generateKey(rand.Reader)
	}
}

// BenchmarkSigning benchmarks the given signing algorithm using
// the provided privkey.
func BenchmarkSigning(b *testing.B, priv types.PrivKey) {
	message := []byte("Hello, world!")
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, err := priv.Sign(message)
		if err != nil {
			b.FailNow()
		}
	}
}

// BenchmarkVerification benchmarks the given verification algorithm using
// the provided privkey on a constant message.
func BenchmarkVerification(b *testing.B, priv types.PrivKey) {
	pub := priv.PubKey()
	// use a short message, so this time doesn't get dominated by hashing.
	message := []byte("Hello, world!")
	signature, err := priv.Sign(message)
	if err != nil {
		b.Fatal(err)
	}
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		pub.VerifySignature(message, signature)
	}
}

// Below is the aforementioned license.

// Copyright (c) 2012 The Go Authors. All rights reserved.

// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:

//    * Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//    * Redistributions in binary form must reproduce the above
// copyright notice, this list of conditions and the following disclaimer
// in the documentation and/or other materials provided with the
// distribution.
//    * Neither the name of Google Inc. nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.

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
