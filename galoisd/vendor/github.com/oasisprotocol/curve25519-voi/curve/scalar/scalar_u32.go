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

//go:build (386 || arm || mips || mipsle || wasm || mips64le || mips64 || riscv64 || loong64 || force32bit) && !force64bit

package scalar

import (
	"encoding/binary"
	"fmt"
)

func m(x, y uint32) uint64 {
	// Note: The paranoid thing to do would be to assume that the Go
	// compiler will fail to do the right thing unless this routine
	// does something like:
	//
	//   hi, lo := bits.Mul32(x, y)
	//   return uint64(hi)<<32 | uint64(lo)
	//
	// due to `bits.Mul32(x, y uint32) (hi, lo uint32)` being
	// guaranteed to be constant time, while normal integer
	// multiplication is not.
	//
	// This is explicitly not done for the following reasons:
	//  * The runtime library just casts and multiplies in the generic
	//    implementation of the `math/bits` call.
	//  * 32x32->64 multiply is constant time on everything that is
	//    actually relevant (386, 486, VIA Nano 2000, ARM 7, ARM 9,
	//    Cortex-M3 are not worth caring about).
	//  * If what this is doing is actually variable-time, then we
	//    are no worse off from the runtime library's old Ed25519
	//    implementation.
	//  * Using the intrinsic utterly kills performance.
	return uint64(x) * uint64(y)
}

const low_29_bit_mask uint32 = (1 << 29) - 1

// unpackedScalar represents a scalar in Z/lZ as 9 29-bit limbs.
type unpackedScalar [9]uint32

// SetBytes unpacks a 32 byte / 256 bit scalar into 9 29-bit limbs.
func (s *unpackedScalar) SetBytes(in []byte) *unpackedScalar {
	if len(in) != ScalarSize {
		panic("curve/scalar/u32: unexpected input size")
	}

	var words [8]uint32
	for i := 0; i < 8; i++ {
		words[i] = binary.LittleEndian.Uint32(in[i*4:])
	}

	const top_mask uint32 = (1 << 24) - 1

	s[0] = words[0] & low_29_bit_mask
	s[1] = ((words[0] >> 29) | (words[1] << 3)) & low_29_bit_mask
	s[2] = ((words[1] >> 26) | (words[2] << 6)) & low_29_bit_mask
	s[3] = ((words[2] >> 23) | (words[3] << 9)) & low_29_bit_mask
	s[4] = ((words[3] >> 20) | (words[4] << 12)) & low_29_bit_mask
	s[5] = ((words[4] >> 17) | (words[5] << 15)) & low_29_bit_mask
	s[6] = ((words[5] >> 14) | (words[6] << 18)) & low_29_bit_mask
	s[7] = ((words[6] >> 11) | (words[7] << 21)) & low_29_bit_mask
	s[8] = (words[7] >> 8) & top_mask

	return s
}

// SetBytesWide reduces a 64 byte / 512 bit scalar mod l.
func (s *unpackedScalar) SetBytesWide(in []byte) (*unpackedScalar, error) {
	if len(in) != ScalarWideSize {
		return nil, fmt.Errorf("curve/scalar/u32: unexpected wide in size")
	}

	var words [16]uint32
	for i := 0; i < 16; i++ {
		words[i] = binary.LittleEndian.Uint32(in[i*4:])
	}

	var lo, hi unpackedScalar
	lo[0] = words[0] & low_29_bit_mask
	lo[1] = ((words[0] >> 29) | (words[1] << 3)) & low_29_bit_mask
	lo[2] = ((words[1] >> 26) | (words[2] << 6)) & low_29_bit_mask
	lo[3] = ((words[2] >> 23) | (words[3] << 9)) & low_29_bit_mask
	lo[4] = ((words[3] >> 20) | (words[4] << 12)) & low_29_bit_mask
	lo[5] = ((words[4] >> 17) | (words[5] << 15)) & low_29_bit_mask
	lo[6] = ((words[5] >> 14) | (words[6] << 18)) & low_29_bit_mask
	lo[7] = ((words[6] >> 11) | (words[7] << 21)) & low_29_bit_mask
	lo[8] = ((words[7] >> 8) | (words[8] << 24)) & low_29_bit_mask
	hi[0] = ((words[8] >> 5) | (words[9] << 27)) & low_29_bit_mask
	hi[1] = (words[9] >> 2) & low_29_bit_mask
	hi[2] = ((words[9] >> 31) | (words[10] << 1)) & low_29_bit_mask
	hi[3] = ((words[10] >> 28) | (words[11] << 4)) & low_29_bit_mask
	hi[4] = ((words[11] >> 25) | (words[12] << 7)) & low_29_bit_mask
	hi[5] = ((words[12] >> 22) | (words[13] << 10)) & low_29_bit_mask
	hi[6] = ((words[13] >> 19) | (words[14] << 13)) & low_29_bit_mask
	hi[7] = ((words[14] >> 16) | (words[15] << 16)) & low_29_bit_mask
	hi[8] = words[15] >> 13

	lo.MontgomeryMul(&lo, &constR)  // (lo * R) / R = lo
	hi.MontgomeryMul(&hi, &constRR) // (hi * R^2) / R = hi * R

	// (hi * R) + lo
	return s.Add(&hi, &lo), nil
}

// ToBytes packs the limbs of the scalar into 32 bytes.
func (s *unpackedScalar) ToBytes(out []byte) {
	if len(out) != ScalarSize {
		panic("curve/scalar/u32: unexpected out size")
	}

	out[0] = byte(s[0] >> 0)
	out[1] = byte(s[0] >> 8)
	out[2] = byte(s[0] >> 16)
	out[3] = byte((s[0] >> 24) | (s[1] << 5))
	out[4] = byte(s[1] >> 3)
	out[5] = byte(s[1] >> 11)
	out[6] = byte(s[1] >> 19)
	out[7] = byte((s[1] >> 27) | (s[2] << 2))
	out[8] = byte(s[2] >> 6)
	out[9] = byte(s[2] >> 14)
	out[10] = byte((s[2] >> 22) | (s[3] << 7))
	out[11] = byte(s[3] >> 1)
	out[12] = byte(s[3] >> 9)
	out[13] = byte(s[3] >> 17)
	out[14] = byte((s[3] >> 25) | (s[4] << 4))
	out[15] = byte(s[4] >> 4)
	out[16] = byte(s[4] >> 12)
	out[17] = byte(s[4] >> 20)
	out[18] = byte((s[4] >> 28) | (s[5] << 1))
	out[19] = byte(s[5] >> 7)
	out[20] = byte(s[5] >> 15)
	out[21] = byte((s[5] >> 23) | (s[6] << 6))
	out[22] = byte(s[6] >> 2)
	out[23] = byte(s[6] >> 10)
	out[24] = byte(s[6] >> 18)
	out[25] = byte((s[6] >> 26) | (s[7] << 3))
	out[26] = byte(s[7] >> 5)
	out[27] = byte(s[7] >> 13)
	out[28] = byte(s[7] >> 21)
	out[29] = byte(s[8] >> 0)
	out[30] = byte(s[8] >> 8)
	out[31] = byte(s[8] >> 16)
}

// Add sets `s = a + b (mod l)`, and returns s.
func (s *unpackedScalar) Add(a, b *unpackedScalar) *unpackedScalar {
	// a + b
	var carry uint32
	for i := 0; i < 9; i++ {
		carry = a[i] + b[i] + (carry >> 29)
		s[i] = carry & low_29_bit_mask
	}

	// subtract l if the sum is >= l
	return s.Sub(s, &constL)
}

// Sub sets `s = a - b (mod l)`, and returns s.
func (s *unpackedScalar) Sub(a, b *unpackedScalar) *unpackedScalar {
	// a - b
	var borrow uint32
	for i := 0; i < 9; i++ {
		borrow = a[i] - (b[i] + (borrow >> 31))
		s[i] = borrow & low_29_bit_mask
	}

	// conditionally add l if the difference is negative
	underflowMask := ((borrow >> 31) ^ 1) - 1
	var carry uint32
	for i := 0; i < 9; i++ {
		carry = (carry >> 29) + s[i] + (constL[i] & underflowMask)
		s[i] = carry & low_29_bit_mask
	}

	return s
}

// FromMontgomery takes a scalar out of Montgomery form, i.e. computes `a/R (mod l)`.
func (s *unpackedScalar) FromMontgomery(a *unpackedScalar) *unpackedScalar {
	var limbs [17]uint64

	for i := 0; i < 9; i++ {
		limbs[i] = uint64(a[i])
	}
	return s.MontgomeryReduce(&limbs)
}

// MontgomeryReduce sets `s = limbs/R (mod l)`, where R is the Montgomery
// modulus 2^261, and returns s.
func (s *unpackedScalar) MontgomeryReduce(limbs *[17]uint64) *unpackedScalar {
	part1 := func(sum uint64) (uint64, uint32) {
		p := uint32(sum) * constLFACTOR & ((1 << 29) - 1)
		return (sum + m(p, constL[0])) >> 29, p
	}

	part2 := func(sum uint64) (uint64, uint32) {
		w := uint32(sum) & ((1 << 29) - 1)
		return sum >> 29, w
	}

	// note: l5,l6,l7 are zero, so their multiplies can be skipped
	l := &constL

	// the first half computes the Montgomery adjustment factor n, and begins adding n*l to make limbs divisible by R
	var (
		carry                              uint64
		n0, n1, n2, n3, n4, n5, n6, n7, n8 uint32
	)
	carry, n0 = part1(limbs[0])
	carry, n1 = part1(carry + limbs[1] + m(n0, l[1]))
	carry, n2 = part1(carry + limbs[2] + m(n0, l[2]) + m(n1, l[1]))
	carry, n3 = part1(carry + limbs[3] + m(n0, l[3]) + m(n1, l[2]) + m(n2, l[1]))
	carry, n4 = part1(carry + limbs[4] + m(n0, l[4]) + m(n1, l[3]) + m(n2, l[2]) + m(n3, l[1]))
	carry, n5 = part1(carry + limbs[5] + m(n1, l[4]) + m(n2, l[3]) + m(n3, l[2]) + m(n4, l[1]))
	carry, n6 = part1(carry + limbs[6] + m(n2, l[4]) + m(n3, l[3]) + m(n4, l[2]) + m(n5, l[1]))
	carry, n7 = part1(carry + limbs[7] + m(n3, l[4]) + m(n4, l[3]) + m(n5, l[2]) + m(n6, l[1]))
	carry, n8 = part1(carry + limbs[8] + m(n0, l[8]) + m(n4, l[4]) + m(n5, l[3]) + m(n6, l[2]) + m(n7, l[1]))

	// limbs is divisible by R now, so we can divide by R by simply storing the upper half as the result
	var r0, r1, r2, r3, r4, r5, r6, r7, r8 uint32
	carry, r0 = part2(carry + limbs[9] + m(n1, l[8]) + m(n5, l[4]) + m(n6, l[3]) + m(n7, l[2]) + m(n8, l[1]))
	carry, r1 = part2(carry + limbs[10] + m(n2, l[8]) + m(n6, l[4]) + m(n7, l[3]) + m(n8, l[2]))
	carry, r2 = part2(carry + limbs[11] + m(n3, l[8]) + m(n7, l[4]) + m(n8, l[3]))
	carry, r3 = part2(carry + limbs[12] + m(n4, l[8]) + m(n8, l[4]))
	carry, r4 = part2(carry + limbs[13] + m(n5, l[8]))
	carry, r5 = part2(carry + limbs[14] + m(n6, l[8]))
	carry, r6 = part2(carry + limbs[15] + m(n7, l[8]))
	carry, r7 = part2(carry + limbs[16] + m(n8, l[8]))
	r8 = uint32(carry)

	return s.Sub(&unpackedScalar{r0, r1, r2, r3, r4, r5, r6, r7, r8}, l)
}

func (s *unpackedScalar) squareInternal() [17]uint64 {
	var z [17]uint64

	s0, s1, s2, s3, s4, s5, s6, s7, s8 := s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7], s[8]
	aa0, aa1, aa2, aa3, aa4, aa5, aa6, aa7 := s0*2, s1*2, s2*2, s3*2, s4*2, s5*2, s6*2, s7*2

	z[0], z[1], z[2], z[3], z[4], z[5], z[6], z[7], z[8], z[9], z[10], z[11], z[12], z[13], z[14], z[15], z[16] = m(s0, s0),
		m(aa0, s1),
		m(aa0, s2)+m(s1, s1),
		m(aa0, s3)+m(aa1, s2),
		m(aa0, s4)+m(aa1, s3)+m(s2, s2),
		m(aa0, s5)+m(aa1, s4)+m(aa2, s3),
		m(aa0, s6)+m(aa1, s5)+m(aa2, s4)+m(s3, s3),
		m(aa0, s7)+m(aa1, s6)+m(aa2, s5)+m(aa3, s4),
		m(aa0, s8)+m(aa1, s7)+m(aa2, s6)+m(aa3, s5)+m(s4, s4),
		m(aa1, s8)+m(aa2, s7)+m(aa3, s6)+m(aa4, s5),
		m(aa2, s8)+m(aa3, s7)+m(aa4, s6)+m(s5, s5),
		m(aa3, s8)+m(aa4, s7)+m(aa5, s6),
		m(aa4, s8)+m(aa5, s7)+m(s6, s6),
		m(aa5, s8)+m(aa6, s7),
		m(aa6, s8)+m(s7, s7),
		m(aa7, s8),
		m(s8, s8)

	return z
}

// scalarMulInternal computes `a * b`.
func scalarMulInternal(a, b *unpackedScalar) [17]uint64 {
	var z [17]uint64

	z[0] = m(a[0], b[0])                                                                 // c00
	z[1] = m(a[0], b[1]) + m(a[1], b[0])                                                 // c01
	z[2] = m(a[0], b[2]) + m(a[1], b[1]) + m(a[2], b[0])                                 // c02
	z[3] = m(a[0], b[3]) + m(a[1], b[2]) + m(a[2], b[1]) + m(a[3], b[0])                 // c03
	z[4] = m(a[0], b[4]) + m(a[1], b[3]) + m(a[2], b[2]) + m(a[3], b[1]) + m(a[4], b[0]) // c04
	z[5] = m(a[1], b[4]) + m(a[2], b[3]) + m(a[3], b[2]) + m(a[4], b[1])                 // c05
	z[6] = m(a[2], b[4]) + m(a[3], b[3]) + m(a[4], b[2])                                 // c06
	z[7] = m(a[3], b[4]) + m(a[4], b[3])                                                 // c07
	z[8] = m(a[4], b[4]) - z[3]                                                          // c08 - c03

	z[10] = z[5] - m(a[5], b[5])                                          // c05mc10
	z[11] = z[6] - (m(a[5], b[6]) + m(a[6], b[5]))                        // c06mc11
	z[12] = z[7] - (m(a[5], b[7]) + m(a[6], b[6]) + m(a[7], b[5]))        // c07mc12
	z[13] = m(a[5], b[8]) + m(a[6], b[7]) + m(a[7], b[6]) + m(a[8], b[5]) // c13
	z[14] = m(a[6], b[8]) + m(a[7], b[7]) + m(a[8], b[6])                 // c14
	z[15] = m(a[7], b[8]) + m(a[8], b[7])                                 // c15
	z[16] = m(a[8], b[8])                                                 // c16

	z[5] = z[10] - z[0]   // c05mc10 - c00
	z[6] = z[11] - z[1]   // c06mc11 - c01
	z[7] = z[12] - z[2]   // c07mc12 - c02
	z[8] = z[8] - z[13]   // c08mc13 - c03
	z[9] = z[14] + z[4]   // c14 + c04
	z[10] = z[15] + z[10] // c15 + c05mc10
	z[11] = z[16] + z[11] // c16 + c06mc11

	aa := [4]uint32{
		a[0] + a[5],
		a[1] + a[6],
		a[2] + a[7],
		a[3] + a[8],
	}
	bb := [4]uint32{
		b[0] + b[5],
		b[1] + b[6],
		b[2] + b[7],
		b[3] + b[8],
	}

	z[5] = m(aa[0], bb[0]) + z[5]                                                                         // c20 + c05mc10 - c00
	z[6] = (m(aa[0], bb[1]) + m(aa[1], bb[0])) + z[6]                                                     // c21 + c06mc11 - c01
	z[7] = (m(aa[0], bb[2]) + m(aa[1], bb[1]) + m(aa[2], bb[0])) + z[7]                                   // c22 + c07mc12 - c02
	z[8] = (m(aa[0], bb[3]) + m(aa[1], bb[2]) + m(aa[2], bb[1]) + m(aa[3], bb[0])) + z[8]                 // c23 + c08mc13 - c03
	z[9] = (m(aa[0], b[4]) + m(aa[1], bb[3]) + m(aa[2], bb[2]) + m(aa[3], bb[1]) + m(a[4], bb[0])) - z[9] // c24 - c14 - c04
	z[10] = (m(aa[1], b[4]) + m(aa[2], bb[3]) + m(aa[3], bb[2]) + m(a[4], bb[1])) - z[10]                 // c25 - c15 - c05mc10
	z[11] = (m(aa[2], b[4]) + m(aa[3], bb[3]) + m(a[4], bb[2])) - z[11]                                   // c26 - c16 - c06mc11
	z[12] = (m(aa[3], b[4]) + m(a[4], bb[3])) - z[12]                                                     // c27 - c07mc12

	return z
}
