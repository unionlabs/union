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
	"crypto/rand"
	"crypto/sha512"
	"fmt"
)

// Example demonstrates common operations.
func Example() {
	// Basic operations attempt to be similar to the w3f/schnorrkel
	// Rust crate, while being as close to an idiomatic Go API as
	// possible.

	// Key generation

	miniSecretKey, err := GenerateMiniSecretKey(rand.Reader)
	if err != nil {
		panic("GenerateMiniSecretKey: " + err.Error())
	}

	secretKey := miniSecretKey.ExpandUniform() // ExpandEd25519 is also supported
	keypair := secretKey.KeyPair()             // The KeyPair type is what is used for signing.
	publicKey := keypair.PublicKey()

	// Serialization

	publicBytes, err := publicKey.MarshalBinary()
	if err != nil {
		panic("publicKey.MarshalBinary: " + err.Error())
	}

	var publicKey2 PublicKey
	if err = publicKey2.UnmarshalBinary(publicBytes); err != nil {
		panic("publicKey.UnmarshalBinary: " + err.Error())
	}
	if !publicKey.Equal(&publicKey2) {
		panic("public key did not round trip with BinaryMarshaller")
	}

	publicKey3, err := NewPublicKeyFromBytes(publicBytes)
	if err != nil {
		panic("NewPublicKeyFromBytes: " + err.Error())
	}
	if !publicKey.Equal(publicKey3) {
		panic("public key did not round trip with NewPublicKeyFromBytes")
	}

	// Signing
	signingContext := NewSigningContext([]byte("example signing context"))
	msg := []byte("test message")

	transcript := signingContext.NewTranscriptBytes(msg)
	signature, err := keypair.Sign(rand.Reader, transcript)
	if err != nil {
		panic("Sign: " + err.Error())
	}

	h := sha512.New512_256()
	_, _ = h.Write(msg)
	transcriptHashed := signingContext.NewTranscriptHash(h)
	signatureHashed, err := keypair.Sign(rand.Reader, transcriptHashed)
	if err != nil {
		panic("Sign(hashed): " + err.Error())
	}

	signatureBytes, err := signature.MarshalBinary()
	if err != nil {
		panic("signature.MarshalBinary: " + err.Error())
	}

	signature2, err := NewSignatureFromBytes(signatureBytes)
	if err != nil {
		panic("NewSignatureFromBytes: " + err.Error())
	}

	// Verification
	//
	// Note: Unlike the "other" Go sr25519 library, signing and verification
	// are side-effect free, and do not alter the transcript, so the transcripts
	// from the signing example are reused for brevity.

	if !publicKey.Verify(transcript, signature) {
		panic("Verify failed")
	}
	if !publicKey.Verify(transcript, signature2) {
		panic("Verify(signature2) failed, round-trip failure?")
	}

	if !publicKey.Verify(transcriptHashed, signatureHashed) {
		panic("Verify(hashed) failed")
	}

	// This would include a (separate) batch-verification example, but the
	// API is essentially identical to Ed25519, except based around transcripts.

	fmt.Println("ok")
	// Output: ok
}
