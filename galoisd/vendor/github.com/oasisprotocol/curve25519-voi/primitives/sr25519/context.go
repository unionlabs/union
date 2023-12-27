// Copyright (c) 2019 Web 3 Foundation. All rights reserved.
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
	"fmt"
	"hash"
	"io"

	"github.com/oasisprotocol/curve25519-voi/curve"
	"github.com/oasisprotocol/curve25519-voi/curve/scalar"
	"github.com/oasisprotocol/curve25519-voi/primitives/merlin"
)

// SigningContext is a Schnoor signing context.
type SigningContext struct {
	t *merlin.Transcript
}

// NewSigningContext initializes a new signing context from a static byte
// string that identifies the signer's role in the larger protocol.
func NewSigningContext(context []byte) *SigningContext {
	t := merlin.NewTranscript("SigningContext")
	t.AppendMessage("", context)
	return &SigningContext{
		t: t,
	}
}

// NewTranscriptBytes initializes a new signing transcript on a message
// provided as a byte array.  If the length of b will overflow a 32-bit
// unsigned integer, this method will panic.
//
// Note: This method should not be used for large messages as it calls
// merlin directly, and merlin is designed for domain separation, not
// performance.
func (sc *SigningContext) NewTranscriptBytes(b []byte) *SigningTranscript {
	t := sc.t.Clone()
	t.AppendMessage("sign-bytes", b)
	return &SigningTranscript{
		t: t,
	}
}

// NewTranscriptHash initializes a new signing transcript on a message
// provided as a hash.Hash, with a digest size of either 256-bits or
// 512-bits.  If the digest size is neither 256-bits nor 512-bits, this
// method will panic.
func (sc *SigningContext) NewTranscriptHash(h hash.Hash) *SigningTranscript {
	var hLabel string
	switch h.Size() {
	case 32:
		hLabel = "sign-256"
	case 64:
		hLabel = "sign-512"
	default:
		panic("sr25519: invalid hash digest size")
	}
	prehash := h.Sum(nil)

	t := sc.t.Clone()
	t.AppendMessage(hLabel, prehash)
	return &SigningTranscript{
		t: t,
	}
}

// NewTranscriptXOF initializes a new signing transcript on a message
// provided as a io.Reader that is an XOF instance.
func (sc *SigningContext) NewTranscriptXOF(xof io.Reader) *SigningTranscript {
	prehash := make([]byte, 32)
	if _, err := io.ReadFull(xof, prehash); err != nil {
		panic("sr25519: failed to read XOF output: " + err.Error())
	}

	t := sc.t.Clone()
	t.AppendMessage("sign-XoF", prehash)
	return &SigningTranscript{
		t: t,
	}
}

// SigningTranscript is a Schnoor signing transcript.
type SigningTranscript struct {
	t *merlin.Transcript
}

func (st *SigningTranscript) clone() *SigningTranscript {
	return &SigningTranscript{
		t: st.t.Clone(),
	}
}

func (st *SigningTranscript) commitBytes(label string, b []byte) {
	st.t.AppendMessage(label, b)
}

func (st *SigningTranscript) protoName(name string) {
	st.commitBytes("proto-name", []byte(name))
}

func (st *SigningTranscript) commitPoint(label string, compressed *curve.CompressedRistretto) {
	st.commitBytes(label, compressed[:])
}

func (st *SigningTranscript) challengeBytes(dest []byte, label string) {
	st.t.ExtractBytes(dest, label)
}

func (st *SigningTranscript) challengeScalar(label string) *scalar.Scalar {
	var scalarBytes [scalar.ScalarWideSize]byte
	st.challengeBytes(scalarBytes[:], label)
	s, err := scalar.NewFromBytesModOrderWide(scalarBytes[:])
	if err != nil {
		panic("sr25519: scalar.NewFromBytesModOrderWide: " + err.Error())
	}
	return s
}

func (st *SigningTranscript) witnessScalar(label string, nonceSeeds [][]byte, rng io.Reader) (*scalar.Scalar, error) {
	rng, err := st.witnessRng(label, nonceSeeds, rng)
	if err != nil {
		return nil, fmt.Errorf("sr25519: failed to construct transcript rng: %w", err)
	}
	return scalar.New().SetRandom(rng)
}

func (st *SigningTranscript) witnessBytes(dest []byte, label string, nonceSeeds [][]byte, rng io.Reader) error {
	rng, err := st.witnessRng(label, nonceSeeds, rng)
	if err != nil {
		return fmt.Errorf("sr25519: failed to construct transcript rng: %w", err)
	}

	if _, err := rng.Read(dest); err != nil {
		return fmt.Errorf("sr25519: failed to read from transcript rng: %w", err)
	}

	return nil
}

func (st *SigningTranscript) witnessRng(label string, nonceSeeds [][]byte, rng io.Reader) (io.Reader, error) {
	br := st.t.BuildRng()
	for _, ns := range nonceSeeds {
		br.RekeyWithWitnessBytes(label, ns)
	}
	return br.Finalize(rng)
}
