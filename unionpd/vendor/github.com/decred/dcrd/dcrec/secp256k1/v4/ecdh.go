// Copyright (c) 2015 The btcsuite developers
// Copyright (c) 2015-2016 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package secp256k1

// GenerateSharedSecret generates a shared secret based on a private key and a
// public key using Diffie-Hellman key exchange (ECDH) (RFC 5903).
// RFC5903 Section 9 states we should only return x.
//
// It is recommended to securily hash the result before using as a cryptographic
// key.
func GenerateSharedSecret(privkey *PrivateKey, pubkey *PublicKey) []byte {
	var point, result JacobianPoint
	pubkey.AsJacobian(&point)
	ScalarMultNonConst(&privkey.Key, &point, &result)
	result.ToAffine()
	xBytes := result.X.Bytes()
	return xBytes[:]
}
