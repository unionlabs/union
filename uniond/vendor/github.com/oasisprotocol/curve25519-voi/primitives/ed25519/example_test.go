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

package ed25519

import (
	"crypto"
	"crypto/rand"
	"crypto/sha512"
	"fmt"
)

// Example demonstrates common operations.
func Example() {
	// Basic operations are API compatible with crypto/ed25519, as in
	// curve25519-voi implements a superset of the crypto/ed25519 API.

	msg := []byte("test message")

	// Key generation
	publicKey, privateKey, err := GenerateKey(rand.Reader)
	if err != nil {
		panic("GenerateKey: " + err.Error())
	}

	// Signing
	sig := Sign(privateKey, msg)

	// Verification
	if !Verify(publicKey, msg, sig) {
		panic("Verify: failed")
	}

	// Verification semantics are incompatible with crypto/ed25519
	// by default.  It is possible to generate signatures that will be
	// accepted by one (and rejected by the other), though this will
	// NEVER happen during normal operation.
	//
	// If exact compatibility with the standard library is required
	// then VerifyWithOptions must be used with the appropriate options.
	//
	// Note: Over 99% of the users do not need this, the various edge
	// case incompatiblities are systemic across the whole Ed25519
	// ecosystem as a whole, and RFC 8032 explicitly allows for this
	// situation.
	opts := &Options{
		Verify: VerifyOptionsStdLib,
	}
	if !VerifyWithOptions(publicKey, msg, sig, opts) {
		panic("VerifyWithOptions: failed")
	}

	// curve25519-voi supports Ed25519ph and Ed25519ctx as defined in
	// RFC 8032 in addition to Ed25519pure.

	// Ed25519ctx
	//
	// Note: Due to Context being defined as a string, a 0-length
	// context is not supported, even though it is allowed but
	// discouraged by the RFC.
	opts = &Options{
		Context: string("test context"),
	}
	sig, err = privateKey.Sign(nil, msg, opts)
	if err != nil {
		panic("SignWithOptions(ctx): " + err.Error())
	}

	if !VerifyWithOptions(publicKey, msg, sig, opts) {
		panic("VerifyWithOptions(ctx): failed")
	}

	// Ed25519ph
	prehash := sha512.Sum512(msg)
	opts = &Options{
		Hash: crypto.SHA512,
	}
	if sig, err = privateKey.Sign(nil, prehash[:], opts); err != nil {
		panic("SignWithOptions(ph): " + err.Error())
	}
	if !VerifyWithOptions(publicKey, prehash[:], sig, opts) {
		panic("VerifyWithOptions(ph): failed")
	}

	fmt.Println("ok")
	// Output: ok
}
