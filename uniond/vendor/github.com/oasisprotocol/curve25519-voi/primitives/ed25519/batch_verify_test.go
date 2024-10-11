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
	"fmt"
	"testing"
)

var benchBatchSizes = []int{1, 2, 4, 8, 16, 32, 64, 128, 256, 384, 512, 768, 1024}

type batchTest int

const (
	batchNoErrors batchTest = iota
	batchShortSig
	batchCorruptKey
	batchCorruptSignatureR
	batchCorruptSignatureS
	batchCorruptMessage
	batchMalformedKey
	batchMalformedSig
	batchMalformedPh
	batchMalformedCtx

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
		"FailsOnShortSig",
		batchShortSig,
		0,
		"batch verification should fail due to short signature",
	},
	{
		"FailsOnCorruptKey",
		batchCorruptKey,
		1,
		"batch verification should fail due to corrupt key",
	},
	{
		"FailsOnCorruptSignature/R",
		batchCorruptSignatureR,
		2,
		"batch verification should fail due to corrupt signature (R)",
	},
	{
		"FailsOnCorruptSignature/S",
		batchCorruptSignatureS,
		3,
		"batch verification should fail due to corrupt signature (S)",
	},
	{
		"FailsOnCorruptMessage",
		batchCorruptMessage,
		4,
		"batch verification should fail due to corrupt message",
	},
	{
		"FailsOnMalformedKey",
		batchMalformedKey,
		5,
		"batch verification should fail due to malformed key",
	},
	{
		"FailsOnMalformedSignature",
		batchMalformedSig,
		6,
		"batch verification should fail due to malformed signature",
	},
}

func (tc *batchVerifierTestCase) makeVerifier(t *testing.T, opts *Options, expanded bool) *BatchVerifier {
	const n = 38

	v := NewBatchVerifier()
	privs := make([]PrivateKey, 0, n)
	pubs := make([]PublicKey, 0, n)
	msgs := make([][]byte, 0, n)
	sigs := make([][]byte, 0, n)
	optsVec := make([]*Options, 0, n)

	for i := 0; i <= testBatchSize; i++ {
		pub, priv, err := GenerateKey(nil)
		if err != nil {
			t.Fatalf("failed to GenerateKey: %v", err)
		}

		var msg []byte
		if i%2 == 0 {
			msg = []byte("easter")
		} else {
			msg = []byte("egg")
		}
		if opts != nil && opts.Hash != 0 {
			h := opts.HashFunc().New()
			_, _ = h.Write(msg)
			msg = h.Sum(nil)
		}

		privs = append(privs, priv)
		pubs = append(pubs, pub)
		msgs = append(msgs, msg)
		optsVec = append(optsVec, opts)
	}

	// If the test case calls for it, introduce errors in the batch.
	culpritIdx := tc.culpritIdx
	switch tc.tst {
	case batchCorruptKey:
		pubs[culpritIdx][1] ^= 1
	case batchMalformedKey:
		pubs[culpritIdx] = pubs[culpritIdx][:31]
	}

	for i := range privs {
		switch {
		case opts == nil:
			sigs = append(sigs, Sign(privs[i], msgs[i]))
		default:
			sig, err := privs[i].Sign(nil, msgs[i], opts)
			if err != nil {
				t.Fatalf("failed to sign message: %v", err)
			}
			sigs = append(sigs, sig)
		}
	}

	switch tc.tst {
	case batchShortSig:
		sigs[culpritIdx] = nil
	case batchCorruptSignatureR:
		sigs[culpritIdx][1] ^= 1
	case batchCorruptSignatureS:
		sigs[culpritIdx][33] ^= 1
	case batchCorruptMessage:
		msgs[culpritIdx] = []byte("not the message")
	case batchMalformedSig:
		sigs[culpritIdx] = append(sigs[culpritIdx], 23)
	case batchMalformedPh:
		msgs[culpritIdx] = []byte("not a pre-hash")
	case batchMalformedCtx:
		b := make([]byte, ContextMaxSize+1)
		for i := range b {
			b[i] = byte('a')
		}
		optsVec[culpritIdx] = &Options{
			Context: string(b),
		}
	}

	for i := range pubs {
		switch expanded {
		case true:
			expPub, err := NewExpandedPublicKey(pubs[i])
			if expanded && i == culpritIdx {
				switch tc.tst {
				case batchMalformedKey:
					if err == nil {
						t.Fatalf("succeded in expanding malformed public key")
					}
					t.Skipf("malformed public key expansion failed as expected")
				case batchCorruptKey:
					// This is non-deterministic since it is possible for
					// the corrupted form to still be a valid ed25519 key.
					//
					// If the key happens to expand, go through with
					// the actual tests since the batch should fail.
					if err != nil {
						t.Skipf("corrupted public key expansion failed")
					}
				}
			}
			if err != nil {
				t.Fatalf("failed to expand public key: %v", err)
			}
			switch {
			case optsVec[i] == nil:
				v.AddExpanded(expPub, msgs[i], sigs[i])
			default:
				v.AddExpandedWithOptions(expPub, msgs[i], sigs[i], optsVec[i])
			}
		case false:
			switch {
			case optsVec[i] == nil:
				v.Add(pubs[i], msgs[i], sigs[i])
			default:
				v.AddWithOptions(pubs[i], msgs[i], sigs[i], optsVec[i])
			}
		}
	}

	return v
}

func (tc *batchVerifierTestCase) run(t *testing.T, opts *Options, expanded bool) {
	v := tc.makeVerifier(t, opts, expanded)
	checkBatchResult(t, v, opts, tc.details, tc.culpritIdx)
}

func checkBatchResult(t *testing.T, v *BatchVerifier, opts *Options, details string, badIndex int) {
	expectedBatchOk := badIndex < 0
	expectedVerifyOk := expectedBatchOk
	if opts != nil && opts.Verify != nil && opts.Verify.CofactorlessVerify {
		// Cofactor-less verification should always fail batch verify.
		expectedBatchOk = false
	}

	// First test that the batch verify returns the expected
	// result for the entire batch.
	if v.VerifyBatchOnly(nil) != expectedBatchOk {
		t.Fatal(details)
	}

	// Then test the actually useful API.
	allValid, valid := v.Verify(nil)
	if allValid != expectedVerifyOk {
		t.Fatalf("Verify returned incorrect summary (Got: %v)", allValid)
	}

	// The ensure that the bit-vector contains the expected
	// signature validity status.  tc.culpritIdx is the index
	// of the malformed/invalid signature.
	for i, sigValid := range valid {
		expectedSigOk := i != badIndex
		if sigValid != expectedSigOk {
			t.Fatalf("bit-vector %d incorrect (Got: %v)", i, sigValid)
		}
	}
}

func TestBatchVerifier(t *testing.T) {
	runTestCases := func(t *testing.T, opts *Options, expanded bool) {
		for _, tc := range batchTestCases {
			t.Run(tc.n, func(t *testing.T) {
				tc.run(t, opts, expanded)
			})
		}
	}
	runCtxTestCases := func(t *testing.T, expanded bool) {
		opts := &Options{
			Context: "Ed25519ctx test context",
		}
		runTestCases(t, opts, expanded)

		tc := &batchVerifierTestCase{
			"FailsOnMalformedContext",
			batchMalformedCtx,
			7,
			"batch verification should fail due to malformed context",
		}
		t.Run(tc.n, func(t *testing.T) {
			tc.run(t, opts, expanded)
		})
	}
	runPhTestCases := func(t *testing.T, expanded bool) {
		opts := &Options{
			Hash: crypto.SHA512,
		}
		runTestCases(t, opts, expanded)

		tc := &batchVerifierTestCase{
			"FailsOnMalformedPreHash",
			batchMalformedPh,
			7,
			"batch verification should fail due to malformed pre-hash",
		}
		t.Run(tc.n, func(t *testing.T) {
			tc.run(t, opts, expanded)
		})
	}

	t.Run("Ed25519pure", func(t *testing.T) {
		runTestCases(t, nil, false)
	})
	t.Run("Ed25519pure/Expanded", func(t *testing.T) {
		runTestCases(t, nil, true)
	})
	t.Run("Ed25519ctx", func(t *testing.T) {
		runCtxTestCases(t, false)
	})
	t.Run("Ed25519ctx/Expanded", func(t *testing.T) {
		runCtxTestCases(t, true)
	})
	t.Run("Ed25519ph", func(t *testing.T) {
		runPhTestCases(t, false)
	})
	t.Run("Ed25519ph/Expanded", func(t *testing.T) {
		runPhTestCases(t, true)
	})
	t.Run("CofactorlessFallback", func(t *testing.T) {
		runTestCases(t, &Options{
			Verify: VerifyOptionsStdLib,
		}, false)
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

		pub, priv, err := GenerateKey(nil)
		if err != nil {
			t.Fatalf("failed to GenerateKey: %v", err)
		}
		msg := []byte("ResetTest")
		sig := Sign(priv, msg)

		for i := 0; i < 10; i++ {
			v.Add(pub, msg, sig)
		}
		v.Add(pub, msg, nil)
		v.AddWithOptions(pub, msg, nil, &Options{
			Verify: VerifyOptionsStdLib,
		})

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
		if v.anyCofactorless != false {
			t.Fatalf("Reset did not clear anyCofactorless")
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
	t.Run("HugeBatch", func(t *testing.T) {
		// Initialize a batch verifier for a gigantic batch.
		const hugeBatchSize = 10000
		v := NewBatchVerifierWithCapacity(hugeBatchSize)

		pub, priv, err := GenerateKey(nil)
		if err != nil {
			t.Fatalf("failed to GenerateKey: %v", err)
		}
		msg := []byte("HugeBatchTest")
		sig := Sign(priv, msg)

		// Ensure the batch verifier gives up expanding keys after
		// the batch grows to be huge.
		for i := 0; i < hugeBatchSize; i++ {
			v.Add(pub, msg, sig)
		}
		for i, ent := range v.entries {
			if (ent.expandedA != nil) != (i < batchPippengerThreshold) {
				t.Fatalf("unexpected expanded A format: index %d, expandedA %v", i, ent.expandedA)
			}
		}
		if !v.anyNotExpanded {
			t.Fatalf("batch verifier did not set the anyNotExpanded flag")
		}

		// Ensure the batch verifier verifies a valid giant batch.
		checkBatchResult(t, v, optionsDefault, "failed large batch verification", -1)

		// Ensure the batch verifier fails to verify a invalid giant batch.
		v.Add(pub, []byte("HugeBatchTest: bad message"), sig)
		checkBatchResult(t, v, optionsDefault, "invalid message succeded", hugeBatchSize)

		// Reset the batch.
		v.Reset()

		// Ensure that Reset clears the key expansion disable.
		if v.anyNotExpanded {
			t.Fatalf("reset did not clear the anyNotExpanded flag")
		}
	})
}

func BenchmarkVerifyBatchOnly(b *testing.B) {
	for _, n := range benchBatchSizes {
		doBenchVerifyBatchOnly(b, n, false)
	}
}

func doBenchVerifyBatchOnly(b *testing.B, n int, expanded bool) {
	// Note: Comparative benchmarks are kind of hard to do, especially
	// against ed25519consensus, which excludes building BatchVerifier
	// from the benchmarks and memory allocation accounting.
	//
	// Since we care about the total, we include it.

	pub, priv, _ := GenerateKey(nil)
	msg := []byte("BatchVerifyTest")
	sig := Sign(priv, msg)

	expPub, err := NewExpandedPublicKey(pub)
	if err != nil {
		b.Fatalf("failed to expand public key: %v", err)
	}

	b.Run(fmt.Sprint(n), func(b *testing.B) {
		b.ReportAllocs()

		for i := 0; i < b.N; i++ {
			v := NewBatchVerifier()
			for j := 0; j < n; j++ {
				switch expanded {
				case true:
					v.AddExpandedWithOptions(expPub, msg, sig, optionsDefault)
				default:
					v.Add(pub, msg, sig)
				}
			}

			if !v.VerifyBatchOnly(nil) {
				b.Fatal("signature set failed batch verification")
			}
		}
	})
}
