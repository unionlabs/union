// Copyright (c) 2016 The Go Authors. All rights reserved.
// Copyright (c) 2019 Oasis Labs Inc. All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//   * Redistributions of source code must retain the above copyright
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

package ed25519

import (
	"bytes"
	"compress/gzip"
	"encoding/json"
	"os"
	"testing"

	"github.com/oasisprotocol/curve25519-voi/internal/testhelpers"
)

// This only tests Ed25519ctx, for Ed25519ph, see ed25519_test.go (testSignVerifyHashed).

type ctxTestVector struct {
	Name      string `json:"name"`
	SecretKey string `json:"secret_key"`
	PublicKey string `json:"public_key"`
	Message   string `json:"message"`
	Context   string `json:"context"`
	Signature string `json:"signature"`
}

func (tv *ctxTestVector) Run(t *testing.T) {
	// The Go representation of a raw private key includes the public
	// key, the RFC test vector's idea of such does not.
	rawPrivate := testhelpers.MustUnhex(t, tv.SecretKey+tv.PublicKey)
	rawPublic := testhelpers.MustUnhex(t, tv.PublicKey)

	privateKey := PrivateKey(rawPrivate)
	publicKey := privateKey.Public().(PublicKey)
	if !bytes.Equal(publicKey[:], rawPublic) {
		t.Fatalf("derived public key does not match test vectors")
	}

	msg := testhelpers.MustUnhex(t, tv.Message)
	ctx := testhelpers.MustUnhex(t, tv.Context)

	expectedSig := testhelpers.MustUnhex(t, tv.Signature)

	opts := &Options{
		Context: string(ctx),
	}

	sig, err := privateKey.Sign(nil, msg, opts)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(sig, expectedSig) {
		t.Error("signature doesn't match test vector")
	}
	if !VerifyWithOptions(publicKey, msg, sig, opts) {
		t.Errorf("valid signature rejected")
	}

	wrongMsg := []byte("bad message" + string(msg))
	if VerifyWithOptions(publicKey, wrongMsg, sig, opts) {
		t.Errorf("signature of different message accepted")
	}

	opts.Context = "bad context" + string(ctx)
	if VerifyWithOptions(publicKey, msg, sig, opts) {
		t.Errorf("signature with different context accepted")
	}
}

func TestSignVerifyCtx(t *testing.T) {
	f, err := os.Open("testdata/rfc8032_ctx.json.gz")
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()
	rd, err := gzip.NewReader(f)
	if err != nil {
		t.Fatal(err)
	}
	defer rd.Close()

	var testVectors []ctxTestVector
	dec := json.NewDecoder(rd)
	if err = dec.Decode(&testVectors); err != nil {
		t.Fatal(err)
	}

	for _, v := range testVectors {
		t.Run(v.Name, v.Run)
	}
}
