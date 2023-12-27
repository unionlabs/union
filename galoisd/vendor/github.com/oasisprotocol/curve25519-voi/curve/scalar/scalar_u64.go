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

//go:build (amd64 || arm64 || ppc64le || ppc64 || s390x || force64bit) && !force32bit

package scalar

import (
	"encoding/binary"
	"fmt"
	"math/bits"
)

const low_52_bit_mask uint64 = (1 << 52) - 1

// unpackedScalar represents a scalar in Z/lZ as 5 52-bit limbs.
type unpackedScalar [5]uint64

// SetBytes unpacks a 32 byte / 256 bit scalar into 5 52-bit limbs.
func (s *unpackedScalar) SetBytes(in []byte) *unpackedScalar {
	if len(in) != ScalarSize {
		panic("curve/scalar/u64: unexpected input size")
	}

	var words [4]uint64
	for i := 0; i < 4; i++ {
		words[i] = binary.LittleEndian.Uint64(in[i*8:])
	}

	const top_mask uint64 = (1 << 48) - 1

	s[0] = words[0] & low_52_bit_mask
	s[1] = ((words[0] >> 52) | (words[1] << 12)) & low_52_bit_mask
	s[2] = ((words[1] >> 40) | (words[2] << 24)) & low_52_bit_mask
	s[3] = ((words[2] >> 28) | (words[3] << 36)) & low_52_bit_mask
	s[4] = (words[3] >> 16) & top_mask

	return s
}

// SetBytesWide reduces a 64 byte / 512 bit scalar mod l.
func (s *unpackedScalar) SetBytesWide(in []byte) (*unpackedScalar, error) {
	if len(in) != ScalarWideSize {
		return nil, fmt.Errorf("curve/scalar/u64: unexpected wide input size")
	}

	var words [8]uint64
	for i := 0; i < 8; i++ {
		words[i] = binary.LittleEndian.Uint64(in[i*8:])
	}

	var lo, hi unpackedScalar
	lo[0] = words[0] & low_52_bit_mask
	lo[1] = ((words[0] >> 52) | (words[1] << 12)) & low_52_bit_mask
	lo[2] = ((words[1] >> 40) | (words[2] << 24)) & low_52_bit_mask
	lo[3] = ((words[2] >> 28) | (words[3] << 36)) & low_52_bit_mask
	lo[4] = ((words[3] >> 16) | (words[4] << 48)) & low_52_bit_mask
	hi[0] = (words[4] >> 4) & low_52_bit_mask
	hi[1] = ((words[4] >> 56) | (words[5] << 8)) & low_52_bit_mask
	hi[2] = ((words[5] >> 44) | (words[6] << 20)) & low_52_bit_mask
	hi[3] = ((words[6] >> 32) | (words[7] << 32)) & low_52_bit_mask
	hi[4] = words[7] >> 20

	lo.MontgomeryMul(&lo, &constR)  // (lo * R) / R = lo
	hi.MontgomeryMul(&hi, &constRR) // (hi * R^2) / R = hi * R

	// (hi * R) + lo
	return s.Add(&hi, &lo), nil
}

// ToBytes packs the limbs of the scalar into 32 bytes.
func (s *unpackedScalar) ToBytes(out []byte) {
	if len(out) != ScalarSize {
		panic("curve/scalar/u64: unexpected output size")
	}

	out[0] = byte(s[0] >> 0)
	out[1] = byte(s[0] >> 8)
	out[2] = byte(s[0] >> 16)
	out[3] = byte(s[0] >> 24)
	out[4] = byte(s[0] >> 32)
	out[5] = byte(s[0] >> 40)
	out[6] = byte((s[0] >> 48) | (s[1] << 4))
	out[7] = byte(s[1] >> 4)
	out[8] = byte(s[1] >> 12)
	out[9] = byte(s[1] >> 20)
	out[10] = byte(s[1] >> 28)
	out[11] = byte(s[1] >> 36)
	out[12] = byte(s[1] >> 44)
	out[13] = byte(s[2] >> 0)
	out[14] = byte(s[2] >> 8)
	out[15] = byte(s[2] >> 16)
	out[16] = byte(s[2] >> 24)
	out[17] = byte(s[2] >> 32)
	out[18] = byte(s[2] >> 40)
	out[19] = byte((s[2] >> 48) | (s[3] << 4))
	out[20] = byte(s[3] >> 4)
	out[21] = byte(s[3] >> 12)
	out[22] = byte(s[3] >> 20)
	out[23] = byte(s[3] >> 28)
	out[24] = byte(s[3] >> 36)
	out[25] = byte(s[3] >> 44)
	out[26] = byte(s[4] >> 0)
	out[27] = byte(s[4] >> 8)
	out[28] = byte(s[4] >> 16)
	out[29] = byte(s[4] >> 24)
	out[30] = byte(s[4] >> 32)
	out[31] = byte(s[4] >> 40)
}

// Add sets `s = a + b (mod l)`, and returns s.
func (s *unpackedScalar) Add(a, b *unpackedScalar) *unpackedScalar {
	// a + b
	var carry uint64
	for i := 0; i < 5; i++ {
		carry = a[i] + b[i] + (carry >> 52)
		s[i] = carry & low_52_bit_mask
	}

	// subtract l if the sum is >= l
	return s.Sub(s, &constL)
}

// Sub sets `s = a - b (mod l)`, and returns s.
func (s *unpackedScalar) Sub(a, b *unpackedScalar) *unpackedScalar {
	// a - b
	var borrow uint64
	for i := 0; i < 5; i++ {
		borrow = a[i] - (b[i] + (borrow >> 63))
		s[i] = borrow & low_52_bit_mask
	}

	// conditionally add l if the difference is negative
	underflow_mask := ((borrow >> 63) ^ 1) - 1
	var carry uint64
	for i := 0; i < 5; i++ {
		carry = s[i] + (constL[i] & underflow_mask) + (carry >> 52)
		s[i] = carry & low_52_bit_mask
	}

	return s
}

//
// Note: The limbs *[18]uint64 parameter that is passed around by this
// implementation is structured as { l0_lo, l0_hi, ... l8_lo, l8_hi }.
//

// FromMontgomery takes a scalar out of Montgomery form, i.e. computes `a/R (mod l)`.
func (s *unpackedScalar) FromMontgomery(a *unpackedScalar) *unpackedScalar {
	var limbs [18]uint64
	for i := 0; i < 5; i++ {
		limbs[i*2] = a[i]
	}
	return s.MontgomeryReduce(&limbs)
}

// MontgomeryReduce sets `s = limbs/R (mod l)`, where R is the Montgomery
// modulus 2^260, and returns s.
func (s *unpackedScalar) MontgomeryReduce(limbs *[18]uint64) *unpackedScalar {
	part1 := func(sum_hi, sum_lo uint64) (c_hi, c_lo, p uint64) {
		var c uint64
		p = sum_lo * constLFACTOR & ((1 << 52) - 1)
		tmp_hi, tmp_lo := bits.Mul64(p, constL[0])
		c_lo, c = bits.Add64(sum_lo, tmp_lo, 0)
		c_hi, _ = bits.Add64(sum_hi, tmp_hi, c)
		c_lo = (c_hi << (64 - 52)) | (c_lo >> 52)
		c_hi = c_hi >> 52
		return
	}

	part2 := func(sum_hi, sum_lo uint64) (c_hi, c_lo, w uint64) {
		w = sum_lo & ((1 << 52) - 1)
		c_hi, c_lo = sum_hi, sum_lo
		c_lo = (c_hi << (64 - 52)) | (c_lo >> 52)
		c_hi = c_hi >> 52
		return
	}

	// note: l[3] is zero, so its multiples can be skipped
	l := &constL

	// the first half computes the Montgomery adjustment factor n, and begins adding n*l to make limbs divisible by R
	var carry uint64

	c_hi, c_lo, n0 := part1(limbs[1], limbs[0])

	var n1 uint64
	c_lo, carry = bits.Add64(c_lo, limbs[2], 0)
	c_hi, _ = bits.Add64(c_hi, limbs[3], carry)
	t_hi, t_lo := bits.Mul64(n0, l[1])
	c_lo, carry = bits.Add64(c_lo, t_lo, 0)
	c_hi, _ = bits.Add64(c_hi, t_hi, carry)
	c_hi, c_lo, n1 = part1(c_hi, c_lo)

	var n2 uint64
	c_lo, carry = bits.Add64(c_lo, limbs[4], 0)
	c_hi, _ = bits.Add64(c_hi, limbs[5], carry)
	t_hi, t_lo = bits.Mul64(n0, l[2])
	c_lo, carry = bits.Add64(c_lo, t_lo, 0)
	c_hi, _ = bits.Add64(c_hi, t_hi, carry)
	t_hi, t_lo = bits.Mul64(n1, l[1])
	c_lo, carry = bits.Add64(c_lo, t_lo, 0)
	c_hi, _ = bits.Add64(c_hi, t_hi, carry)
	c_hi, c_lo, n2 = part1(c_hi, c_lo)

	var n3 uint64
	c_lo, carry = bits.Add64(c_lo, limbs[6], 0)
	c_hi, _ = bits.Add64(c_hi, limbs[7], carry)
	t_hi, t_lo = bits.Mul64(n1, l[2])
	c_lo, carry = bits.Add64(c_lo, t_lo, 0)
	c_hi, _ = bits.Add64(c_hi, t_hi, carry)
	t_hi, t_lo = bits.Mul64(n2, l[1])
	c_lo, carry = bits.Add64(c_lo, t_lo, 0)
	c_hi, _ = bits.Add64(c_hi, t_hi, carry)
	c_hi, c_lo, n3 = part1(c_hi, c_lo)

	var n4 uint64
	c_lo, carry = bits.Add64(c_lo, limbs[8], 0)
	c_hi, _ = bits.Add64(c_hi, limbs[9], carry)
	t_hi, t_lo = bits.Mul64(n0, l[4])
	c_lo, carry = bits.Add64(c_lo, t_lo, 0)
	c_hi, _ = bits.Add64(c_hi, t_hi, carry)
	t_hi, t_lo = bits.Mul64(n2, l[2])
	c_lo, carry = bits.Add64(c_lo, t_lo, 0)
	c_hi, _ = bits.Add64(c_hi, t_hi, carry)
	t_hi, t_lo = bits.Mul64(n3, l[1])
	c_lo, carry = bits.Add64(c_lo, t_lo, 0)
	c_hi, _ = bits.Add64(c_hi, t_hi, carry)
	c_hi, c_lo, n4 = part1(c_hi, c_lo)

	// limbs is divisible by R now, so we can divide by R by simply storing the upper half as the result

	var r0 uint64
	c_lo, carry = bits.Add64(c_lo, limbs[10], 0)
	c_hi, _ = bits.Add64(c_hi, limbs[11], carry)
	t_hi, t_lo = bits.Mul64(n1, l[4])
	c_lo, carry = bits.Add64(c_lo, t_lo, 0)
	c_hi, _ = bits.Add64(c_hi, t_hi, carry)
	t_hi, t_lo = bits.Mul64(n3, l[2])
	c_lo, carry = bits.Add64(c_lo, t_lo, 0)
	c_hi, _ = bits.Add64(c_hi, t_hi, carry)
	t_hi, t_lo = bits.Mul64(n4, l[1])
	c_lo, carry = bits.Add64(c_lo, t_lo, 0)
	c_hi, _ = bits.Add64(c_hi, t_hi, carry)
	c_hi, c_lo, r0 = part2(c_hi, c_lo)

	var r1 uint64
	c_lo, carry = bits.Add64(c_lo, limbs[12], 0)
	c_hi, _ = bits.Add64(c_hi, limbs[13], carry)
	t_hi, t_lo = bits.Mul64(n2, l[4])
	c_lo, carry = bits.Add64(c_lo, t_lo, 0)
	c_hi, _ = bits.Add64(c_hi, t_hi, carry)
	t_hi, t_lo = bits.Mul64(n4, l[2])
	c_lo, carry = bits.Add64(c_lo, t_lo, 0)
	c_hi, _ = bits.Add64(c_hi, t_hi, carry)
	c_hi, c_lo, r1 = part2(c_hi, c_lo)

	var r2 uint64
	c_lo, carry = bits.Add64(c_lo, limbs[14], 0)
	c_hi, _ = bits.Add64(c_hi, limbs[15], carry)
	t_hi, t_lo = bits.Mul64(n3, l[4])
	c_lo, carry = bits.Add64(c_lo, t_lo, 0)
	c_hi, _ = bits.Add64(c_hi, t_hi, carry)
	c_hi, c_lo, r2 = part2(c_hi, c_lo)

	c_lo, carry = bits.Add64(c_lo, limbs[16], 0)
	c_hi, _ = bits.Add64(c_hi, limbs[17], carry)
	t_hi, t_lo = bits.Mul64(n4, l[4])
	c_lo, carry = bits.Add64(c_lo, t_lo, 0)
	c_hi, _ = bits.Add64(c_hi, t_hi, carry)
	_, r4, r3 := part2(c_hi, c_lo)

	// result may be >= l, so attempt to subtract l
	return s.Sub(&unpackedScalar{r0, r1, r2, r3, r4}, l)
}

func (s *unpackedScalar) squareInternal() [18]uint64 {
	var (
		z     [18]uint64
		carry uint64
	)

	s0, s1, s2, s3, s4 := s[0], s[1], s[2], s[3], s[4]
	aa0, aa1, aa2, aa3 := s0*2, s1*2, s2*2, s3*2

	z[1], z[0] = bits.Mul64(s0, s0)

	z[3], z[2] = bits.Mul64(aa0, s1)

	z_hi, z_lo := bits.Mul64(aa0, s2)
	t_hi, t_lo := bits.Mul64(s1, s1)
	z[4], carry = bits.Add64(z_lo, t_lo, 0)
	z[5], _ = bits.Add64(z_hi, t_hi, carry)

	z_hi, z_lo = bits.Mul64(aa0, s3)
	t_hi, t_lo = bits.Mul64(aa1, s2)
	z[6], carry = bits.Add64(z_lo, t_lo, 0)
	z[7], _ = bits.Add64(z_hi, t_hi, carry)

	z_hi, z_lo = bits.Mul64(aa0, s4)
	t_hi, t_lo = bits.Mul64(aa1, s3)
	z_lo, carry = bits.Add64(z_lo, t_lo, 0)
	z_hi, _ = bits.Add64(z_hi, t_hi, carry)
	t_hi, t_lo = bits.Mul64(s2, s2)
	z[8], carry = bits.Add64(z_lo, t_lo, 0)
	z[9], _ = bits.Add64(z_hi, t_hi, carry)

	z_hi, z_lo = bits.Mul64(aa1, s4)
	t_hi, t_lo = bits.Mul64(aa2, s3)
	z[10], carry = bits.Add64(z_lo, t_lo, 0)
	z[11], _ = bits.Add64(z_hi, t_hi, carry)

	z_hi, z_lo = bits.Mul64(aa2, s4)
	t_hi, t_lo = bits.Mul64(s3, s3)
	z[12], carry = bits.Add64(z_lo, t_lo, 0)
	z[13], _ = bits.Add64(z_hi, t_hi, carry)

	z[15], z[14] = bits.Mul64(aa3, s4)

	z[17], z[16] = bits.Mul64(s4, s4)

	return z
}

// scalarMulInternal computes `a * b`.
func scalarMulInternal(a, b *unpackedScalar) [18]uint64 {
	var (
		z     [18]uint64
		carry uint64
	)

	a0, a1, a2, a3, a4 := a[0], a[1], a[2], a[3], a[4]
	b0, b1, b2, b3, b4 := b[0], b[1], b[2], b[3], b[4]

	z[1], z[0] = bits.Mul64(a0, b0)

	z_hi, z_lo := bits.Mul64(a0, b1)
	t_hi, t_lo := bits.Mul64(a1, b0)
	z[2], carry = bits.Add64(z_lo, t_lo, 0)
	z[3], _ = bits.Add64(z_hi, t_hi, carry)

	z_hi, z_lo = bits.Mul64(a0, b2)
	t_hi, t_lo = bits.Mul64(a1, b1)
	z_lo, carry = bits.Add64(z_lo, t_lo, 0)
	z_hi, _ = bits.Add64(z_hi, t_hi, carry)
	t_hi, t_lo = bits.Mul64(a2, b0)
	z[4], carry = bits.Add64(z_lo, t_lo, 0)
	z[5], _ = bits.Add64(z_hi, t_hi, carry)

	z_hi, z_lo = bits.Mul64(a0, b3)
	t_hi, t_lo = bits.Mul64(a1, b2)
	z_lo, carry = bits.Add64(z_lo, t_lo, 0)
	z_hi, _ = bits.Add64(z_hi, t_hi, carry)
	t_hi, t_lo = bits.Mul64(a2, b1)
	z_lo, carry = bits.Add64(z_lo, t_lo, 0)
	z_hi, _ = bits.Add64(z_hi, t_hi, carry)
	t_hi, t_lo = bits.Mul64(a3, b0)
	z[6], carry = bits.Add64(z_lo, t_lo, 0)
	z[7], _ = bits.Add64(z_hi, t_hi, carry)

	z_hi, z_lo = bits.Mul64(a0, b4)
	t_hi, t_lo = bits.Mul64(a1, b3)
	z_lo, carry = bits.Add64(z_lo, t_lo, 0)
	z_hi, _ = bits.Add64(z_hi, t_hi, carry)
	t_hi, t_lo = bits.Mul64(a2, b2)
	z_lo, carry = bits.Add64(z_lo, t_lo, 0)
	z_hi, _ = bits.Add64(z_hi, t_hi, carry)
	t_hi, t_lo = bits.Mul64(a3, b1)
	z_lo, carry = bits.Add64(z_lo, t_lo, 0)
	z_hi, _ = bits.Add64(z_hi, t_hi, carry)
	t_hi, t_lo = bits.Mul64(a4, b0)
	z[8], carry = bits.Add64(z_lo, t_lo, 0)
	z[9], _ = bits.Add64(z_hi, t_hi, carry)

	z_hi, z_lo = bits.Mul64(a1, b4)
	t_hi, t_lo = bits.Mul64(a2, b3)
	z_lo, carry = bits.Add64(z_lo, t_lo, 0)
	z_hi, _ = bits.Add64(z_hi, t_hi, carry)
	t_hi, t_lo = bits.Mul64(a3, b2)
	z_lo, carry = bits.Add64(z_lo, t_lo, 0)
	z_hi, _ = bits.Add64(z_hi, t_hi, carry)
	t_hi, t_lo = bits.Mul64(a4, b1)
	z[10], carry = bits.Add64(z_lo, t_lo, 0)
	z[11], _ = bits.Add64(z_hi, t_hi, carry)

	z_hi, z_lo = bits.Mul64(a2, b4)
	t_hi, t_lo = bits.Mul64(a3, b3)
	z_lo, carry = bits.Add64(z_lo, t_lo, 0)
	z_hi, _ = bits.Add64(z_hi, t_hi, carry)
	t_hi, t_lo = bits.Mul64(a4, b2)
	z[12], carry = bits.Add64(z_lo, t_lo, 0)
	z[13], _ = bits.Add64(z_hi, t_hi, carry)

	z_hi, z_lo = bits.Mul64(a3, b4)
	t_hi, t_lo = bits.Mul64(a4, b3)
	z[14], carry = bits.Add64(z_lo, t_lo, 0)
	z[15], _ = bits.Add64(z_hi, t_hi, carry)

	z[17], z[16] = bits.Mul64(a4, b4)

	return z
}
