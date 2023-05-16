// Copyright (c) 2013-2016 The btcsuite developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package btcec

import (
	secp "github.com/decred/dcrd/dcrec/secp256k1/v4"
)

// PrivateKey wraps an ecdsa.PrivateKey as a convenience mainly for signing
// things with the the private key without having to directly import the ecdsa
// package.
type PrivateKey = secp.PrivateKey

// PrivKeyFromBytes returns a private and public key for `curve' based on the
// private key passed as an argument as a byte slice.
func PrivKeyFromBytes(pk []byte) (*PrivateKey, *PublicKey) {
	privKey := secp.PrivKeyFromBytes(pk)

	return privKey, privKey.PubKey()
}

// NewPrivateKey is a wrapper for ecdsa.GenerateKey that returns a PrivateKey
// instead of the normal ecdsa.PrivateKey.
func NewPrivateKey() (*PrivateKey, error) {
	return secp.GeneratePrivateKey()
}

// PrivKeyFromScalar instantiates a new private key from a scalar encoded as a
// big integer.
func PrivKeyFromScalar(key *ModNScalar) *PrivateKey {
	return &PrivateKey{Key: *key}
}

// PrivKeyBytesLen defines the length in bytes of a serialized private key.
const PrivKeyBytesLen = 32
