// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

// Package h2c implements Hashing to Elliptic Curves as specified in RFC 9380.
package h2c

import (
	"crypto"
	_ "crypto/sha256" // Pull in SHA256

	"gitlab.com/yawning/secp256k1-voi"
)

const (
	ell = 48  // L = ceil((ceil(log2(p)) + k) / 8)
	kay = 128 // k = target security level in bits

	encodeToCurveSize = ell
	hashToCurveSize   = ell * 2
)

// Secp256k1_XMD_SHA256_SSWU_RO implements the secp256k1_XMD:SHA-256_SSWU_RO_
// h2c suite.
func Secp256k1_XMD_SHA256_SSWU_RO(domainSeparator, message []byte) (*secp256k1.Point, error) { //nolint:revive
	// 1. u = hash_to_field(msg, 2)
	var uBytes [hashToCurveSize]byte
	if err := expandMessageXMD(uBytes[:], crypto.SHA256, domainSeparator, message); err != nil {
		return nil, err
	}

	// 2. Q0 = map_to_curve(u[0])
	q0 := secp256k1.NewIdentityPoint().SetUniformBytes(uBytes[:ell])

	// 3. Q1 = map_to_curve(u[1])
	q1 := secp256k1.NewIdentityPoint().SetUniformBytes(uBytes[ell:])

	// 4. R = Q0 + Q1              # Point addition
	r := secp256k1.NewIdentityPoint().Add(q0, q1)

	// 5. P = clear_cofactor(R)
	// 6. return P

	return r, nil
}

// Secp256k1_XMD_SHA256_SSWU_NU implements the secp256k1_XMD:SHA-256_SSWU_NU_
// h2c suite.
func Secp256k1_XMD_SHA256_SSWU_NU(domainSeparator, message []byte) (*secp256k1.Point, error) { //nolint:revive
	// 1. u = hash_to_field(msg, 1)
	var uBytes [encodeToCurveSize]byte
	if err := expandMessageXMD(uBytes[:], crypto.SHA256, domainSeparator, message); err != nil {
		return nil, err
	}

	// 2. Q = map_to_curve(u[0])
	q := secp256k1.NewIdentityPoint().SetUniformBytes(uBytes[:])

	// 3. P = clear_cofactor(Q)
	// 4. return P

	return q, nil
}
