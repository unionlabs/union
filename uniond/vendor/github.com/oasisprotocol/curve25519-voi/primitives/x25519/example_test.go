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

package x25519

import (
	"bytes"
	"crypto/rand"
	"fmt"
)

// Example demonstrates common operations.
func Example() {
	// Basic operations are API compatible with x/crypto/curve25519.

	// Key generation
	alicePrivate := make([]byte, ScalarSize)
	if _, err := rand.Read(alicePrivate); err != nil {
		panic("rand.Read: " + err.Error())
	}

	alicePublic, err := X25519(alicePrivate, Basepoint)
	if err != nil {
		panic("x25519.X25519(Basepoint): " + err.Error())
	}

	var bobPrivate, bobPublic [32]byte
	if _, err := rand.Read(bobPrivate[:]); err != nil {
		panic("rand.Read: " + err.Error())
	}
	ScalarBaseMult(&bobPublic, &bobPrivate)

	// Shared secret
	//
	// Note: If the "all zero output" check for contributory behavior
	// is not wanted, then the "deprecated" ScalarMult call should be
	// used.  Marking a routine that still has useful behavior as
	// deprecated isn't great, but that is what x/crypto/curve25519 does.
	aliceShared, err := X25519(alicePrivate, bobPublic[:])
	if err != nil {
		panic("x25519.X25519: " + err.Error())
	}

	var bobShared, tmp [32]byte
	copy(tmp[:], alicePublic)
	ScalarMult(&bobShared, &bobPrivate, &tmp) //nolint: staticcheck

	if !bytes.Equal(aliceShared, bobShared[:]) {
		panic("shared secret mismatch")
	}

	fmt.Println("ok")
	// Output: ok
}
