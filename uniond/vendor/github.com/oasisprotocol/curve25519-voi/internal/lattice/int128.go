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

package lattice

import (
	"encoding/binary"
	"math/bits"

	"github.com/oasisprotocol/curve25519-voi/curve/scalar"
)

// This is shamelessly stolen from https://github.com/ryanavella/wide
// which is available under the UNLICENSE public domain dedication,
// with the following notable alterations:
//
//  * `IsNeg` renamed to `IsNegative` to match conventions in the rest
//    of the module.
//  * `ToScalar`, `isZero` added.
//  * Methods that do not need to be public, are now private.
//  * Methods altered to use `math/bits` intrinsics.
//
// Note: The consumers of this API and it's outputs do not require
// things to be constant-time.

const (
	int64Size  = 64
	int128Size = 128
)

var (
	i128Zero Int128
	i128One  = Int128{lo: 1}
)

// Int128 is a representation of a signed 128-bit integer.
type Int128 struct {
	hi int64
	lo uint64
}

// ToScalar sets the scalar to the Int128, honoring the sign.
func (x Int128) ToScalar(s *scalar.Scalar) *scalar.Scalar {
	xAbs := x.Abs()

	var b [scalar.ScalarSize]byte
	binary.LittleEndian.PutUint64(b[0:8], xAbs.lo)
	binary.LittleEndian.PutUint64(b[8:16], uint64(xAbs.hi))

	if _, err := s.SetBits(b[:]); err != nil {
		panic("internal/lattice: failed to serialize int128 as scalar: " + err.Error())
	}
	if x.IsNegative() {
		s.Neg(s)
	}

	return s
}

// IsNegative returns whether or not the Int128 is negative.
func (x Int128) IsNegative() bool {
	return x.hi < 0
}

// Abs returns the absolute value of an Int128.
func (x Int128) Abs() Int128 {
	if x.IsNegative() {
		return x.neg()
	}
	return x
}

// isZero returns whether or not the Int128 is zero.
func (x Int128) isZero() bool {
	return x.hi == 0 && x.lo == 0
}

// neg returns the additive inverse of an Int128.
func (x Int128) neg() (z Int128) {
	z = z.sub(x)
	return z
}

// add returns the sum of two Int128's
func (x Int128) add(y Int128) (z Int128) {
	var carryOut uint64
	z.lo, carryOut = bits.Add64(x.lo, y.lo, 0)
	z.hi = x.hi + y.hi + int64(carryOut)
	return z
}

// sub returns the difference of two Int128's
func (x Int128) sub(y Int128) (z Int128) {
	var borrowOut uint64
	z.lo, borrowOut = bits.Sub64(x.lo, y.lo, 0)
	z.hi = x.hi - (y.hi + int64(borrowOut))
	return z
}

// shl returns an Int128 left-shifted by a uint (i.e. x << n).
func (x Int128) shl(n uint) (z Int128) {
	switch {
	case n >= int128Size:
		return z // z.hi, z.lo = 0, 0
	case n >= int64Size:
		z.hi = int64(x.lo << (n - int64Size))
		z.lo = 0
		return z
	default:
		z.hi = int64(uint64(x.hi)<<n | x.lo>>(int64Size-n))
		z.lo = x.lo << n
		return z
	}
}

func newInt128(hi int64, lo uint64) Int128 {
	return Int128{hi: hi, lo: lo}
}

func newInt128FromScalar(s *scalar.Scalar) Int128 {
	var b [scalar.ScalarSize]byte
	if err := s.ToBytes(b[:]); err != nil {
		panic("internal/lattice: failed to serialize scalar as int128: " + err.Error())
	}

	return Int128{
		hi: int64(binary.LittleEndian.Uint64(b[8:16])),
		lo: binary.LittleEndian.Uint64(b[0:8]),
	}
}
