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

// Package cache implements a set of caching wrappers around Ed25519
// signature verification to transparently accelerate repeated verification
// with the same public key(s).
package cache

import (
	"github.com/oasisprotocol/curve25519-voi/curve"
	"github.com/oasisprotocol/curve25519-voi/primitives/ed25519"
)

// Cache is an expanded public key cache.
type Cache interface {
	// Get returns a public key's corresponding expanded public key iff
	// present in the cache, or returns nil.
	Get(publicKey *curve.CompressedEdwardsY) *ed25519.ExpandedPublicKey

	// Put adds the expanded public key to the cache.
	Put(publicKey *curve.CompressedEdwardsY, expanded *ed25519.ExpandedPublicKey)
}

// Verifier verifies signatures, storing expanded public keys in a cache
// for reuse by subsequent verification with the same public key.
//
// Note: Unless there are more cache hits than misses, this will likely
// be a net performance loss.  Integration should be followed by
// benchmarking.
type Verifier struct {
	cache Cache
}

// Verify repors whether sig is a valid Ed25519 signature by public key.
func (v *Verifier) Verify(publicKey ed25519.PublicKey, message, sig []byte) bool {
	return v.VerifyWithOptions(publicKey, message, sig, &ed25519.Options{})
}

// VerifyWithOptions reports whether sig is a valid Ed25519 signature by
// publicKey, with extra Options.
func (v *Verifier) VerifyWithOptions(publicKey ed25519.PublicKey, message, sig []byte, opts *ed25519.Options) bool {
	expanded, ok := v.upsertPublicKey(publicKey)
	if !ok {
		return false
	}

	return ed25519.VerifyExpandedWithOptions(expanded, message, sig, opts)
}

// Add will add the signature to the batch verifier.
func (v *Verifier) Add(verifier *ed25519.BatchVerifier, publicKey ed25519.PublicKey, message, sig []byte) {
	v.AddWithOptions(verifier, publicKey, message, sig, &ed25519.Options{})
}

// AddWithOptions will add the signature to the batch verifier, with
// extra Options.
func (v *Verifier) AddWithOptions(verifier *ed25519.BatchVerifier, publicKey ed25519.PublicKey, message, sig []byte, opts *ed25519.Options) {
	// Note: BatchVerifier.AddWithOptions will do the right thing if
	// the expanded public key is nil.
	expanded, _ := v.upsertPublicKey(publicKey)
	verifier.AddExpandedWithOptions(expanded, message, sig, opts)
}

// AddPublicKey will expand and add the public key to the cache.
func (v *Verifier) AddPublicKey(publicKey ed25519.PublicKey) {
	v.upsertPublicKey(publicKey)
}

func (v *Verifier) upsertPublicKey(publicKey ed25519.PublicKey) (*ed25519.ExpandedPublicKey, bool) {
	var (
		compressed curve.CompressedEdwardsY
		err        error
	)
	if _, err = compressed.SetBytes(publicKey); err != nil {
		return nil, false
	}

	expanded := v.cache.Get(&compressed)
	if expanded == nil {
		if expanded, err = ed25519.NewExpandedPublicKey(compressed[:]); err != nil {
			return nil, false
		}
		v.cache.Put(&compressed, expanded)
	}

	return expanded, true
}

// NewVerifier creates a new Verifier instance backed by a Cache.
func NewVerifier(cache Cache) *Verifier {
	return &Verifier{
		cache: cache,
	}
}
