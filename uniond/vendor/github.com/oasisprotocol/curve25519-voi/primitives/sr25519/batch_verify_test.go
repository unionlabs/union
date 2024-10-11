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
	"testing"
)

var benchBatchSizes = []int{1, 2, 4, 8, 16, 32, 64, 128, 256, 384, 512, 768, 1024}

type batchTest int

const (
	batchNoErrors batchTest = iota
	batchBadTranscript
	batchBadSignature
	batchBadPublicKey

	testBatchSize = 38
)

type batchVerifierTestCase struct {
	n          string
	tst        batchTest
	culpritIdx int
	details    string
}

var batchTestCases = []*batchVerifierTestCase{
	{
		"Verify",
		batchNoErrors,
		-1,
		"failed batch verification",
	},
	{
		"FailsOnBadTranscript",
		batchBadTranscript,
		0,
		"batch verification should fail due to bad transcript",
	},
	{
		"FailsOnBadSignature",
		batchBadSignature,
		1,
		"batch verification should fail due to bad signature",
	},
	{
		"FailsOnBadPublicKey",
		batchBadPublicKey,
		2,
		"batch verification should fail due to bad public key",
	},
}

func (tc *batchVerifierTestCase) makeVerifier(t *testing.T) *BatchVerifier {
	const n = 38

	v := NewBatchVerifier()
	kps := make([]*KeyPair, 0, n)
	pubs := make([]*PublicKey, 0, n)
	msgs := make([][]byte, 0, n)

	goodContext := NewSigningContext([]byte("test-batch-verify:good"))
	badContext := NewSigningContext([]byte("test-batch-verify:bad"))

	for i := 0; i < testBatchSize; i++ {
		kp, err := GenerateKeyPair(nil)
		if err != nil {
			t.Fatalf("failed to GenerateKeyPair: %v", err)
		}

		var msg []byte
		if i%2 == 0 {
			msg = []byte("easter")
		} else {
			msg = []byte("egg")
		}

		msgs = append(msgs, msg)
		kps = append(kps, kp)
		pubs = append(pubs, kp.PublicKey())
	}

	// If the test case calls for it, introduce errors in the batch.
	culpritIdx := tc.culpritIdx
	switch tc.tst {
	case batchBadPublicKey:
		for {
			kp, err := GenerateKeyPair(nil)
			if err != nil {
				t.Fatalf("failed to GenerateKeyPair: %v", err)
			}
			pk := kp.PublicKey()
			if !pk.Equal(pubs[0]) {
				pubs[culpritIdx] = pk
				break
			}
		}
	}

	sigs := make([]*Signature, 0, n)
	transcripts := make([]*SigningTranscript, 0, n)

	for i := range kps {
		st := goodContext.NewTranscriptBytes(msgs[i])
		sig, err := kps[i].Sign(nil, st)
		if err != nil {
			t.Fatalf("failed to Sign: %v", err)
		}

		transcripts = append(transcripts, st)
		sigs = append(sigs, sig)
	}

	switch tc.tst {
	case batchBadTranscript:
		transcripts[culpritIdx] = badContext.NewTranscriptBytes(msgs[culpritIdx])
	case batchBadSignature:
		sigs[culpritIdx] = sigs[0]
	}

	// Build the batch.
	for i := range pubs {
		v.Add(pubs[i], transcripts[i], sigs[i])
	}

	return v
}

func (tc *batchVerifierTestCase) run(t *testing.T) {
	v := tc.makeVerifier(t)
	expectedBatchOk := tc.culpritIdx < 0
	expectedVerifyOk := expectedBatchOk

	// First test that the batch verify returns the expected
	// result for the entire batch.
	if v.VerifyBatchOnly(nil) != expectedBatchOk {
		t.Error(tc.details)
	}

	// Then test the actually useful API.
	allValid, valid := v.Verify(nil)
	if allValid != expectedVerifyOk {
		t.Errorf("Verify returned incorrect summary (Got: %v)", allValid)
	}

	// The ensure that the bit-vector contains the expected
	// signature validity status.  tc.culpritIdx is the index
	// of the malformed/invalid signature.
	for i, sigValid := range valid {
		expectedSigOk := i != tc.culpritIdx
		if sigValid != expectedSigOk {
			t.Errorf("bit-vector %d incorrect (Got: %v)", i, sigValid)
		}
	}
}

func TestBatchVerifier(t *testing.T) {
	t.Run("sr25519", func(t *testing.T) {
		for _, tc := range batchTestCases {
			t.Run(tc.n, func(t *testing.T) {
				tc.run(t)
			})
		}
	})

	t.Run("EmptyBatchFails", func(t *testing.T) {
		v := NewBatchVerifier()

		if v.VerifyBatchOnly(nil) {
			t.Error("batch verification should fail on an empty batch")
		}
	})
	t.Run("Reset", func(t *testing.T) {
		v := NewBatchVerifier()

		// Reseting an empty batch verifier should work.
		v.Reset()

		ctx := NewSigningContext([]byte("test-batch-verify:reset"))
		kp, err := GenerateKeyPair(nil)
		if err != nil {
			t.Fatalf("failed to GenerateKeyPair: %v", err)
		}
		pub := kp.PublicKey()
		st := ctx.NewTranscriptBytes([]byte("ResetTest"))
		sig, err := kp.Sign(nil, st)
		if err != nil {
			t.Fatalf("failed to Sign: %v", err)
		}

		for i := 0; i < 10; i++ {
			v.Add(pub, st, sig)
		}
		v.Add(pub, st, &Signature{})
		if v.anyInvalid == false {
			t.Fatalf("Uninitialized signature did not invalidate batch")
		}

		v.Reset()
		if len(v.entries) != 0 {
			t.Fatalf("Reset did not shrink entries")
		}
		if cap(v.entries) == 0 {
			// Can't check for an exact capacity since this is at the
			// mercy of how stdlib reallocs.
			t.Fatalf("Reset did not preserve entries backing store")
		}
		if v.anyInvalid != false {
			t.Fatalf("Reset did not clear anyInvalid")
		}
	})
	t.Run("NewWithCapacity", func(t *testing.T) {
		v := NewBatchVerifierWithCapacity(10)

		if l := len(v.entries); l != 0 {
			t.Fatalf("unexpected v.entries length: %d", l)
		}
		if c := cap(v.entries); c != 10 {
			t.Fatalf("unexpected v.entries capacity: %d", c)
		}
	})
}

func BenchmarkVerifyBatchOnly(b *testing.B) {
	for _, n := range benchBatchSizes {
		doBenchVerifyBatchOnly(b, n)
	}
}

func doBenchVerifyBatchOnly(b *testing.B, n int) {
	kp, err := GenerateKeyPair(nil)
	if err != nil {
		b.Fatalf("failed to GenerateKeyPair: %v", err)
	}
	pk := kp.PublicKey()

	msg := []byte("BatchVerifyTest")

	sc := NewSigningContext([]byte("benchmark-batch-verify"))
	st := sc.NewTranscriptBytes(msg)
	sig, err := kp.Sign(nil, st)
	if err != nil {
		b.Fatalf("failed to sign: %v", err)
	}

	b.Run(fmt.Sprint(n), func(b *testing.B) {
		b.ReportAllocs()

		for i := 0; i < b.N; i++ {
			v := NewBatchVerifier()
			for j := 0; j < n; j++ {
				v.Add(pk, st, sig)
			}

			if !v.VerifyBatchOnly(nil) {
				b.Fatal("signature set failed batch verification")
			}
		}
	})
}
