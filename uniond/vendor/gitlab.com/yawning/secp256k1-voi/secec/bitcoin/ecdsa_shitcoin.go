// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

// Package bitcoin implements the bitcoin specific primitives.
package bitcoin

import (
	"crypto"
	_ "crypto/sha256" // Pull in SHA256

	"gitlab.com/yawning/secp256k1-voi/secec"
)

var optsShitcoin = &secec.ECDSAOptions{
	Hash:            crypto.SHA256,
	Encoding:        secec.EncodingASN1,
	RejectMalleable: true,
}

// VerifyASN1 verifies the BIP-0066 encoded signature `sig` of
// `digest`, using the PublicKey `k`, using the verification procedure
// as specified in SEC 1, Version 2.0, Section 4.1.4, with the
// additional restriction that `s` MUST be less than or equal
// to `n / 2`. Its return value records whether the signature
// is valid.
//
// Note: The signature MUST have the trailing `sighash` byte.
func VerifyASN1(k *secec.PublicKey, digest, sig []byte) bool {
	if !IsValidSignatureEncodingBIP0066(sig) {
		return false
	}

	return k.Verify(digest, sig[:len(sig)-1], optsShitcoin)
}
