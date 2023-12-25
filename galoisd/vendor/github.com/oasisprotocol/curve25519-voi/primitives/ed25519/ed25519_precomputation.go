// Copyright (c) 2019-2021 Oasis Labs Inc. All rights reserved.
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
	"crypto/sha512"
	"fmt"

	"github.com/oasisprotocol/curve25519-voi/curve"
	"github.com/oasisprotocol/curve25519-voi/curve/scalar"
)

// ExpandedPublicKey is a PublicKey stored in an expanded representation
// for the purpose of accelerating repeated signature verification.
//
// Precomputation will be faster if more than 1 verification will be
// done, and each ExpandedPublicKey is ~1.47 KiB in size.
type ExpandedPublicKey struct {
	compressed curve.CompressedEdwardsY

	negA curve.ExpandedEdwardsPoint

	isValidY     bool
	isSmallOrder bool
	isCanonical  bool
}

// CompressedY returns the unexpanded public key as a compressed Edwards
// y-coordinate.
func (k *ExpandedPublicKey) CompressedY() curve.CompressedEdwardsY {
	return k.compressed
}

func (vOpts *VerifyOptions) checkExpandedPublicKey(publicKey *ExpandedPublicKey) bool {
	// This is equivalent to VerifyOptions.unpackPublicKey, but all of
	// the outcomes are cached at the precompute step.
	if !publicKey.isValidY {
		return false
	}

	if !vOpts.AllowSmallOrderA && publicKey.isSmallOrder {
		return false
	}
	if !vOpts.AllowNonCanonicalA && !publicKey.isCanonical {
		return false
	}

	return true
}

// NewExpandedPublicKey creates a new expanded public key from an existing
// public key.
func NewExpandedPublicKey(publicKey PublicKey) (*ExpandedPublicKey, error) {
	var pre ExpandedPublicKey

	// Unpack the point.
	var p curve.EdwardsPoint
	if _, err := pre.compressed.SetBytes(publicKey); err != nil {
		return nil, fmt.Errorf("ed25519: invalid public key: %w", err)
	}
	if _, err := p.SetCompressedY(&pre.compressed); err != nil {
		return nil, fmt.Errorf("ed25519: failed to decompress public key: %w", err)
	}

	// Check before negating the point.
	pre.isSmallOrder = p.IsSmallOrder()
	pre.isCanonical = pre.compressed.IsCanonicalVartime()

	// Serial verification uses -A, batch verification can just negate
	// the corresponding scalar (dirt cheap), and it saves carrying
	// another table around.
	pre.negA.SetEdwardsPoint(p.Neg(&p))
	pre.isValidY = true

	return &pre, nil
}

// VerifyExpanded reports whether sig is a valid Ed25519
// signature by publicKey.
func VerifyExpanded(publicKey *ExpandedPublicKey, message, sig []byte) bool {
	return VerifyExpandedWithOptions(publicKey, message, sig, optionsDefault)
}

// VerifyExpandedWithOptions reports whether sig is a valid Ed25519
// signature by publicKey with the extra Options to support Ed25519ph
// (pre-hashed by SHA-512) or Ed25519ctx (includes a domain separation
// context). It will panic if len(message) is not sha512.Size
// (if pre-hashed), len(opts.Context) is greater than ContextMaxSize,
// or opts is nil.
func VerifyExpandedWithOptions(publicKey *ExpandedPublicKey, message, sig []byte, opts *Options) bool {
	ok, err := verifyExpandedWithOptionsNoPanic(publicKey, message, sig, opts)
	if err != nil {
		panic(err)
	}

	return ok
}

func verifyExpandedWithOptionsNoPanic(publicKey *ExpandedPublicKey, message, sig []byte, opts *Options) (bool, error) {
	// This is equivalent to verifyWithOptionsNoPanic, but it uses
	// a expanded public key.  The reason why this a separate
	// routine is because creating an ExpanedPublicKey on the fly
	// incurs heap allocation overhead.
	f, context, err := opts.verify()
	if err != nil {
		return false, err
	}
	vOpts := opts.Verify
	if vOpts == nil {
		vOpts = VerifyOptionsDefault
	}

	f, err = checkHash(f, message, opts.HashFunc())
	if err != nil {
		return false, err
	}

	if ok := vOpts.checkExpandedPublicKey(publicKey); !ok {
		return false, nil
	}

	var (
		checkR curve.EdwardsPoint
		S      scalar.Scalar
	)
	if ok := vOpts.unpackSignature(sig, &checkR, &S); !ok {
		return false, nil
	}

	var (
		hash [64]byte
		hram scalar.Scalar
	)
	h := sha512.New()
	if dom2 := makeDom2(f, context); dom2 != nil {
		_, _ = h.Write(dom2)
	}
	_, _ = h.Write(sig[:32])
	_, _ = h.Write(publicKey.compressed[:])
	_, _ = h.Write(message)
	h.Sum(hash[:0])
	if _, err = hram.SetBytesModOrderWide(hash[:]); err != nil {
		return false, fmt.Errorf("ed25519: failed to deserialize H(R,A,m) scalar: %w", err)
	}

	// `-A` is already derived as part of precomputation (For `SB - H(R,A,m)A`)
	negA := &publicKey.negA

	if vOpts.CofactorlessVerify {
		var R curve.EdwardsPoint
		R.ExpandedDoubleScalarMulBasepointVartime(&hram, negA, &S)
		return cofactorlessVerify(&R, sig), nil
	}

	var rDiff curve.EdwardsPoint
	return rDiff.ExpandedTripleScalarMulBasepointVartime(&hram, negA, &S, &checkR).IsSmallOrder(), nil
}
