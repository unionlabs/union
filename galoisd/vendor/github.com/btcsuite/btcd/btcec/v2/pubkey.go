// Copyright (c) 2013-2014 The btcsuite developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package btcec

import (
	secp "github.com/decred/dcrd/dcrec/secp256k1/v4"
)

// These constants define the lengths of serialized public keys.
const (
	PubKeyBytesLenCompressed = 33
)

const (
	pubkeyCompressed   byte = 0x2 // y_bit + x coord
	pubkeyUncompressed byte = 0x4 // x coord + y coord
	pubkeyHybrid       byte = 0x6 // y_bit + x coord + y coord
)

// IsCompressedPubKey returns true the the passed serialized public key has
// been encoded in compressed format, and false otherwise.
func IsCompressedPubKey(pubKey []byte) bool {
	// The public key is only compressed if it is the correct length and
	// the format (first byte) is one of the compressed pubkey values.
	return len(pubKey) == PubKeyBytesLenCompressed &&
		(pubKey[0]&^byte(0x1) == pubkeyCompressed)
}

// ParsePubKey parses a public key for a koblitz curve from a bytestring into a
// ecdsa.Publickey, verifying that it is valid. It supports compressed,
// uncompressed and hybrid signature formats.
func ParsePubKey(pubKeyStr []byte) (*PublicKey, error) {
	return secp.ParsePubKey(pubKeyStr)
}

// PublicKey is an ecdsa.PublicKey with additional functions to
// serialize in uncompressed, compressed, and hybrid formats.
type PublicKey = secp.PublicKey

// NewPublicKey instantiates a new public key with the given x and y
// coordinates.
//
// It should be noted that, unlike ParsePubKey, since this accepts arbitrary x
// and y coordinates, it allows creation of public keys that are not valid
// points on the secp256k1 curve.  The IsOnCurve method of the returned instance
// can be used to determine validity.
func NewPublicKey(x, y *FieldVal) *PublicKey {
	return secp.NewPublicKey(x, y)
}
