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

// Package scalar128 implements the fast generation of random 128-bit
// scalars for the purpose of batch verification.
package scalar128

import (
	"bytes"
	"fmt"
	"io"

	"golang.org/x/crypto/chacha20"

	"github.com/oasisprotocol/curve25519-voi/curve/scalar"
)

const randomizerSize = scalar.ScalarSize / 2

var zeroRandomizer [scalar.ScalarSize]byte

// The ideas behind this implementation is based on "Faster batch fogery
// identification" (https://eprint.iacr.org/2012/549.pdf), which is basically
// one of the few places where how to select an appropriate z_i is discussed.
//
// It turns out that even in the no-assembly (`purego`) case, calling chacha20
// is more performant than reading from the system entropy source, at least
// on my 64-bit Intel Linux systems.

// FixRawRangeVartime adjusts the raw scalar to be in the correct range,
// in variable-time.
//
// From the paper:
//
//   Of course, it is also safe to simply generate z_i as a uniform
//   random b-bit integer, disregarding the negligible chance that
//   z_i == 0; but this requires minor technical modifications to
//   the security guarantees stated below, so we prefer to require
//   z != 0.
//
func FixRawRangeVartime(rawScalar *[scalar.ScalarSize]byte) {
	// From the paper:
	//
	//   As precomputation we choose z_1, z_2, . . . , z_n independently
	//   and uniformly at random from the set {1, 2, 3, . . . , 2^b},
	//   where b is the security level. There are several reasonable ways
	//   to do this: for example, generate a uniform random b-bit integer
	//   and add 1, or generate a uniform random b-bit integer and
	//   replace 0 with 2^b.
	//
	// We go with the latter approach and replace 0 with 2^b, since
	// it is significantly faster for the common case.

	if bytes.Equal(rawScalar[:], zeroRandomizer[:]) {
		rawScalar[randomizerSize] = 1
	}
}

// Generator is a random 128-bit scalar generator.
type Generator struct {
	cipher *chacha20.Cipher

	// Go's escape analysis insists on sticking this on the heap, if
	// it is declared in the function when building with `purego`,
	// but not if assembly is being used for chacha20.
	tmp [scalar.ScalarSize]byte
}

// SetScalarVartime sets the scalar to a random 128-bit scalar, in
// variable-time.
func (gen *Generator) SetScalarVartime(s *scalar.Scalar) error {
	// perf: This probably isn't needed, XOR-ing the previous output
	// with more keystream is still random.
	for i := range gen.tmp[:randomizerSize+1] {
		gen.tmp[i] = 0
	}

	gen.cipher.XORKeyStream(gen.tmp[:randomizerSize], gen.tmp[:randomizerSize])
	FixRawRangeVartime(&gen.tmp)

	// Since the random scalar is 128-bits, there is no need to reduce.
	if _, err := s.SetBits(gen.tmp[:]); err != nil {
		return fmt.Errorf("internal/scalar128: failed to deserialize random scalar: %w", err)
	}

	return nil
}

// NewGenerator constructs a new random scalar generator, using entropy
// from the provided source.
func NewGenerator(rand io.Reader) (*Generator, error) {
	var (
		key   [chacha20.KeySize]byte
		nonce [chacha20.NonceSize]byte
	)

	if _, err := io.ReadFull(rand, key[:]); err != nil {
		return nil, fmt.Errorf("internal/scalar128: failed to read random key: %w", err)
	}

	cipher, err := chacha20.NewUnauthenticatedCipher(key[:], nonce[:])
	if err != nil {
		return nil, fmt.Errorf("internal/scalar128: failed initialize stream cipher: %w", err)
	}

	return &Generator{
		cipher: cipher,
	}, nil
}
