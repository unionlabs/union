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
	"crypto/rand"
	"fmt"
)

// ExampleBatchVerifier demonstrates batch verification functionality.
func ExampleBatchVerifier() {
	// curve25519-voi provides batch verification, with an API that
	// is similar to ed25519consensus.

	msg1 := []byte("test message")
	msg2 := []byte("test message 2")
	msg3 := []byte("test message with a vengance")
	msg4 := []byte("live free or test message")
	msg5 := []byte("a good day to test message")

	publicKey, privateKey, err := GenerateKey(rand.Reader)
	if err != nil {
		panic("GenerateKey: " + err.Error())
	}

	publicKey2, privateKey2, err := GenerateKey(rand.Reader)
	if err != nil {
		panic("GenerateKey: " + err.Error())
	}

	sig1 := Sign(privateKey, msg1)
	sig2 := Sign(privateKey2, msg2)
	sig3 := Sign(privateKey, msg3)
	opts := &Options{
		Context: "yippie ki-yay",
		Verify:  VerifyOptionsFIPS_186_5,
	}
	sig4, err := privateKey.Sign(nil, msg4, opts)
	if err != nil {
		panic("SignWithOptions: " + err.Error())
	}
	sig5 := Sign(privateKey, msg5)

	// Batch verification
	verifier := NewBatchVerifier()
	verifier.Add(publicKey, msg1, sig1)
	verifier.Add(publicKey2, msg2, sig2) // Public keys can be different.
	verifier.Add(publicKey, msg3, sig3)

	// It is possible to specify verification options on a
	// per-batch-entry basis, including Ed25519ph/Ed25519ctx.
	verifier.AddWithOptions(publicKey, msg4, sig4, opts)

	// Only verify the batch, receiving no information about which
	// batch entries are incorrect in the event of a failure.
	//
	// Note: Verify is probably more useful.
	if !verifier.VerifyBatchOnly(rand.Reader) {
		panic("verifier.VerifyBatchOnly failed")
	}

	// Verify the batch, and return a vector showing the valid entries.
	allOk, okVec := verifier.Verify(rand.Reader)
	if !allOk {
		panic("verifier.Verify failed")
	}
	for i, v := range okVec {
		if !v {
			panic(fmt.Sprintf("verifier.Verify: entry %d invalid", i))
		}
	}

	// Batch failure
	sig5[16] ^= 0xa5
	verifier.Add(publicKey, msg5, sig5)
	allOk, okVec = verifier.Verify(rand.Reader)
	if allOk {
		panic("verifier.Verify succeeded when batch should fail")
	}
	for i, v := range okVec[:4] {
		if !v {
			panic(fmt.Sprintf("verifier.Verify: entry %d invalid", i))
		}
	}
	if okVec[4] {
		panic("verifier.Verify: entry 4 valid")
	}

	fmt.Println("ok")
	// Output: ok
}
