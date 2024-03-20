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

package field

import (
	"fmt"

	"github.com/oasisprotocol/curve25519-voi/internal/disalloweq"
	"github.com/oasisprotocol/curve25519-voi/internal/subtle"
)

func m(x, y uint32) uint64 {
	// See the comment in curve/scalar/scalar_u32.go as to why this
	// does not use `bits.Mul32`.
	return uint64(x) * uint64(y)
}

// Element represents an element of the field Z/(2^255 - 19).
type Element struct {
	disalloweq.DisallowEqual //nolint:unused
	inner                    [10]uint32
}

// Add sets `fe = a + b`, and returns fe.
func (fe *Element) Add(a, b *Element) *Element {
	fe.inner[0] = a.inner[0] + b.inner[0]
	fe.inner[1] = a.inner[1] + b.inner[1]
	fe.inner[2] = a.inner[2] + b.inner[2]
	fe.inner[3] = a.inner[3] + b.inner[3]
	fe.inner[4] = a.inner[4] + b.inner[4]
	fe.inner[5] = a.inner[5] + b.inner[5]
	fe.inner[6] = a.inner[6] + b.inner[6]
	fe.inner[7] = a.inner[7] + b.inner[7]
	fe.inner[8] = a.inner[8] + b.inner[8]
	fe.inner[9] = a.inner[9] + b.inner[9]

	return fe
}

// Sub sets `fe = a - b`, and returns fe.
func (fe *Element) Sub(a, b *Element) *Element {
	// Compute a - b as ((a + 2^4 * p) - b) to avoid underflow.
	return fe.reduce(&[10]uint64{
		uint64((a.inner[0] + (0x3ffffed << 4)) - b.inner[0]),
		uint64((a.inner[1] + (0x1ffffff << 4)) - b.inner[1]),
		uint64((a.inner[2] + (0x3ffffff << 4)) - b.inner[2]),
		uint64((a.inner[3] + (0x1ffffff << 4)) - b.inner[3]),
		uint64((a.inner[4] + (0x3ffffff << 4)) - b.inner[4]),
		uint64((a.inner[5] + (0x1ffffff << 4)) - b.inner[5]),
		uint64((a.inner[6] + (0x3ffffff << 4)) - b.inner[6]),
		uint64((a.inner[7] + (0x1ffffff << 4)) - b.inner[7]),
		uint64((a.inner[8] + (0x3ffffff << 4)) - b.inner[8]),
		uint64((a.inner[9] + (0x1ffffff << 4)) - b.inner[9]),
	})
}

// Mul sets `fe = a * b`, and returns fe.
func (fe *Element) Mul(a, b *Element) *Element {
	x, y := a.inner, b.inner

	// We assume that the input limbs x[i], y[i] are bounded by:
	//
	// x[i], y[i] < 2^(26 + b) if i even
	// x[i], y[i] < 2^(25 + b) if i odd
	//
	// where b is a (real) parameter representing the excess bits of
	// the limbs.  We track the bitsizes of all variables through
	// the computation and solve at the end for the allowable
	// headroom bitsize b (which determines how many additions we
	// can perform between reductions or multiplications).

	y1_19 := 19 * y[1] // This fits in a u32
	y2_19 := 19 * y[2] // iff 26 + b + lg(19) < 32
	y3_19 := 19 * y[3] // if  b < 32 - 26 - 4.248 = 1.752
	y4_19 := 19 * y[4]
	y5_19 := 19 * y[5] // below, b<2.5: this is a bottleneck,
	y6_19 := 19 * y[6] // could be avoided by promoting to
	y7_19 := 19 * y[7] // u64 here instead of in m()
	y8_19 := 19 * y[8]
	y9_19 := 19 * y[9]

	// What happens when we multiply x[i] with y[j] and place the
	// result into the (i+j)-th limb?
	//
	// x[i]      represents the value x[i]*2^ceil(i*51/2)
	// y[j]      represents the value y[j]*2^ceil(j*51/2)
	// z[i+j]    represents the value z[i+j]*2^ceil((i+j)*51/2)
	// x[i]*y[j] represents the value x[i]*y[i]*2^(ceil(i*51/2)+ceil(j*51/2))
	//
	// Since the radix is already accounted for, the result placed
	// into the (i+j)-th limb should be
	//
	// x[i]*y[i]*2^(ceil(i*51/2)+ceil(j*51/2) - ceil((i+j)*51/2)).
	//
	// The value of ceil(i*51/2)+ceil(j*51/2) - ceil((i+j)*51/2) is
	// 1 when both i and j are odd, and 0 otherwise.  So we add
	//
	//   x[i]*y[j] if either i or j is even
	// 2*x[i]*y[j] if i and j are both odd
	//
	// by using precomputed multiples of x[i] for odd i:

	x1_2 := 2 * x[1] // This fits in a u32 iff 25 + b + 1 < 32
	x3_2 := 2 * x[3] //                    iff b < 6
	x5_2 := 2 * x[5]
	x7_2 := 2 * x[7]
	x9_2 := 2 * x[9]

	z0 := m(x[0], y[0]) + m(x1_2, y9_19) + m(x[2], y8_19) + m(x3_2, y7_19) + m(x[4], y6_19) + m(x5_2, y5_19) + m(x[6], y4_19) + m(x7_2, y3_19) + m(x[8], y2_19) + m(x9_2, y1_19)
	z1 := m(x[0], y[1]) + m(x[1], y[0]) + m(x[2], y9_19) + m(x[3], y8_19) + m(x[4], y7_19) + m(x[5], y6_19) + m(x[6], y5_19) + m(x[7], y4_19) + m(x[8], y3_19) + m(x[9], y2_19)
	z2 := m(x[0], y[2]) + m(x1_2, y[1]) + m(x[2], y[0]) + m(x3_2, y9_19) + m(x[4], y8_19) + m(x5_2, y7_19) + m(x[6], y6_19) + m(x7_2, y5_19) + m(x[8], y4_19) + m(x9_2, y3_19)
	z3 := m(x[0], y[3]) + m(x[1], y[2]) + m(x[2], y[1]) + m(x[3], y[0]) + m(x[4], y9_19) + m(x[5], y8_19) + m(x[6], y7_19) + m(x[7], y6_19) + m(x[8], y5_19) + m(x[9], y4_19)
	z4 := m(x[0], y[4]) + m(x1_2, y[3]) + m(x[2], y[2]) + m(x3_2, y[1]) + m(x[4], y[0]) + m(x5_2, y9_19) + m(x[6], y8_19) + m(x7_2, y7_19) + m(x[8], y6_19) + m(x9_2, y5_19)
	z5 := m(x[0], y[5]) + m(x[1], y[4]) + m(x[2], y[3]) + m(x[3], y[2]) + m(x[4], y[1]) + m(x[5], y[0]) + m(x[6], y9_19) + m(x[7], y8_19) + m(x[8], y7_19) + m(x[9], y6_19)
	z6 := m(x[0], y[6]) + m(x1_2, y[5]) + m(x[2], y[4]) + m(x3_2, y[3]) + m(x[4], y[2]) + m(x5_2, y[1]) + m(x[6], y[0]) + m(x7_2, y9_19) + m(x[8], y8_19) + m(x9_2, y7_19)
	z7 := m(x[0], y[7]) + m(x[1], y[6]) + m(x[2], y[5]) + m(x[3], y[4]) + m(x[4], y[3]) + m(x[5], y[2]) + m(x[6], y[1]) + m(x[7], y[0]) + m(x[8], y9_19) + m(x[9], y8_19)
	z8 := m(x[0], y[8]) + m(x1_2, y[7]) + m(x[2], y[6]) + m(x3_2, y[5]) + m(x[4], y[4]) + m(x5_2, y[3]) + m(x[6], y[2]) + m(x7_2, y[1]) + m(x[8], y[0]) + m(x9_2, y9_19)
	z9 := m(x[0], y[9]) + m(x[1], y[8]) + m(x[2], y[7]) + m(x[3], y[6]) + m(x[4], y[5]) + m(x[5], y[4]) + m(x[6], y[3]) + m(x[7], y[2]) + m(x[8], y[1]) + m(x[9], y[0])

	return fe.reduce(&[10]uint64{z0, z1, z2, z3, z4, z5, z6, z7, z8, z9})
}

// Mul121666 sets `fe = t * 121666`, and returns fe.
func (fe *Element) Mul121666(t *Element) *Element {
	return fe.Mul(t, &constAPLUS2_OVER_FOUR)
}

// Neg sets `fe = -t`, and returns fe.
func (fe *Element) Neg(t *Element) *Element {
	// Compute -b as ((2^4 * p) - b) to avoid underflow.
	return fe.reduce(&[10]uint64{
		uint64((0x3ffffed << 4) - t.inner[0]),
		uint64((0x1ffffff << 4) - t.inner[1]),
		uint64((0x3ffffff << 4) - t.inner[2]),
		uint64((0x1ffffff << 4) - t.inner[3]),
		uint64((0x3ffffff << 4) - t.inner[4]),
		uint64((0x1ffffff << 4) - t.inner[5]),
		uint64((0x3ffffff << 4) - t.inner[6]),
		uint64((0x1ffffff << 4) - t.inner[7]),
		uint64((0x3ffffff << 4) - t.inner[8]),
		uint64((0x1ffffff << 4) - t.inner[9]),
	})
}

// ConditionalSelect sets the field element to a iff choice == 0 and
// b iff choice == 1.
func (fe *Element) ConditionalSelect(a, b *Element, choice int) {
	fe.inner[0] = subtle.ConstantTimeSelectUint32(choice, b.inner[0], a.inner[0])
	fe.inner[1] = subtle.ConstantTimeSelectUint32(choice, b.inner[1], a.inner[1])
	fe.inner[2] = subtle.ConstantTimeSelectUint32(choice, b.inner[2], a.inner[2])
	fe.inner[3] = subtle.ConstantTimeSelectUint32(choice, b.inner[3], a.inner[3])
	fe.inner[4] = subtle.ConstantTimeSelectUint32(choice, b.inner[4], a.inner[4])
	fe.inner[5] = subtle.ConstantTimeSelectUint32(choice, b.inner[5], a.inner[5])
	fe.inner[6] = subtle.ConstantTimeSelectUint32(choice, b.inner[6], a.inner[6])
	fe.inner[7] = subtle.ConstantTimeSelectUint32(choice, b.inner[7], a.inner[7])
	fe.inner[8] = subtle.ConstantTimeSelectUint32(choice, b.inner[8], a.inner[8])
	fe.inner[9] = subtle.ConstantTimeSelectUint32(choice, b.inner[9], a.inner[9])
}

// ConditionalSwap conditionally swaps the field elements according to choice.
func (fe *Element) ConditionalSwap(other *Element, choice int) {
	subtle.ConstantTimeSwapUint32(choice, &other.inner[0], &fe.inner[0])
	subtle.ConstantTimeSwapUint32(choice, &other.inner[1], &fe.inner[1])
	subtle.ConstantTimeSwapUint32(choice, &other.inner[2], &fe.inner[2])
	subtle.ConstantTimeSwapUint32(choice, &other.inner[3], &fe.inner[3])
	subtle.ConstantTimeSwapUint32(choice, &other.inner[4], &fe.inner[4])
	subtle.ConstantTimeSwapUint32(choice, &other.inner[5], &fe.inner[5])
	subtle.ConstantTimeSwapUint32(choice, &other.inner[6], &fe.inner[6])
	subtle.ConstantTimeSwapUint32(choice, &other.inner[7], &fe.inner[7])
	subtle.ConstantTimeSwapUint32(choice, &other.inner[8], &fe.inner[8])
	subtle.ConstantTimeSwapUint32(choice, &other.inner[9], &fe.inner[9])
}

// ConditionalAssign conditionally assigns the field element according to choice.
func (fe *Element) ConditionalAssign(other *Element, choice int) {
	fe.inner[0] = subtle.ConstantTimeSelectUint32(choice, other.inner[0], fe.inner[0])
	fe.inner[1] = subtle.ConstantTimeSelectUint32(choice, other.inner[1], fe.inner[1])
	fe.inner[2] = subtle.ConstantTimeSelectUint32(choice, other.inner[2], fe.inner[2])
	fe.inner[3] = subtle.ConstantTimeSelectUint32(choice, other.inner[3], fe.inner[3])
	fe.inner[4] = subtle.ConstantTimeSelectUint32(choice, other.inner[4], fe.inner[4])
	fe.inner[5] = subtle.ConstantTimeSelectUint32(choice, other.inner[5], fe.inner[5])
	fe.inner[6] = subtle.ConstantTimeSelectUint32(choice, other.inner[6], fe.inner[6])
	fe.inner[7] = subtle.ConstantTimeSelectUint32(choice, other.inner[7], fe.inner[7])
	fe.inner[8] = subtle.ConstantTimeSelectUint32(choice, other.inner[8], fe.inner[8])
	fe.inner[9] = subtle.ConstantTimeSelectUint32(choice, other.inner[9], fe.inner[9])
}

// One sets fe to one, and returns fe.
func (fe *Element) One() *Element {
	*fe = NewElement2625(1, 0, 0, 0, 0, 0, 0, 0, 0, 0)
	return fe
}

// MinusOne sets fe to -1, and returns fe.
func (fe *Element) MinusOne() *Element {
	*fe = NewElement2625(
		0x3ffffec, 0x1ffffff, 0x3ffffff, 0x1ffffff, 0x3ffffff,
		0x1ffffff, 0x3ffffff, 0x1ffffff, 0x3ffffff, 0x1ffffff,
	)
	return fe
}

func (fe *Element) reduce(z *[10]uint64) *Element {
	const (
		low_25_bit_mask uint64 = (1 << 25) - 1
		low_26_bit_mask uint64 = (1 << 26) - 1
	)

	carry := func(z *[10]uint64, i uint) {
		switch i & 1 {
		case 0:
			// Even limbs have 26 bits.
			z[i+1] += z[i] >> 26
			z[i] &= low_26_bit_mask
		case 1:
			// Odd limbs have 25 bits.
			z[i+1] += z[i] >> 25
			z[i] &= low_25_bit_mask
		}
	}

	// Perform two halves of the carry chain in parallel.
	carry(z, 0)
	carry(z, 4)
	carry(z, 1)
	carry(z, 5)
	carry(z, 2)
	carry(z, 6)
	carry(z, 3)
	carry(z, 7)
	// Since z[3] < 2^64, c < 2^(64-25) = 2^39,
	// so    z[4] < 2^26 + 2^39 < 2^39.0002
	carry(z, 4)
	carry(z, 8)
	// Now z[4] < 2^26
	// and z[5] < 2^25 + 2^13.0002 < 2^25.0004 (good enough)

	// Last carry has a multiplication by 19:
	z[0] += 19 * (z[9] >> 25)
	z[9] &= low_25_bit_mask

	// Since z[9] < 2^64, c < 2^(64-25) = 2^39,
	//    so z[0] + 19*c < 2^26 + 2^43.248 < 2^43.249.
	carry(z, 0)
	// Now z[1] < 2^25 - 2^(43.249 - 26)
	//          < 2^25.007 (good enough)
	// and we're done.

	fe.inner[0] = uint32(z[0])
	fe.inner[1] = uint32(z[1])
	fe.inner[2] = uint32(z[2])
	fe.inner[3] = uint32(z[3])
	fe.inner[4] = uint32(z[4])
	fe.inner[5] = uint32(z[5])
	fe.inner[6] = uint32(z[6])
	fe.inner[7] = uint32(z[7])
	fe.inner[8] = uint32(z[8])
	fe.inner[9] = uint32(z[9])

	return fe
}

// SetBytes loads a field element from the low 255-bits of a 256-bit input.
//
// WARNING: This function does not check that the input used the canonical
// representative.  It masks the high bit, but it will happily decode
// 2^255 - 18 to 1.  Applications that require a canonical encoding of
// every field element should decode, re-encode to the canonical encoding,
// and check that the input was canonical.
func (fe *Element) SetBytes(in []byte) (*Element, error) {
	if len(in) != ElementSize {
		return nil, fmt.Errorf("internal/field/u32: unexpected in size")
	}

	load3 := func(b []byte) uint64 {
		return uint64(b[0]) | (uint64(b[1]) << 8) | (uint64(b[2]) << 16)
	}
	load4 := func(b []byte) uint64 {
		return uint64(b[0]) | (uint64(b[1]) << 8) | (uint64(b[2]) << 16) | (uint64(b[3]) << 24)
	}

	var h [10]uint64
	const low_23_bit_mask uint64 = (1 << 23) - 1
	h[0] = load4(in[0:4])
	h[1] = load3(in[4:7]) << 6
	h[2] = load3(in[7:10]) << 5
	h[3] = load3(in[10:13]) << 3
	h[4] = load3(in[13:16]) << 2
	h[5] = load4(in[16:20])
	h[6] = load3(in[20:23]) << 7
	h[7] = load3(in[23:26]) << 5
	h[8] = load3(in[26:29]) << 4
	h[9] = (load3(in[29:32]) & low_23_bit_mask) << 2

	fe.reduce(&h)

	return fe, nil
}

// SetBytesWide loads a field element from a 512-bit little-endian input.
func (fe *Element) SetBytesWide(in []byte) (*Element, error) {
	if len(in) != ElementWideSize {
		return nil, fmt.Errorf("internal/field/u32: unexpected input size")
	}

	var lo, hi Element
	if _, err := lo.SetBytes(in[:ElementSize]); err != nil {
		return nil, fmt.Errorf("internal/field/u32: failed to deserialize lo: %w", err)
	}
	if _, err := hi.SetBytes(in[ElementSize:]); err != nil {
		return nil, fmt.Errorf("internal/field/u32: failed to deserialize hi: %w", err)
	}

	// Handle the 256th and 512th bits (MSB of lo and hi) explicitly
	// as SetBytes ignores them.
	lo.inner[0] += uint32(in[31]>>7)*19 + uint32(in[63]>>7)*2*19*19

	// Back-of-the-envelope math says that this will fit without widening,
	// but might as well do it this way.
	return fe.reduce(&[10]uint64{
		uint64(lo.inner[0]) + 2*19*uint64(hi.inner[0]),
		uint64(lo.inner[1]) + 2*19*uint64(hi.inner[1]),
		uint64(lo.inner[2]) + 2*19*uint64(hi.inner[2]),
		uint64(lo.inner[3]) + 2*19*uint64(hi.inner[3]),
		uint64(lo.inner[4]) + 2*19*uint64(hi.inner[4]),
		uint64(lo.inner[5]) + 2*19*uint64(hi.inner[5]),
		uint64(lo.inner[6]) + 2*19*uint64(hi.inner[6]),
		uint64(lo.inner[7]) + 2*19*uint64(hi.inner[7]),
		uint64(lo.inner[8]) + 2*19*uint64(hi.inner[8]),
		uint64(lo.inner[9]) + 2*19*uint64(hi.inner[9]),
	}), nil
}

// ToBytes packs the field element into 32 bytes.  The encoding is canonical.
func (fe *Element) ToBytes(out []byte) error {
	if len(out) != ElementSize {
		return fmt.Errorf("internal/field/u32: unexpected output size")
	}

	// Reduce the value represented by `fe` to the range [0,2*p)
	var reduced Element
	reduced.reduce(&[10]uint64{
		uint64(fe.inner[0]), uint64(fe.inner[1]), uint64(fe.inner[2]), uint64(fe.inner[3]), uint64(fe.inner[4]),
		uint64(fe.inner[5]), uint64(fe.inner[6]), uint64(fe.inner[7]), uint64(fe.inner[8]), uint64(fe.inner[9]),
	})

	h0, h1, h2, h3, h4, h5, h6, h7, h8, h9 := reduced.inner[0], reduced.inner[1], reduced.inner[2], reduced.inner[3], reduced.inner[4], reduced.inner[5], reduced.inner[6], reduced.inner[7], reduced.inner[8], reduced.inner[9]

	// Let h be the value to encode.
	//
	// Write h = pq + r with 0 <= r < p.  We want to compute r = h mod p.
	//
	// Since h < 2*p, q = 0 or 1, with q = 0 when h < p and q = 1 when h >= p.
	//
	// Notice that h >= p <==> h + 19 >= p + 19 <==> h + 19 >= 2^255.
	// Therefore q can be computed as the carry bit of h + 19.

	q := (h0 + 19) >> 26
	q = (h1 + q) >> 25
	q = (h2 + q) >> 26
	q = (h3 + q) >> 25
	q = (h4 + q) >> 26
	q = (h5 + q) >> 25
	q = (h6 + q) >> 26
	q = (h7 + q) >> 25
	q = (h8 + q) >> 26
	q = (h9 + q) >> 25

	// Now we can compute r as r = h - pq = r - (2^255-19)q = r + 19q - 2^255q

	const (
		low_25_bit_mask uint32 = (1 << 25) - 1
		low_26_bit_mask uint32 = (1 << 26) - 1
	)

	h0 += 19 * q

	// Now carry the result to compute r + 19q...
	h1 += h0 >> 26
	h0 = h0 & low_26_bit_mask
	h2 += h1 >> 25
	h1 = h1 & low_25_bit_mask
	h3 += h2 >> 26
	h2 = h2 & low_26_bit_mask
	h4 += h3 >> 25
	h3 = h3 & low_25_bit_mask
	h5 += h4 >> 26
	h4 = h4 & low_26_bit_mask
	h6 += h5 >> 25
	h5 = h5 & low_25_bit_mask
	h7 += h6 >> 26
	h6 = h6 & low_26_bit_mask
	h8 += h7 >> 25
	h7 = h7 & low_25_bit_mask
	h9 += h8 >> 26
	h8 = h8 & low_26_bit_mask

	// ... but instead of carrying the value
	// (h9 >> 25) = q*2^255 into another limb,
	// discard it, subtracting the value from h.
	h9 = h9 & low_25_bit_mask

	out[0] = byte(h0 >> 0)
	out[1] = byte(h0 >> 8)
	out[2] = byte(h0 >> 16)
	out[3] = byte((h0 >> 24) | (h1 << 2))
	out[4] = byte(h1 >> 6)
	out[5] = byte(h1 >> 14)
	out[6] = byte((h1 >> 22) | (h2 << 3))
	out[7] = byte(h2 >> 5)
	out[8] = byte(h2 >> 13)
	out[9] = byte((h2 >> 21) | (h3 << 5))
	out[10] = byte(h3 >> 3)
	out[11] = byte(h3 >> 11)
	out[12] = byte((h3 >> 19) | (h4 << 6))
	out[13] = byte(h4 >> 2)
	out[14] = byte(h4 >> 10)
	out[15] = byte(h4 >> 18)
	out[16] = byte(h5 >> 0)
	out[17] = byte(h5 >> 8)
	out[18] = byte(h5 >> 16)
	out[19] = byte((h5 >> 24) | (h6 << 1))
	out[20] = byte(h6 >> 7)
	out[21] = byte(h6 >> 15)
	out[22] = byte((h6 >> 23) | (h7 << 3))
	out[23] = byte(h7 >> 5)
	out[24] = byte(h7 >> 13)
	out[25] = byte((h7 >> 21) | (h8 << 4))
	out[26] = byte(h8 >> 4)
	out[27] = byte(h8 >> 12)
	out[28] = byte((h8 >> 20) | (h9 << 6))
	out[29] = byte(h9 >> 2)
	out[30] = byte(h9 >> 10)
	out[31] = byte(h9 >> 18)

	return nil
}

// Pow2k sets `fe = t^(2^k)`, given `k > 0`, and returns fe.
func (fe *Element) Pow2k(t *Element, k uint) *Element {
	if k == 0 {
		panic("internal/field/u32: k out of bounds")
	}

	var z [10]uint64

	// Handle the first squaring separately to save a copy.
	squareInner(&t.inner, &z)
	fe.reduce(&z)

	// And do the rest.
	for ; k > 1; k-- {
		squareInner(&fe.inner, &z)
		fe.reduce(&z)
	}

	return fe
}

// Square sets `fe = t^2`, and returns fe.
func (fe *Element) Square(t *Element) *Element {
	var z [10]uint64
	squareInner(&t.inner, &z)
	return fe.reduce(&z)
}

// Square2 sets `fe = 2*t^2`, and returns fe.
func (fe *Element) Square2(t *Element) *Element {
	var z [10]uint64
	squareInner(&t.inner, &z)
	for i := 0; i < 10; i++ {
		z[i] *= 2
	}
	return fe.reduce(&z)
}

func squareInner(x *[10]uint32, z *[10]uint64) {
	// Optimized version of multiplication for the case of squaring.
	// Pre- and post- conditions identical to multiplication function.
	x0_2 := 2 * x[0]
	x1_2 := 2 * x[1]
	x2_2 := 2 * x[2]
	x3_2 := 2 * x[3]
	x4_2 := 2 * x[4]
	x5_2 := 2 * x[5]
	x6_2 := 2 * x[6]
	x7_2 := 2 * x[7]
	x5_19 := 19 * x[5]
	x6_19 := 19 * x[6]
	x7_19 := 19 * x[7]
	x8_19 := 19 * x[8]
	x9_19 := 19 * x[9]

	// This block is rearranged so that instead of doing a 32-bit multiplication by 38, we do a
	// 64-bit multiplication by 2 on the results.  This is because lg(38) is too big: we would
	// have less than 1 bit of headroom left, which is too little.
	z[0] = m(x[0], x[0]) + m(x2_2, x8_19) + m(x4_2, x6_19) + (m(x1_2, x9_19)+m(x3_2, x7_19)+m(x[5], x5_19))*2
	z[1] = m(x0_2, x[1]) + m(x3_2, x8_19) + m(x5_2, x6_19) + (m(x[2], x9_19)+m(x[4], x7_19))*2
	z[2] = m(x0_2, x[2]) + m(x1_2, x[1]) + m(x4_2, x8_19) + m(x[6], x6_19) + (m(x3_2, x9_19)+m(x5_2, x7_19))*2
	z[3] = m(x0_2, x[3]) + m(x1_2, x[2]) + m(x5_2, x8_19) + (m(x[4], x9_19)+m(x[6], x7_19))*2
	z[4] = m(x0_2, x[4]) + m(x1_2, x3_2) + m(x[2], x[2]) + m(x6_2, x8_19) + (m(x5_2, x9_19)+m(x[7], x7_19))*2
	z[5] = m(x0_2, x[5]) + m(x1_2, x[4]) + m(x2_2, x[3]) + m(x7_2, x8_19) + m(x[6], x9_19)*2
	z[6] = m(x0_2, x[6]) + m(x1_2, x5_2) + m(x2_2, x[4]) + m(x3_2, x[3]) + m(x[8], x8_19) + m(x7_2, x9_19)*2
	z[7] = m(x0_2, x[7]) + m(x1_2, x[6]) + m(x2_2, x[5]) + m(x3_2, x[4]) + m(x[8], x9_19)*2
	z[8] = m(x0_2, x[8]) + m(x1_2, x7_2) + m(x2_2, x[6]) + m(x3_2, x5_2) + m(x[4], x[4]) + m(x[9], x9_19)*2
	z[9] = m(x0_2, x[9]) + m(x1_2, x[8]) + m(x2_2, x[7]) + m(x3_2, x[6]) + m(x4_2, x[5])
}

// NewElement2625 constructs a field element from its raw component limbs.
func NewElement2625(l0, l1, l2, l3, l4, l5, l6, l7, l8, l9 uint32) Element {
	return Element{
		inner: [10]uint32{
			l0, l1, l2, l3, l4, l5, l6, l7, l8, l9,
		},
	}
}
