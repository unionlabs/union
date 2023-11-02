// Copyright (c) 2013-2014 The btcsuite developers
// Copyright (c) 2015-2023 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package secp256k1

import (
	cryptorand "crypto/rand"
	"io"
)

// PrivateKey provides facilities for working with secp256k1 private keys within
// this package and includes functionality such as serializing and parsing them
// as well as computing their associated public key.
type PrivateKey struct {
	Key ModNScalar
}

// NewPrivateKey instantiates a new private key from a scalar encoded as a
// big integer.
func NewPrivateKey(key *ModNScalar) *PrivateKey {
	return &PrivateKey{Key: *key}
}

// PrivKeyFromBytes returns a private based on the provided byte slice which is
// interpreted as an unsigned 256-bit big-endian integer in the range [0, N-1],
// where N is the order of the curve.
//
// WARNING: This means passing a slice with more than 32 bytes is truncated and
// that truncated value is reduced modulo N.  Further, 0 is not a valid private
// key.  It is up to the caller to provide a value in the appropriate range of
// [1, N-1].  Failure to do so will either result in an invalid private key or
// potentially weak private keys that have bias that could be exploited.
//
// This function primarily exists to provide a mechanism for converting
// serialized private keys that are already known to be good.
//
// Typically callers should make use of GeneratePrivateKey or
// GeneratePrivateKeyFromRand when creating private keys since they properly
// handle generation of appropriate values.
func PrivKeyFromBytes(privKeyBytes []byte) *PrivateKey {
	var privKey PrivateKey
	privKey.Key.SetByteSlice(privKeyBytes)
	return &privKey
}

// generatePrivateKey generates and returns a new private key that is suitable
// for use with secp256k1 using the provided reader as a source of entropy.  The
// provided reader must be a source of cryptographically secure randomness to
// avoid weak private keys.
func generatePrivateKey(rand io.Reader) (*PrivateKey, error) {
	// The group order is close enough to 2^256 that there is only roughly a 1
	// in 2^128 chance of generating an invalid private key, so this loop will
	// virtually never run more than a single iteration in practice.
	var key PrivateKey
	var b32 [32]byte
	for valid := false; !valid; {
		if _, err := io.ReadFull(rand, b32[:]); err != nil {
			return nil, err
		}

		// The private key is only valid when it is in the range [1, N-1], where
		// N is the order of the curve.
		overflow := key.Key.SetBytes(&b32)
		valid = (key.Key.IsZeroBit() | overflow) == 0
	}
	zeroArray32(&b32)

	return &key, nil
}

// GeneratePrivateKey generates and returns a new cryptographically secure
// private key that is suitable for use with secp256k1.
func GeneratePrivateKey() (*PrivateKey, error) {
	return generatePrivateKey(cryptorand.Reader)
}

// GeneratePrivateKeyFromRand generates a private key that is suitable for use
// with secp256k1 using the provided reader as a source of entropy.  The
// provided reader must be a source of cryptographically secure randomness, such
// as [crypto/rand.Reader], to avoid weak private keys.
func GeneratePrivateKeyFromRand(rand io.Reader) (*PrivateKey, error) {
	return generatePrivateKey(rand)
}

// PubKey computes and returns the public key corresponding to this private key.
func (p *PrivateKey) PubKey() *PublicKey {
	var result JacobianPoint
	ScalarBaseMultNonConst(&p.Key, &result)
	result.ToAffine()
	return NewPublicKey(&result.X, &result.Y)
}

// Zero manually clears the memory associated with the private key.  This can be
// used to explicitly clear key material from memory for enhanced security
// against memory scraping.
func (p *PrivateKey) Zero() {
	p.Key.Zero()
}

// PrivKeyBytesLen defines the length in bytes of a serialized private key.
const PrivKeyBytesLen = 32

// Serialize returns the private key as a 256-bit big-endian binary-encoded
// number, padded to a length of 32 bytes.
func (p PrivateKey) Serialize() []byte {
	var privKeyBytes [PrivKeyBytesLen]byte
	p.Key.PutBytes(&privKeyBytes)
	return privKeyBytes[:]
}
