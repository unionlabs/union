// Copyright (c) 2020 Henry de Valence. All rights reserved.
// Copyright (c) 2020-2021 Oasis Labs Inc. All rights reserved.
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
	cryptorand "crypto/rand"
	"crypto/sha512"
	"io"

	"github.com/oasisprotocol/curve25519-voi/curve"
	"github.com/oasisprotocol/curve25519-voi/curve/scalar"
	"github.com/oasisprotocol/curve25519-voi/internal/scalar128"
)

const batchPippengerThreshold = (190 - 1) / 2

// BatchVerifier accumulates batch entries with Add, before performing
// batch verification with Verify.
type BatchVerifier struct {
	entries []entry

	anyInvalid      bool
	anyCofactorless bool
	anyNotExpanded  bool
}

type entry struct {
	signature []byte

	// Note: Unlike ed25519consensus, this stores R, -A, S, and hram
	// in the entry, so that it is possible to implement a more
	// useful verification API (information about which signature(s)
	// are invalid is wanted a lot of the time), without having to
	// redo a non-trivial amount of computation.
	R    curve.EdwardsPoint
	negA curve.EdwardsPoint
	S    scalar.Scalar
	hram scalar.Scalar

	// Additionally provisions are made for A to *also* be stored
	// in expanded form, so that precomputation can be leveraged
	// to accelerated the multiscalar multiply for moderate sized
	// batches, and the serial verification in the event of a batch
	// failure.
	expandedA *ExpandedPublicKey

	wantCofactorless bool
	canBeValid       bool
}

func (e *entry) doInit(publicKey PublicKey, expandedPublicKey *ExpandedPublicKey, message, sig []byte, opts *Options) {
	// Until everything has been deserialized correctly, assume the
	// entry is totally invalid.
	e.canBeValid = false

	fBase, context, err := opts.verify()
	if err != nil {
		return
	}
	vOpts := opts.Verify
	if vOpts == nil {
		vOpts = VerifyOptionsDefault
	}

	// This is nonsensical (as in, cofactorless batch-verification
	// is flat out incorrect), but the API allows for requesting it.
	if e.wantCofactorless = vOpts.CofactorlessVerify; e.wantCofactorless {
		e.signature = sig
	}

	// Validate A, Deserialize R and S.
	var compressedA []byte
	if expandedPublicKey != nil {
		if ok := vOpts.checkExpandedPublicKey(expandedPublicKey); !ok {
			return
		}
		e.expandedA = expandedPublicKey
		e.negA.SetExpanded(&expandedPublicKey.negA)
		compressedA = expandedPublicKey.compressed[:]
	} else {
		if ok := vOpts.unpackPublicKey(publicKey, &e.negA); !ok {
			return
		}
		e.expandedA = nil
		e.negA.Neg(&e.negA)
		compressedA = publicKey
	}
	if ok := vOpts.unpackSignature(sig, &e.R, &e.S); !ok {
		return
	}

	// Calculate H(R,A,m).
	var (
		f dom2Flag

		hash [64]byte
		h    = sha512.New()
	)

	if f, err = checkHash(fBase, message, opts.HashFunc()); err != nil {
		return
	}

	if dom2 := makeDom2(f, context); dom2 != nil {
		_, _ = h.Write(dom2)
	}
	_, _ = h.Write(sig[:32])
	_, _ = h.Write(compressedA)
	_, _ = h.Write(message)
	h.Sum(hash[:0])
	if _, err = e.hram.SetBytesModOrderWide(hash[:]); err != nil {
		return
	}

	// Ok, R, A, S, and hram can at least be deserialized/computed,
	// so it is possible for the entry to be valid.
	e.canBeValid = true
}

// Add adds a (public key, message, sig) triple to the current batch.
func (v *BatchVerifier) Add(publicKey PublicKey, message, sig []byte) {
	v.AddWithOptions(publicKey, message, sig, optionsDefault)
}

// AddWithOptions adds a (public key, message, sig, opts) quad to the
// current batch.
//
// WARNING: This routine will panic if opts is nil.
func (v *BatchVerifier) AddWithOptions(publicKey PublicKey, message, sig []byte, opts *Options) {
	// If everything added so far has an expanded public key, and the batch
	// is not too large, then expand the public key for increased performance.
	if v.precomputeOk() {
		// The error is explicitly discarded as doInit will do the
		// right thing if the public key is nil.
		precomputedPublicKey, _ := NewExpandedPublicKey(publicKey)
		v.AddExpandedWithOptions(precomputedPublicKey, message, sig, opts)
		return
	}

	// Add an entry without the expanded public key.
	var e entry

	e.doInit(publicKey, nil, message, sig, opts)
	v.anyInvalid = v.anyInvalid || !e.canBeValid
	v.anyCofactorless = v.anyCofactorless || e.wantCofactorless
	v.anyNotExpanded = true
	v.entries = append(v.entries, e)
}

// AddExpanded adds a (expanded public key, message, sig) triple to the
// current batch.
func (v *BatchVerifier) AddExpanded(publicKey *ExpandedPublicKey, message, sig []byte) {
	v.AddExpandedWithOptions(publicKey, message, sig, optionsDefault)
}

// AddExpandedWithOptions adds a (precomputed public key, message, sig,
// opts) quad to the current batch.
//
// WARNING: This routine will panic if opts is nil.
func (v *BatchVerifier) AddExpandedWithOptions(publicKey *ExpandedPublicKey, message, sig []byte, opts *Options) {
	var e entry

	e.doInit(nil, publicKey, message, sig, opts)
	v.anyInvalid = v.anyInvalid || !e.canBeValid
	v.anyCofactorless = v.anyCofactorless || e.wantCofactorless
	v.anyNotExpanded = v.anyNotExpanded || e.expandedA == nil // Can be omitted.
	v.entries = append(v.entries, e)
}

// VerifyBatchOnly checks all entries in the current batch using entropy
// from rand, returning true if all entries are valid and false if any one
// entry is invalid.  If rand is nil, crypto/rand.Reader will be used.
//
// If a failure arises it is unknown which entry failed, the caller must
// verify each entry individually.
//
// Calling VerifyBatchOnly on an empty batch, or a batch containing any
// entries that specify cofactor-less verification will return false.
func (v *BatchVerifier) VerifyBatchOnly(rand io.Reader) bool {
	if rand == nil {
		rand = cryptorand.Reader
	}

	vl := len(v.entries)
	numDynamic := 1 + vl
	numTerms := numDynamic + vl

	// Handle some early aborts.
	switch {
	case vl == 0:
		// Abort early on an empty batch, which probably indicates a bug
		return false
	case v.anyInvalid:
		// Abort early if any of the `Add`/`AddWithOptions` calls failed
		// to fully execute, since at least one entry is invalid.
		return false
	case v.anyCofactorless:
		// Abort early if any of the entries requested cofactor-less
		// verification, since that flat out doesn't work.
		return false
	}

	zGen, err := scalar128.NewGenerator(rand)
	if err != nil {
		panic("ed25519: failed to initialize random scalar generator: " + err.Error())
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

	// Prepare the various slices based on if this is precomputed or not.
	//
	// Note: There is no need to allocate a backing-store since B, Rs and
	// As already have concrete instances.
	var (
		points []*curve.EdwardsPoint
		Rs     []*curve.EdwardsPoint

		staticAs []*curve.ExpandedEdwardsPoint
		As       []*curve.EdwardsPoint
	)

	isPrecomputed := v.precomputeOk()

	if isPrecomputed {
		points = make([]*curve.EdwardsPoint, numDynamic)   // B | Rs
		staticAs = make([]*curve.ExpandedEdwardsPoint, vl) // As
	} else {
		points = make([]*curve.EdwardsPoint, numTerms) // B | Rs | As
		As = points[1+vl:]
	}
	points[0] = curve.ED25519_BASEPOINT_POINT // B
	Rs = points[1 : 1+vl]

	for i := range v.entries {
		// Avoid range copying each v.entries[i] literal.
		entry := &v.entries[i]
		Rs[i] = &entry.R
		if isPrecomputed {
			staticAs[i] = &entry.expandedA.negA
		} else {
			As[i] = &entry.negA
		}

		if err = zGen.SetScalarVartime(Rcoeffs[i]); err != nil {
			panic("ed25519: failed to generate z_i: " + err.Error())
		}

		var sz scalar.Scalar
		Bcoeff.Add(Bcoeff, sz.Mul(Rcoeffs[i], &entry.S))

		// The precomputation uses tables for -A, so to avoid having
		// to derive and store a separate table, the entry also contains
		// -A, for both the precomputed case and the non-precomputed
		// case (for simplicity).
		//
		// Calculate -[z_i * k_i] to fix the sign.
		Acoeffs[i].Mul(Rcoeffs[i], &entry.hram) // Acoeffs[i] = z_i * k_i
		Acoeffs[i].Neg(Acoeffs[i])              // Acoeffs[i] = -Acoeffs[i]
	}

	Bcoeff.Neg(Bcoeff) // this term is subtracted in the summation

	// Check the cofactored batch verification equation.
	var shouldBeId curve.EdwardsPoint
	if isPrecomputed {
		dynamicScalars := scalars[0 : 1+vl] // Bcoeff | Rcoeffs
		shouldBeId.ExpandedMultiscalarMulVartime(Acoeffs, staticAs, dynamicScalars, points)
	} else {
		shouldBeId.MultiscalarMulVartime(scalars, points)
	}
	return shouldBeId.IsSmallOrder()
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

	// If batch verification is possible, do the batch verification.
	if !v.anyInvalid && !v.anyCofactorless {
		if v.VerifyBatchOnly(rand) {
			// Fast-path, the entire batch is valid.
			return true, valid
		}
	}

	// Slow-path, one or more signatures is either invalid, or needs
	// cofactor-less verification.
	//
	// Note: In the case of the latter it is still possible for the
	// entire batch to be valid, but it is incorrect to trust the
	// batch verification results.
	allValid := !v.anyInvalid
	for i := range v.entries {
		// If the entry is known to be invalid, skip the serial
		// verification.
		if !valid[i] {
			continue
		}

		// If the entry has -A in expanded form, use the precomputed
		// multiplies since it will be faster.
		entry := &v.entries[i]
		if entry.expandedA != nil {
			negA := &entry.expandedA.negA
			if entry.wantCofactorless {
				var R curve.EdwardsPoint
				R.ExpandedDoubleScalarMulBasepointVartime(&entry.hram, negA, &entry.S)
				valid[i] = cofactorlessVerify(&R, entry.signature)
			} else {
				var rDiff curve.EdwardsPoint
				valid[i] = rDiff.ExpandedTripleScalarMulBasepointVartime(&entry.hram, negA, &entry.S, &entry.R).IsSmallOrder()
			}
		} else {
			negA := &entry.negA
			if entry.wantCofactorless {
				var R curve.EdwardsPoint
				R.DoubleScalarMulBasepointVartime(&entry.hram, negA, &entry.S)
				valid[i] = cofactorlessVerify(&R, entry.signature)
			} else {
				var rDiff curve.EdwardsPoint
				valid[i] = rDiff.TripleScalarMulBasepointVartime(&entry.hram, negA, &entry.S, &entry.R).IsSmallOrder()
			}
		}

		allValid = allValid && valid[i]
	}

	return allValid, valid
}

// ForceNoPublicKeyExpansion disables the key expansion for a given batch.
// This setting will NOT persist across Reset, and must be called again
// if a batch is reused.
//
// This setting can be used for large batches that are built via
// Add/AddWithOptions to reduce memory consumption and allocations.
func (v *BatchVerifier) ForceNoPublicKeyExpansion() *BatchVerifier {
	v.anyNotExpanded = true
	return v
}

// Reset resets a batch for reuse.
//
// Note: This method will reuse the internal entires slice to reduce memory
// reallocations.  If the next batch is known to be significantly smaller
// it may be more memory efficient to simply create a new batch.
func (v *BatchVerifier) Reset() *BatchVerifier {
	// Remove the reference to each expanded -A so that they may be
	// garbage collected.  The pointer will be overwritten on
	// subsequent batch verify calls.
	for _, ent := range v.entries {
		ent.expandedA = nil
	}

	// Allow re-using the existing entries slice.
	v.entries = v.entries[:0]

	// Reset the rest of the verifier state.
	v.anyInvalid = false
	v.anyCofactorless = false
	v.anyNotExpanded = false

	return v
}

func (v *BatchVerifier) precomputeOk() bool {
	return !v.anyNotExpanded && len(v.entries) < batchPippengerThreshold
}

// NewBatchVerfier creates an empty BatchVerifier.
func NewBatchVerifier() *BatchVerifier {
	return &BatchVerifier{}
}

// NewBatchVerifierWithCapacity creates an empty BatchVerifier, with
// preallocations for a pre-determined batch size hint.
func NewBatchVerifierWithCapacity(n int) *BatchVerifier {
	v := NewBatchVerifier()
	if n > 0 {
		v.entries = make([]entry, 0, n)
	}

	return v
}
