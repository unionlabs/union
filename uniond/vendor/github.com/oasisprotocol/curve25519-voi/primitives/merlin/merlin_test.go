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

package merlin

import (
	"fmt"
	"io"
	"testing"

	"github.com/oasisprotocol/curve25519-voi/curve/scalar"
	"github.com/oasisprotocol/curve25519-voi/internal/zeroreader"
)

// testExtractBytes is a simple wrapper around ExtractBytes that allocates
// the destination buffer.
func (t *Transcript) testExtractBytes(label string, outLen int) []byte {
	dest := make([]byte, outLen)
	t.ExtractBytes(dest, label)
	return dest
}

// Initialize STROBE-128(4d65726c696e2076312e30)   # b"Merlin v1.0"
// meta-AD : 646f6d2d736570 || LE32(13)    # b"dom-sep"
// AD : 746573742070726f746f636f6c    # b"test protocol"
// meta-AD : 736f6d65206c6162656c || LE32(9)       # b"some label"
// AD : 736f6d652064617461    # b"some data"
// meta-AD : 6368616c6c656e6765 || LE32(32)        # b"challenge"
// PRF: d5a21972d0d5fe320c0d263fac7fffb8145aa640af6e9bca177c03c7efcf0615
// test transcript::tests::equivalence_simple ... ok

func TestSimpleTranscript(t *testing.T) {
	mt := NewTranscript("test protocol")
	mt.AppendMessage("some label", []byte("some data"))

	cBytes := mt.testExtractBytes("challenge", 32)
	cHex := fmt.Sprintf("%x", cBytes)
	expectedHex := "d5a21972d0d5fe320c0d263fac7fffb8145aa640af6e9bca177c03c7efcf0615"

	if cHex != expectedHex {
		t.Errorf("\nGot : %s\nWant: %s", cHex, expectedHex)
	}
}

func TestComplexTranscript(t *testing.T) {
	tr := NewTranscript("test protocol")
	tr.AppendMessage("step1", []byte("some data"))

	data := make([]byte, 1024)
	for i := range data {
		data[i] = 99
	}

	var chlBytes []byte
	for i := 0; i < 32; i++ {
		chlBytes = tr.testExtractBytes("challenge", 32)
		tr.AppendMessage("bigdata", data)
		tr.AppendMessage("challengedata", chlBytes)
	}

	expectedChlHex := "a8c933f54fae76e3f9bea93648c1308e7dfa2152dd51674ff3ca438351cf003c"
	chlHex := fmt.Sprintf("%x", chlBytes)

	if chlHex != expectedChlHex {
		t.Errorf("\nGot : %s\nWant: %s", chlHex, expectedChlHex)
	}
}

func TestClone(t *testing.T) {
	mt := NewTranscript("test protocol")
	mt.AppendMessage("some label", []byte("some data"))

	mtCopy, mtCopy2 := mt.Clone(), mt.Clone()

	// Ensure that mtCopy matches what we would get from mt.
	cBytes := mtCopy.testExtractBytes("challenge", 32)
	cHex := fmt.Sprintf("%x", cBytes)
	expectedHex := "d5a21972d0d5fe320c0d263fac7fffb8145aa640af6e9bca177c03c7efcf0615"
	if cHex != expectedHex {
		t.Errorf("\nmtCopy Got : %s\nWant: %s", cHex, expectedHex)
	}

	// Append more to mtCopy2, ensure that it is different.
	mtCopy2.AppendMessage("someother label", []byte("someother data"))
	cBytes = mtCopy2.testExtractBytes("challenge", 32)
	cHex = fmt.Sprintf("%x", cBytes)
	if cHex == expectedHex {
		t.Errorf("\nmtCopy2 Got : %s\nWant: %s", cHex, expectedHex)
	}

	// Finally, extract from mt.
	cBytes = mt.testExtractBytes("challenge", 32)
	cHex = fmt.Sprintf("%x", cBytes)
	if cHex != expectedHex {
		t.Errorf("\nmtCopy Got : %s\nWant: %s", cHex, expectedHex)
	}
}

func TestTranscriptRng(t *testing.T) {
	protocolLabel := "test TranscriptRng collisions"

	t1 := NewTranscript(protocolLabel)
	t2 := NewTranscript(protocolLabel)
	t3 := NewTranscript(protocolLabel)
	t4 := NewTranscript(protocolLabel)

	commitmentLabel := "com"
	commitment1 := []byte("commitment data 1")
	commitment2 := []byte("commitment data 2")

	t1.AppendMessage(commitmentLabel, commitment1)
	t2.AppendMessage(commitmentLabel, commitment2)
	t3.AppendMessage(commitmentLabel, commitment2)
	t4.AppendMessage(commitmentLabel, commitment2)

	witnessLabel := "witness"
	witness1 := []byte("witness data 1")
	witness2 := []byte("witness data 2")

	mustBuildRng := func(tr *Transcript, wb []byte, n string) io.Reader {
		var badRng zeroreader.ZeroReader
		r, err := tr.BuildRng().RekeyWithWitnessBytes(witnessLabel, wb).Finalize(badRng)
		if err != nil {
			t.Fatalf("\n%s Finalize failed: %v", n, err)
		}
		return r
	}

	r1 := mustBuildRng(t1, witness1, "t1")
	r2 := mustBuildRng(t2, witness1, "t2")
	r3 := mustBuildRng(t3, witness2, "t3")
	r4 := mustBuildRng(t4, witness2, "t4")

	mustRandomScalar := func(r io.Reader, n string) *scalar.Scalar {
		s, err := scalar.New().SetRandom(r)
		if err != nil {
			t.Fatalf("\nscalar.New().SetRandom(%s) failed: %v", n, err)
		}
		return s
	}

	s1 := mustRandomScalar(r1, "r1")
	s2 := mustRandomScalar(r2, "r2")
	s3 := mustRandomScalar(r3, "r3")
	s4 := mustRandomScalar(r4, "r4")

	// Transcript t1 has different commitments than t2, t3, t4, so
	// it should produce distinct challenges from all of them.
	if s1.Equal(s2) == 1 {
		t.Fatalf("s1 == s2")
	}
	if s1.Equal(s3) == 1 {
		t.Fatalf("s1 == s3")
	}
	if s1.Equal(s4) == 1 {
		t.Fatalf("s1 == s4")
	}

	// Transcript t2 has different witness variables from t3, t4,
	// so it should produce distinct challenges from all of them.
	if s2.Equal(s3) == 1 {
		t.Fatalf("s2 == s3")
	}
	if s2.Equal(s4) == 1 {
		t.Fatalf("s2 == s4")
	}

	// Transcripts t3 and t4 have the same commitments and
	// witnesses, so they should give different challenges only
	// based on the RNG. Checking that they're equal in the
	// presence of a bad RNG checks that the different challenges
	// above aren't because the RNG is accidentally different.
	if s3.Equal(s4) != 1 {
		t.Fatalf("s3 != s4")
	}
}
