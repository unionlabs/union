// Copyright (c) 2019 George Tankersley
// Copyright (c) 2019 Henry de Valence
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

// Package merlin implements Merlin proof transcripts.
package merlin

import (
	"crypto/rand"
	"encoding/binary"
	"fmt"
	"io"
	"math"

	"github.com/oasisprotocol/curve25519-voi/internal/strobe"
	_ "github.com/oasisprotocol/curve25519-voi/internal/toolchain"
)

const (
	merlinProtocolLabel  = "Merlin v1.0"
	domainSeparatorLabel = "dom-sep"
)

// Transcript is a Merlin proof transcript.
type Transcript struct {
	s strobe.Strobe
}

// NewTranscript initializes a new transcript with the specified protocol label.
func NewTranscript(appLabel string) *Transcript {
	t := Transcript{
		s: strobe.New(merlinProtocolLabel),
	}

	t.AppendMessage(domainSeparatorLabel, []byte(appLabel))
	return &t
}

// Clone returns a deep-copy of the transcript.
func (t *Transcript) Clone() *Transcript {
	return &Transcript{
		s: *t.s.Clone(),
	}
}

// Append adds the message to the transcript with the supplied label.
// If the length of label or message will overflow a 32-bit unsigned
// integer this method will panic.
func (t *Transcript) AppendMessage(label string, message []byte) {
	if lLen := uint64(len(label)); lLen > math.MaxUint32 {
		panic("merlin: label length exceeds limits")
	}
	if mLen := uint64(len(message)); mLen > math.MaxUint32 {
		panic("merlin: message length exceeds limits")
	}

	// AD[label || le32(len(message))](message)

	var sizeBuffer [4]byte
	binary.LittleEndian.PutUint32(sizeBuffer[0:], uint32(len(message)))

	t.s.MetaAD([]byte(label), false)
	t.s.MetaAD(sizeBuffer[:], true)

	t.s.AD(message, false)
}

// ExtractBytes fills dest with the verifier's challenge bytes. The label
// parameter is metadata about the challenge, and is also appended to
// the transcript. See the Transcript Protocols section of the Merlin website
// for details on labels.  If the length of label or dest will overflow
// a 32-bit unsigned integer this method will panic.
func (t *Transcript) ExtractBytes(dest []byte, label string) {
	if lLen := uint64(len(label)); lLen > math.MaxUint32 {
		panic("merlin: label length exceeds limits")
	}
	if dLen := uint64(len(dest)); dLen > math.MaxUint32 {
		panic("merlin: dest length exceeds limits")
	}

	var sizeBuffer [4]byte
	binary.LittleEndian.PutUint32(sizeBuffer[0:], uint32(len(dest)))

	t.s.MetaAD([]byte(label), false)
	t.s.MetaAD(sizeBuffer[:], true)

	t.s.PRF(dest)
}

// BuildRng constructs a transcript RNG builder bound to the current
// transcript state.
func (t *Transcript) BuildRng() *TranscriptRngBuilder {
	return &TranscriptRngBuilder{
		s: t.s.Clone(),
	}
}

// TranscriptRngBuilder constructs a transcript RNG by rekeying the transcript
// with prover secrets and an external RNG.
type TranscriptRngBuilder struct {
	s *strobe.Strobe
}

// RekeyWithitnessBytes rekeys the transcript using the provided witness data.
// If the length of label or witness will overflow a 32-bit unsigned
// integer this method will panic.
func (rb *TranscriptRngBuilder) RekeyWithWitnessBytes(label string, witness []byte) *TranscriptRngBuilder {
	if lLen := uint64(len(label)); lLen > math.MaxUint32 {
		panic("merlin: label length exceeds limits")
	}
	if wLen := uint64(len(witness)); wLen > math.MaxUint32 {
		panic("merlin: witness length exceeds limits")
	}

	// AD[label || le32(len(witness))](witness)

	var sizeBuffer [4]byte
	binary.LittleEndian.PutUint32(sizeBuffer[0:], uint32(len(witness)))

	rb.s.MetaAD([]byte(label), false)
	rb.s.MetaAD(sizeBuffer[:], true)

	rb.s.KEY(witness)

	return rb
}

// Finalize rekeys and finalizes the transcript, and constructs the RNG.
// If rng is nil, crypto/rand.Reader will be used.
//
// Note: This invalidates the TranscriptRngBuilder.
func (rb *TranscriptRngBuilder) Finalize(rng io.Reader) (io.Reader, error) {
	if rng == nil {
		rng = rand.Reader
	}

	randomBytes := make([]byte, 32)
	if _, err := io.ReadFull(rng, randomBytes); err != nil {
		return nil, fmt.Errorf("merlin: failed to read entropy: %w", err)
	}

	rb.s.MetaAD([]byte("rng"), false)

	rb.s.KEY(randomBytes)

	r := &transcriptRng{
		s: rb.s,
	}
	rb.s = nil // Crash on further calls to rb.

	return r, nil
}

type transcriptRng struct {
	s *strobe.Strobe
}

func (rng *transcriptRng) Read(p []byte) (int, error) {
	l := len(p)
	if uint64(l) > math.MaxUint32 {
		return 0, fmt.Errorf("merlin: read length exceeds limits")
	}

	var sizeBuffer [4]byte
	binary.LittleEndian.PutUint32(sizeBuffer[0:], uint32(l))

	rng.s.MetaAD(sizeBuffer[:], false)

	rng.s.PRF(p)

	return l, nil
}
