// Copyright (c) 2021 Oasis Labs Inc. All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
// 1. Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright
// notice, this list of conditions and the following disclaimer in the
// documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS
// IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED
// TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A
// PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED
// TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

package sr25519

import (
	"testing"

	"github.com/oasisprotocol/curve25519-voi/internal/testhelpers"
)

func TestSignS11n(t *testing.T) {
	// Round trip is covered by TestSignVerify.

	var sigUninit Signature
	b, err := sigUninit.MarshalBinary()
	if err != nil {
		t.Fatalf("skUninit.MarshalBinary: %v", err)
	}

	if l := len(b); l != SignatureSize {
		t.Fatalf("invalid serialized sigUninit lenght: %v", l)
	}
	if !isAllZeros(b[:63]) || b[63] != 128 {
		t.Fatalf("invalid serialized sigUnint (Got %v)", b)
	}
}

func TestSignVerify(t *testing.T) {
	kp, err := GenerateKeyPair(nil)
	if err != nil {
		t.Fatalf("failed to GenerateKeyPair: %v", err)
	}

	msg := []byte("I hurt myself today, to see if I still feel")
	sc := NewSigningContext([]byte("test context pls ignore"))
	st := sc.NewTranscriptBytes(msg)

	sig, err := kp.Sign(nil, st)
	if err != nil {
		t.Fatalf("failed to Sign: %v", err)
	}
	sigBytes, err := sig.MarshalBinary()
	if err != nil {
		t.Fatalf("failed to serialize signature: %v", err)
	}

	var vSig Signature
	if err := vSig.UnmarshalBinary(sigBytes); err != nil {
		t.Fatalf("failed to deserialize signature: %v", err)
	}

	// Re-use the same transcript used when signing to test that it is
	// side-effect free.
	if !kp.PublicKey().Verify(st, &vSig) {
		t.Fatalf("failed to verify signature: %v", err)
	}
}

func TestVerifyVector(t *testing.T) {
	// You would figure, that people will learn at some point to provide
	// test vectors, especially given all the pain that's come from
	// edge cases etc with the algorithm this was ostensibly created
	// to replace, but you would be wrong.
	//
	// While I'm ranting about this, you would figure, that people
	// would learn to actually write a detailed specification of the
	// algorithm, but you would be wrong, again.
	//
	// Just steal the test vector from go-schnorrkel for now.  It is
	// probably the only thing that god-forsaken module is good for
	// at this point.

	const (
		pkHex  = "46ebddef8cd9bb167dc30878d7113b7e168e6f0646beffd77d69d39bad76b47a"
		sigHex = "4e172314444b8f820bb54c22e95076f220ed25373e5c178234aa6c211d29271244b947e3ff3418ff6b45fd1df1140c8cbff69fc58ee6dc96df70936a2bb74b82"
		msg    = "this is a message"
	)

	pkBytes := testhelpers.MustUnhex(t, pkHex)
	var pk PublicKey
	if err := pk.UnmarshalBinary(pkBytes); err != nil {
		t.Fatalf("failed to deserialize public key: %v", err)
	}

	sigBytes := testhelpers.MustUnhex(t, sigHex)
	var sig Signature
	if err := sig.UnmarshalBinary(sigBytes); err != nil {
		t.Fatalf("failed to deserialize signature: %v", err)
	}

	sc := NewSigningContext([]byte("substrate"))
	st := sc.NewTranscriptBytes([]byte(msg))
	if !pk.Verify(st, &sig) {
		t.Fatalf("signature failed to verify")
	}

	st = sc.NewTranscriptBytes([]byte("wrong message"))
	if pk.Verify(st, &sig) {
		t.Fatalf("bad signature verified")
	}
}

func makeBenchTranscript(sc *SigningContext) *SigningTranscript {
	msg := []byte("Test message")
	return sc.NewTranscriptBytes(msg)
}

func BenchmarkSigning(b *testing.B) {
	kp, err := GenerateKeyPair(nil)
	if err != nil {
		b.Fatalf("failed to GenerateKeyPair: %v", err)
	}

	sc := NewSigningContext([]byte("benchmark-signature"))

	b.ReportAllocs()
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		st := makeBenchTranscript(sc)
		sig, err := kp.Sign(nil, st)
		if err != nil {
			b.Fatalf("failed to Sign: %v", err)
		}

		b.StopTimer()
		if !kp.PublicKey().Verify(st, sig) {
			b.Fatalf("Verify failed")
		}
		b.StartTimer()
	}
}

func BenchmarkVerification(b *testing.B) {
	kp, err := GenerateKeyPair(nil)
	if err != nil {
		b.Fatalf("failed to GenerateKeyPair: %v", err)
	}

	sc := NewSigningContext([]byte("benchmark-signature"))
	st := makeBenchTranscript(sc)
	sig, err := kp.Sign(nil, st)
	if err != nil {
		b.Fatalf("failed to sign: %v", err)
	}

	b.ReportAllocs()
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		if !kp.PublicKey().Verify(st, sig) {
			b.Fatalf("Verify failed")
		}
	}
}
