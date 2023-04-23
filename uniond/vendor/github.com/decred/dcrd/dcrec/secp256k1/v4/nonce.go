// Copyright (c) 2013-2014 The btcsuite developers
// Copyright (c) 2015-2020 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package secp256k1

import (
	"bytes"
	"crypto/sha256"
	"hash"
)

// References:
//   [GECC]: Guide to Elliptic Curve Cryptography (Hankerson, Menezes, Vanstone)
//
//   [ISO/IEC 8825-1]: Information technology — ASN.1 encoding rules:
//     Specification of Basic Encoding Rules (BER), Canonical Encoding Rules
//     (CER) and Distinguished Encoding Rules (DER)
//
//   [SEC1]: Elliptic Curve Cryptography (May 31, 2009, Version 2.0)
//     https://www.secg.org/sec1-v2.pdf

var (
	// singleZero is used during RFC6979 nonce generation.  It is provided
	// here to avoid the need to create it multiple times.
	singleZero = []byte{0x00}

	// zeroInitializer is used during RFC6979 nonce generation.  It is provided
	// here to avoid the need to create it multiple times.
	zeroInitializer = bytes.Repeat([]byte{0x00}, sha256.BlockSize)

	// singleOne is used during RFC6979 nonce generation.  It is provided
	// here to avoid the need to create it multiple times.
	singleOne = []byte{0x01}

	// oneInitializer is used during RFC6979 nonce generation.  It is provided
	// here to avoid the need to create it multiple times.
	oneInitializer = bytes.Repeat([]byte{0x01}, sha256.Size)
)

// hmacsha256 implements a resettable version of HMAC-SHA256.
type hmacsha256 struct {
	inner, outer hash.Hash
	ipad, opad   [sha256.BlockSize]byte
}

// Write adds data to the running hash.
func (h *hmacsha256) Write(p []byte) {
	h.inner.Write(p)
}

// initKey initializes the HMAC-SHA256 instance to the provided key.
func (h *hmacsha256) initKey(key []byte) {
	// Hash the key if it is too large.
	if len(key) > sha256.BlockSize {
		h.outer.Write(key)
		key = h.outer.Sum(nil)
	}
	copy(h.ipad[:], key)
	copy(h.opad[:], key)
	for i := range h.ipad {
		h.ipad[i] ^= 0x36
	}
	for i := range h.opad {
		h.opad[i] ^= 0x5c
	}
	h.inner.Write(h.ipad[:])
}

// ResetKey resets the HMAC-SHA256 to its initial state and then initializes it
// with the provided key.  It is equivalent to creating a new instance with the
// provided key without allocating more memory.
func (h *hmacsha256) ResetKey(key []byte) {
	h.inner.Reset()
	h.outer.Reset()
	copy(h.ipad[:], zeroInitializer)
	copy(h.opad[:], zeroInitializer)
	h.initKey(key)
}

// Resets the HMAC-SHA256 to its initial state using the current key.
func (h *hmacsha256) Reset() {
	h.inner.Reset()
	h.inner.Write(h.ipad[:])
}

// Sum returns the hash of the written data.
func (h *hmacsha256) Sum() []byte {
	h.outer.Reset()
	h.outer.Write(h.opad[:])
	h.outer.Write(h.inner.Sum(nil))
	return h.outer.Sum(nil)
}

// newHMACSHA256 returns a new HMAC-SHA256 hasher using the provided key.
func newHMACSHA256(key []byte) *hmacsha256 {
	h := new(hmacsha256)
	h.inner = sha256.New()
	h.outer = sha256.New()
	h.initKey(key)
	return h
}

// NonceRFC6979 generates a nonce deterministically according to RFC 6979 using
// HMAC-SHA256 for the hashing function.  It takes a 32-byte hash as an input
// and returns a 32-byte nonce to be used for deterministic signing.  The extra
// and version arguments are optional, but allow additional data to be added to
// the input of the HMAC.  When provided, the extra data must be 32-bytes and
// version must be 16 bytes or they will be ignored.
//
// Finally, the extraIterations parameter provides a method to produce a stream
// of deterministic nonces to ensure the signing code is able to produce a nonce
// that results in a valid signature in the extremely unlikely event the
// original nonce produced results in an invalid signature (e.g. R == 0).
// Signing code should start with 0 and increment it if necessary.
func NonceRFC6979(privKey []byte, hash []byte, extra []byte, version []byte, extraIterations uint32) *ModNScalar {
	// Input to HMAC is the 32-byte private key and the 32-byte hash.  In
	// addition, it may include the optional 32-byte extra data and 16-byte
	// version.  Create a fixed-size array to avoid extra allocs and slice it
	// properly.
	const (
		privKeyLen = 32
		hashLen    = 32
		extraLen   = 32
		versionLen = 16
	)
	var keyBuf [privKeyLen + hashLen + extraLen + versionLen]byte

	// Truncate rightmost bytes of private key and hash if they are too long and
	// leave left padding of zeros when they're too short.
	if len(privKey) > privKeyLen {
		privKey = privKey[:privKeyLen]
	}
	if len(hash) > hashLen {
		hash = hash[:hashLen]
	}
	offset := privKeyLen - len(privKey) // Zero left padding if needed.
	offset += copy(keyBuf[offset:], privKey)
	offset += hashLen - len(hash) // Zero left padding if needed.
	offset += copy(keyBuf[offset:], hash)
	if len(extra) == extraLen {
		offset += copy(keyBuf[offset:], extra)
		if len(version) == versionLen {
			offset += copy(keyBuf[offset:], version)
		}
	} else if len(version) == versionLen {
		// When the version was specified, but not the extra data, leave the
		// extra data portion all zero.
		offset += privKeyLen
		offset += copy(keyBuf[offset:], version)
	}
	key := keyBuf[:offset]

	// Step B.
	//
	// V = 0x01 0x01 0x01 ... 0x01 such that the length of V, in bits, is
	// equal to 8*ceil(hashLen/8).
	//
	// Note that since the hash length is a multiple of 8 for the chosen hash
	// function in this optimized implementation, the result is just the hash
	// length, so avoid the extra calculations.  Also, since it isn't modified,
	// start with a global value.
	v := oneInitializer

	// Step C (Go zeroes all allocated memory).
	//
	// K = 0x00 0x00 0x00 ... 0x00 such that the length of K, in bits, is
	// equal to 8*ceil(hashLen/8).
	//
	// As above, since the hash length is a multiple of 8 for the chosen hash
	// function in this optimized implementation, the result is just the hash
	// length, so avoid the extra calculations.
	k := zeroInitializer[:hashLen]

	// Step D.
	//
	// K = HMAC_K(V || 0x00 || int2octets(x) || bits2octets(h1))
	//
	// Note that key is the "int2octets(x) || bits2octets(h1)" portion along
	// with potential additional data as described by section 3.6 of the RFC.
	hasher := newHMACSHA256(k)
	hasher.Write(oneInitializer)
	hasher.Write(singleZero[:])
	hasher.Write(key)
	k = hasher.Sum()

	// Step E.
	//
	// V = HMAC_K(V)
	hasher.ResetKey(k)
	hasher.Write(v)
	v = hasher.Sum()

	// Step F.
	//
	// K = HMAC_K(V || 0x01 || int2octets(x) || bits2octets(h1))
	//
	// Note that key is the "int2octets(x) || bits2octets(h1)" portion along
	// with potential additional data as described by section 3.6 of the RFC.
	hasher.Reset()
	hasher.Write(v)
	hasher.Write(singleOne[:])
	hasher.Write(key[:])
	k = hasher.Sum()

	// Step G.
	//
	// V = HMAC_K(V)
	hasher.ResetKey(k)
	hasher.Write(v)
	v = hasher.Sum()

	// Step H.
	//
	// Repeat until the value is nonzero and less than the curve order.
	var generated uint32
	for {
		// Step H1 and H2.
		//
		// Set T to the empty sequence.  The length of T (in bits) is denoted
		// tlen; thus, at that point, tlen = 0.
		//
		// While tlen < qlen, do the following:
		//   V = HMAC_K(V)
		//   T = T || V
		//
		// Note that because the hash function output is the same length as the
		// private key in this optimized implementation, there is no need to
		// loop or create an intermediate T.
		hasher.Reset()
		hasher.Write(v)
		v = hasher.Sum()

		// Step H3.
		//
		// k = bits2int(T)
		// If k is within the range [1,q-1], return it.
		//
		// Otherwise, compute:
		// K = HMAC_K(V || 0x00)
		// V = HMAC_K(V)
		var secret ModNScalar
		overflow := secret.SetByteSlice(v)
		if !overflow && !secret.IsZero() {
			generated++
			if generated > extraIterations {
				return &secret
			}
		}

		// K = HMAC_K(V || 0x00)
		hasher.Reset()
		hasher.Write(v)
		hasher.Write(singleZero[:])
		k = hasher.Sum()

		// V = HMAC_K(V)
		hasher.ResetKey(k)
		hasher.Write(v)
		v = hasher.Sum()
	}
}
