// Copyright (c) 2019 Web 3 Foundation. All rights reserved.
// Copyright (c) 2020 Henry de Valence. All rights reserved.
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
	cryptorand "crypto/rand"
	"io"

	"github.com/oasisprotocol/curve25519-voi/curve"
	"github.com/oasisprotocol/curve25519-voi/curve/scalar"
	"github.com/oasisprotocol/curve25519-voi/internal/scalar128"
	"github.com/oasisprotocol/curve25519-voi/internal/zeroreader"
	"github.com/oasisprotocol/curve25519-voi/primitives/merlin"
)

type BatchVerifier struct {
	entries []entry

	anyInvalid bool
}

type entry struct {
	// Store r, s, A, and hram in the entry, so that it is possible
	// to implement a more useful verfication API (information about
	// which signature(s) are invalid is wanted a lot of the time),
	// without having to redo a non-trivial amount of computation.
	R    curve.RistrettoPoint
	S    scalar.Scalar
	A    curve.RistrettoPoint
	hram scalar.Scalar

	witnessA     curve.CompressedRistretto
	witnessR     curve.CompressedRistretto
	witnessBytes [16]byte

	canBeValid bool
}

func (e *entry) doInit(pk *PublicKey, transcript *SigningTranscript, signature *Signature) {
	// Until everything has been deserialized correctly, assume the
	// entry is totally invalid.
	e.canBeValid = false

	// Check for a uninitialized public key/signature.
	if pk.point == nil || signature.s == nil {
		return
	}

	// Signature deserialization checks that the signature is well-formed,
	// *except* for doing point-decompression on R.
	if _, err := e.R.SetCompressed(&signature.rCompressed); err != nil {
		return
	}
	e.S.Set(signature.s)
	e.A.Set(pk.point)

	// Calculate the challenge scalar (hram aka k).
	e.hram.Set(deriveVerifyChallengeScalar(pk, transcript, signature))

	// Calculate the transcript's delinearization component.
	if err := transcript.witnessBytes(e.witnessBytes[:], "", nil, zeroreader.ZeroReader{}); err != nil {
		panic("sr25519: failed to generate transcript delinearization value: " + err.Error())
	}
	e.witnessA = pk.compressed
	e.witnessR = signature.rCompressed

	// Ok, so the signature and public key appear to be well-formed,
	// so it is possible for the entry to be valid.
	e.canBeValid = true
}

// Add adds a (public key, transcript, signature) triple to the current
// batch.
func (v *BatchVerifier) Add(pk *PublicKey, transcript *SigningTranscript, signature *Signature) {
	var e entry

	e.doInit(pk, transcript, signature)
	v.anyInvalid = v.anyInvalid || !e.canBeValid
	v.entries = append(v.entries, e)
}

// VerifyBatchOnly checks all entries in the current batch using entropy
// from rand, returning true if all entries are valid and false if any one
// entry is invalid.  If rand is nil, crypto/rand.Reader will be used.
//
// If a failure arises it is unknown which entry failed, the caller must
// verify each entry individually.
func (v *BatchVerifier) VerifyBatchOnly(rand io.Reader) bool {
	if rand == nil {
		rand = cryptorand.Reader
	}

	vl := len(v.entries)
	numTerms := 1 + vl + vl

	// Handle some early aborts.
	switch {
	case vl == 0:
		// Abort early on an empty batch, which probably indicates a bug
		return false
	case v.anyInvalid:
		// Abort early if any of the `Add` calls failed to fully execute,
		// since at least one entry is invalid.
		return false
	}

	// The batch verification equation is
	//
	// [-sum(z_i * s_i)]B + sum([z_i]R_i) + sum([z_i * k_i]A_i) = 0.
	// where for each signature i,
	// - A_i is the verification key;
	// - R_i is the signature's R value;
	// - s_i is the signature's s value;
	// - k_i is the hash of the message and other data;
	// - z_i is a random 128-bit Scalar.
	svals := make([]scalar.Scalar, numTerms)
	scalars := make([]*scalar.Scalar, numTerms)

	// Populate scalars variable with concrete scalars to reduce heap allocation
	for i := range scalars {
		scalars[i] = &svals[i]
	}

	Bcoeff := scalars[0]
	Rcoeffs := scalars[1 : 1+vl]
	Acoeffs := scalars[1+vl:]

	// No need to allocate a backing-store since B, Rs and As already
	// have concrete instances.
	points := make([]*curve.RistrettoPoint, numTerms) // B | Rs | As
	Rs := points[1 : 1+vl]
	As := points[1+vl:]

	// Accumulate public keys, signatures, and transcripts for
	// delinearization.
	//
	// Note: Iterating over v repeatedly is kind of gross, but it's
	// what the Rust implementation does.  I'm not convinced that
	// it matters, but for now do the same thing.
	zs_t := &SigningTranscript{
		t: merlin.NewTranscript("V-RNG"),
	}
	for i := range v.entries {
		zs_t.commitPoint("", &v.entries[i].witnessA)
	}
	for i := range v.entries {
		zs_t.commitPoint("", &v.entries[i].witnessR)
	}
	for i := range v.entries {
		zs_t.commitBytes("", v.entries[i].witnessBytes[:])
	}
	zs_rng, err := zs_t.witnessRng("", nil, rand)
	if err != nil {
		panic("sr25519: failed to instantiate delinearization rng: " + err.Error())
	}

	points[0] = curve.RISTRETTO_BASEPOINT_POINT // B
	var randomBytes [scalar.ScalarSize]byte
	for i := range v.entries {
		// Avoid range copying each v.entries[i] literal.
		entry := &v.entries[i]
		Rs[i] = &entry.R
		As[i] = &entry.A

		// An inquisitive reader would ask why this doesn't just do
		// `z.SetRandom(rand)`, and instead, opts to duplicate the code.
		//
		// Go's escape analysis fails to realize that `randomBytes`
		// doesn't escape, so doing this saves n-1 allocations,
		// which can be quite large, especially as the batch size
		// increases.
		//
		// Additionally, we want z_i to be 128-bit scalars, so only
		// sampling 128-bits, and skipping the reduction is more
		// performant.
		if _, err = io.ReadFull(zs_rng, randomBytes[:scalar.ScalarSize/2]); err != nil {
			panic("sr25519: failed to generate batch verification scalar: " + err.Error())
		}
		scalar128.FixRawRangeVartime(&randomBytes) // Be extra paranoid, and deal with 0.
		if _, err = Rcoeffs[i].SetBits(randomBytes[:]); err != nil {
			panic("sr25519: failed to deserialize batch verification scalar: " + err.Error())
		}

		var sz scalar.Scalar
		Bcoeff.Add(Bcoeff, sz.Mul(Rcoeffs[i], &entry.S))
		Acoeffs[i].Mul(Rcoeffs[i], &entry.hram)
	}
	Bcoeff.Neg(Bcoeff) // this term is subtracted in the summation

	// Check the batch verification equation.
	var shouldBeId curve.RistrettoPoint
	return shouldBeId.MultiscalarMulVartime(scalars, points).IsIdentity()
}

// Verify checks all entries in the current batch using entropy from rand,
// returning true if all entries in the current bach are valid.  If one or
// more signature is invalid, each entry in the batch will be verified
// serially, and the returned bit-vector will provide information about
// each individual entry.  If rand is nil, crypto/rand.Reader will be used.
//
// Note: This method is only faster than individually verifying each
// signature if every signature is valid.  That said, this method will
// always out-perform calling VerifyBatchOnly followed by falling back
// to serial verification.
func (v *BatchVerifier) Verify(rand io.Reader) (bool, []bool) {
	vl := len(v.entries)
	if vl == 0 {
		return false, nil
	}

	// Start by assuming everything is valid, unless we know for sure
	// otherwise (ie: public key/signature/options were malformed).
	valid := make([]bool, vl)
	for i := range v.entries {
		valid[i] = v.entries[i].canBeValid
	}

	if !v.anyInvalid {
		if v.VerifyBatchOnly(rand) {
			// Fast-path, the entire batch is valid.
			return true, valid
		}
	}

	// Slow-path, one or more signatures is invalid with overwhelming
	// probability.  The results of serial verification is held to be
	// correct.
	allValid := !v.anyInvalid
	for i := range v.entries {
		// If the entry is known to be invalid, skip the serial
		// verification.
		if !valid[i] {
			continue
		}

		entry := &v.entries[i]

		var (
			negA  curve.RistrettoPoint
			rDiff curve.RistrettoPoint
		)
		negA.Neg(&entry.A)
		valid[i] = rDiff.TripleScalarMulBasepointVartime(&entry.hram, &negA, &entry.S, &entry.R).IsIdentity()
		allValid = allValid && valid[i]
	}

	return allValid, valid
}

// Reset resets a batch for reuse.
//
// Note: This method will reuse the existing entires slice to reduce memory
// reallocations.  If the next batch is known to be significantly smaller
// it may be more memory efficient to simply create a new batch.
func (v *BatchVerifier) Reset() *BatchVerifier {
	// Allow re-using the existing entries slice.
	v.entries = v.entries[:0]

	v.anyInvalid = false

	return v
}

// NewBatchVerifier creates an empty BatchVerifier.
func NewBatchVerifier() *BatchVerifier {
	return &BatchVerifier{}
}

// NewBatchVerifierWithCapacity creates an empty BatchVerifier, with
// preallocations done for a pre-determined batch size.
func NewBatchVerifierWithCapacity(n int) *BatchVerifier {
	v := NewBatchVerifier()
	if n > 0 {
		v.entries = make([]entry, 0, n)
	}

	return v
}
