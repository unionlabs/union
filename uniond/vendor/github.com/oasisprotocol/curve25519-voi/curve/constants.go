// Copyright (c) 2016-2019 isis agora lovecruft. All rights reserved.
// Copyright (c) 2016-2019 Henry de Valence. All rights reserved.
// Copyright (c) 2020-2021 Oasis Labs Inc. All rights reserved.
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

package curve

import "github.com/oasisprotocol/curve25519-voi/internal/field"

const (
	// CompressedPointSize is the size of a compressed point in bytes.
	CompressedPointSize = 32

	// MontgomeryPointSize is the size of the u-coordinate of a point on
	// the Montgomery form in bytes.
	MontgomeryPointSize = 32

	// RistrettoUniformSize is the size of the uniformly random bytes
	// required to construct a random Ristretto point.
	RistrettoUniformSize = 64
)

var (
	// ED25519_BASEPOINT_COMPRESSED is the Ed25519 basepoint, in
	// CompressedEdwardsY format.
	ED25519_BASEPOINT_COMPRESSED = &CompressedEdwardsY{
		0x58, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
		0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
		0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
		0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
	}

	// X25519_BASEPOINT is the X25519 basepoint, in MontgomeryPoint
	// format.
	X25519_BASEPOINT = &MontgomeryPoint{
		0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
		0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
		0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
		0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	}

	// RISTRETTO_BASEPOINT_COMPRESED is the Ristretto basepoint, in
	// CompressedRistretto format.
	RISTRETTO_BASEPOINT_COMPRESSED = &CompressedRistretto{
		0xe2, 0xf2, 0xae, 0x0a, 0x6a, 0xbc, 0x4e, 0x71,
		0xa8, 0x84, 0xa9, 0x61, 0xc5, 0x00, 0x51, 0x5f,
		0x58, 0xe3, 0x0b, 0x6a, 0xa5, 0x82, 0xdd, 0x8d,
		0xb6, 0xa6, 0x59, 0x45, 0xe0, 0x8d, 0x2d, 0x76,
	}

	// RISTRETTO_BASEPOINT_POINT is the Ristretto basepoint, in
	// RistrettoPoint format.
	RISTRETTO_BASEPOINT_POINT = &RistrettoPoint{
		inner: *ED25519_BASEPOINT_POINT,
	}

	// RISTRETTO_BASEPOINT_TABLE is the Ristretto basepoint, as a
	// RistrettoBasepointTable for scalar multiplication.
	RISTRETTO_BASEPOINT_TABLE = &RistrettoBasepointTable{
		inner: *ED25519_BASEPOINT_TABLE,
	}
)

func newEdwardsPoint(X, Y, Z, T field.Element) *EdwardsPoint {
	return &EdwardsPoint{
		edwardsPointInner{
			X: X,
			Y: Y,
			Z: Z,
			T: T,
		},
	}
}
