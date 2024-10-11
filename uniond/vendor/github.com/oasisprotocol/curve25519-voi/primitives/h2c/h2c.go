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

// Package h2c implements the "Hashing to Elliptic Curves" IETF draft.
package h2c

import (
	"crypto"
	_ "crypto/sha512"
	"fmt"

	"golang.org/x/crypto/sha3"

	"github.com/oasisprotocol/curve25519-voi/curve"
	"github.com/oasisprotocol/curve25519-voi/internal/elligator"
	"github.com/oasisprotocol/curve25519-voi/internal/field"
	_ "github.com/oasisprotocol/curve25519-voi/internal/toolchain"
)

const (
	ell = 48  // L = ceil((ceil(log2(2^255-19)) + k) / 8)
	kay = 128 // k = target security level in bits

	encodeToCurveSize = ell
	hashToCurveSize   = ell * 2
)

// Edwards25519_XMD_SHA512_ELL2_RO implements the edwards25519_XMD:SHA-512_ELL2_RO_
// suite.
func Edwards25519_XMD_SHA512_ELL2_RO(domainSeparator, message []byte) (*curve.EdwardsPoint, error) {
	return Edwards25519_XMD_ELL2_RO(crypto.SHA512, domainSeparator, message)
}

// Edwards25519_XMD_SHA512_ELL2_NU implements the edwards25519_XMD:SHA-512_ELL2_NU_
// suite.
func Edwards25519_XMD_SHA512_ELL2_NU(domainSeparator, message []byte) (*curve.EdwardsPoint, error) {
	return Edwards25519_XMD_ELL2_NU(crypto.SHA512, domainSeparator, message)
}

// Edwards25519_XMD_ELL2_RO implements a generic edwards25519 random oracle suite
// using `expand_message_xmd`.
func Edwards25519_XMD_ELL2_RO(hFunc crypto.Hash, domainSeparator, message []byte) (*curve.EdwardsPoint, error) {
	var uniformBytes [hashToCurveSize]byte
	if err := ExpandMessageXMD(uniformBytes[:], hFunc, domainSeparator, message); err != nil {
		return nil, fmt.Errorf("h2c: failed to expand message: %w", err)
	}
	return hashToCurve(&uniformBytes), nil
}

// Edwards25519_XMD_ELL2_NU implements a generic edwards25519 nonuniform suite
// using `expand_messsage_xmd`.
func Edwards25519_XMD_ELL2_NU(hFunc crypto.Hash, domainSeparator, message []byte) (*curve.EdwardsPoint, error) {
	var uniformBytes [encodeToCurveSize]byte
	if err := ExpandMessageXMD(uniformBytes[:], hFunc, domainSeparator, message); err != nil {
		return nil, fmt.Errorf("h2c: failed to expand message: %w", err)
	}
	return encodeToCurve(&uniformBytes), nil
}

// Edwards25519_XOF_ELL2_RO implements a generic edwards25519 random oracle suite
// using `expand_message_xof`.
func Edwards25519_XOF_ELL2_RO(xofFunc sha3.ShakeHash, domainSeparator, message []byte) (*curve.EdwardsPoint, error) {
	var uniformBytes [hashToCurveSize]byte
	if err := ExpandMessageXOF(uniformBytes[:], xofFunc, domainSeparator, message); err != nil {
		return nil, fmt.Errorf("h2c: failed to expand message: %w", err)
	}
	return hashToCurve(&uniformBytes), nil
}

// Edwards25519_XOF_ELL2_NU implements a generic edwards25519 nonuniform suite
// using `expand_messsage_xof`.
func Edwards25519_XOF_ELL2_NU(xofFunc sha3.ShakeHash, domainSeparator, message []byte) (*curve.EdwardsPoint, error) {
	var uniformBytes [encodeToCurveSize]byte
	if err := ExpandMessageXOF(uniformBytes[:], xofFunc, domainSeparator, message); err != nil {
		return nil, fmt.Errorf("h2c: failed to expand message: %w", err)
	}
	return encodeToCurve(&uniformBytes), nil
}

// Ristretto255_XMD_R255MAP_RO implements a generic ristretto255 random oracle suite
// using `expand_message_xmd`.
func Ristretto255_XMD_R255MAP_RO(hFunc crypto.Hash, domainSeparator, message []byte) (*curve.RistrettoPoint, error) {
	var uniformBytes [curve.RistrettoUniformSize]byte
	if err := ExpandMessageXMD(uniformBytes[:], hFunc, domainSeparator, message); err != nil {
		return nil, fmt.Errorf("h2c: failed to expand message: %w", err)
	}

	var p curve.RistrettoPoint
	return p.SetUniformBytes(uniformBytes[:])
}

// Ristretto255_XOF_R255MAP_RO implements a generic ristretto255 random oracle suite
// using `expand_message_xof`.
func Ristretto255_XOF_R255MAP_RO(xofFunc sha3.ShakeHash, domainSeparator, message []byte) (*curve.RistrettoPoint, error) {
	var uniformBytes [curve.RistrettoUniformSize]byte
	if err := ExpandMessageXOF(uniformBytes[:], xofFunc, domainSeparator, message); err != nil {
		return nil, fmt.Errorf("h2c: failed to expand message: %w", err)
	}

	var p curve.RistrettoPoint
	return p.SetUniformBytes(uniformBytes[:])
}

func hashToCurve(uniformBytes *[hashToCurveSize]byte) *curve.EdwardsPoint {
	fe0 := uniformToField25519(uniformBytes[:ell])
	fe1 := uniformToField25519(uniformBytes[ell:])

	Q0 := elligator.EdwardsFlavor(fe0)
	Q1 := elligator.EdwardsFlavor(fe1)

	var p curve.EdwardsPoint
	p.Add(Q0, Q1)
	p.MulByCofactor(&p)

	return &p
}

func encodeToCurve(uniformBytes *[encodeToCurveSize]byte) *curve.EdwardsPoint {
	fe := uniformToField25519(uniformBytes[:])
	Q := elligator.EdwardsFlavor(fe)

	var p curve.EdwardsPoint
	p.MulByCofactor(Q)

	return &p
}

func uniformToField25519(b []byte) *field.Element {
	if len(b) != ell {
		panic("h2c: invalid uniform bytes length")
	}

	// Our field decoding routine wants little-endian 512-bit inputs, so
	// reverse the byte-order, and zero-extend.
	bLE := reversedByteSlice(b)
	var bLEExtended [field.ElementWideSize]byte
	copy(bLEExtended[:], bLE) // Zero-extend

	var fe field.Element
	if _, err := fe.SetBytesWide(bLEExtended[:]); err != nil {
		panic("h2c: failed to decode wide field element: " + err.Error())
	}

	return &fe
}

func reversedByteSlice(b []byte) []byte {
	bLen := len(b)
	if bLen == 0 {
		return []byte{}
	}

	out := make([]byte, bLen)
	for i, j := bLen-1, 0; i >= 0; i, j = i-1, j+1 {
		out[j] = b[i]
	}

	return out
}
