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

package field

import (
	"encoding/binary"
	"fmt"
	"math/bits"

	"github.com/oasisprotocol/curve25519-voi/internal/disalloweq"
	"github.com/oasisprotocol/curve25519-voi/internal/subtle"
)

const (
	low_51_bit_mask uint64 = (1 << 51) - 1

	// 16 * p
	p_times_sixteen_0    = 36028797018963664
	p_times_sixteen_1234 = 36028797018963952
)

// Element represents an element of the field Z/(2^255 - 19).
type Element struct {
	disalloweq.DisallowEqual //nolint:unused
	inner                    [5]uint64
}

// Add sets `fe = a + b`, and returns fe.
func (fe *Element) Add(a, b *Element) *Element {
	fe.inner[0] = a.inner[0] + b.inner[0]
	fe.inner[1] = a.inner[1] + b.inner[1]
	fe.inner[2] = a.inner[2] + b.inner[2]
	fe.inner[3] = a.inner[3] + b.inner[3]
	fe.inner[4] = a.inner[4] + b.inner[4]
	return fe
}

// Sub sets `fe = a - b`, and returns fe.
func (fe *Element) Sub(a, b *Element) *Element {
	// To avoid underflow, first add a multiple of p.
	// Choose 16*p = p << 4 to be larger than 54-bit b.
	//
	// If we could statically track the bitlengths of the limbs
	// of every Element, we could choose a multiple of p
	// just bigger than b and avoid having to do a reduction.

	return fe.reduce(&[5]uint64{
		(a.inner[0] + p_times_sixteen_0) - b.inner[0],
		(a.inner[1] + p_times_sixteen_1234) - b.inner[1],
		(a.inner[2] + p_times_sixteen_1234) - b.inner[2],
		(a.inner[3] + p_times_sixteen_1234) - b.inner[3],
		(a.inner[4] + p_times_sixteen_1234) - b.inner[4],
	})
}

// Mul sets `fe =a * b`, and returns fe.
func (fe *Element) Mul(a, b *Element) *Element {
	feMul(fe, a, b)
	return fe
}

func feMulGeneric(fe, a, b *Element) { //nolint:unused,deadcode
	a0, a1, a2, a3, a4 := a.inner[0], a.inner[1], a.inner[2], a.inner[3], a.inner[4]
	b0, b1, b2, b3, b4 := b.inner[0], b.inner[1], b.inner[2], b.inner[3], b.inner[4]

	// Precondition: assume input limbs a[i], b[i] are bounded as
	//
	// a[i], b[i] < 2^(51 + b)
	//
	// where b is a real parameter measuring the "bit excess" of the limbs.

	// 64-bit precomputations to avoid 128-bit multiplications.
	//
	// This fits into a u64 whenever 51 + b + lg(19) < 64.
	//
	// Since 51 + b + lg(19) < 51 + 4.25 + b
	//                       = 55.25 + b,
	// this fits if b < 8.75.
	b1_19 := b1 * 19
	b2_19 := b2 * 19
	b3_19 := b3 * 19
	b4_19 := b4 * 19

	// Multiply to get 128-bit coefficients of output
	var carry uint64

	c0_hi, c0_lo := bits.Mul64(a0, b0)
	t0_hi, t0_lo := bits.Mul64(a4, b1_19)
	c0_lo, carry = bits.Add64(c0_lo, t0_lo, 0)
	c0_hi, _ = bits.Add64(c0_hi, t0_hi, carry)
	t0_hi, t0_lo = bits.Mul64(a3, b2_19)
	c0_lo, carry = bits.Add64(c0_lo, t0_lo, 0)
	c0_hi, _ = bits.Add64(c0_hi, t0_hi, carry)
	t0_hi, t0_lo = bits.Mul64(a2, b3_19)
	c0_lo, carry = bits.Add64(c0_lo, t0_lo, 0)
	c0_hi, _ = bits.Add64(c0_hi, t0_hi, carry)
	t0_hi, t0_lo = bits.Mul64(a1, b4_19)
	c0_lo, carry = bits.Add64(c0_lo, t0_lo, 0)
	c0_hi, _ = bits.Add64(c0_hi, t0_hi, carry)

	c1_hi, c1_lo := bits.Mul64(a1, b0)
	t1_hi, t1_lo := bits.Mul64(a0, b1)
	c1_lo, carry = bits.Add64(c1_lo, t1_lo, 0)
	c1_hi, _ = bits.Add64(c1_hi, t1_hi, carry)
	t1_hi, t1_lo = bits.Mul64(a4, b2_19)
	c1_lo, carry = bits.Add64(c1_lo, t1_lo, 0)
	c1_hi, _ = bits.Add64(c1_hi, t1_hi, carry)
	t1_hi, t1_lo = bits.Mul64(a3, b3_19)
	c1_lo, carry = bits.Add64(c1_lo, t1_lo, 0)
	c1_hi, _ = bits.Add64(c1_hi, t1_hi, carry)
	t1_hi, t1_lo = bits.Mul64(a2, b4_19)
	c1_lo, carry = bits.Add64(c1_lo, t1_lo, 0)
	c1_hi, _ = bits.Add64(c1_hi, t1_hi, carry)

	c2_hi, c2_lo := bits.Mul64(a2, b0)
	t2_hi, t2_lo := bits.Mul64(a1, b1)
	c2_lo, carry = bits.Add64(c2_lo, t2_lo, 0)
	c2_hi, _ = bits.Add64(c2_hi, t2_hi, carry)
	t2_hi, t2_lo = bits.Mul64(a0, b2)
	c2_lo, carry = bits.Add64(c2_lo, t2_lo, 0)
	c2_hi, _ = bits.Add64(c2_hi, t2_hi, carry)
	t2_hi, t2_lo = bits.Mul64(a4, b3_19)
	c2_lo, carry = bits.Add64(c2_lo, t2_lo, 0)
	c2_hi, _ = bits.Add64(c2_hi, t2_hi, carry)
	t2_hi, t2_lo = bits.Mul64(a3, b4_19)
	c2_lo, carry = bits.Add64(c2_lo, t2_lo, 0)
	c2_hi, _ = bits.Add64(c2_hi, t2_hi, carry)

	c3_hi, c3_lo := bits.Mul64(a3, b0)
	t3_hi, t3_lo := bits.Mul64(a2, b1)
	c3_lo, carry = bits.Add64(c3_lo, t3_lo, 0)
	c3_hi, _ = bits.Add64(c3_hi, t3_hi, carry)
	t3_hi, t3_lo = bits.Mul64(a1, b2)
	c3_lo, carry = bits.Add64(c3_lo, t3_lo, 0)
	c3_hi, _ = bits.Add64(c3_hi, t3_hi, carry)
	t3_hi, t3_lo = bits.Mul64(a0, b3)
	c3_lo, carry = bits.Add64(c3_lo, t3_lo, 0)
	c3_hi, _ = bits.Add64(c3_hi, t3_hi, carry)
	t3_hi, t3_lo = bits.Mul64(a4, b4_19)
	c3_lo, carry = bits.Add64(c3_lo, t3_lo, 0)
	c3_hi, _ = bits.Add64(c3_hi, t3_hi, carry)

	c4_hi, c4_lo := bits.Mul64(a4, b0)
	t4_hi, t4_lo := bits.Mul64(a3, b1)
	c4_lo, carry = bits.Add64(c4_lo, t4_lo, 0)
	c4_hi, _ = bits.Add64(c4_hi, t4_hi, carry)
	t4_hi, t4_lo = bits.Mul64(a2, b2)
	c4_lo, carry = bits.Add64(c4_lo, t4_lo, 0)
	c4_hi, _ = bits.Add64(c4_hi, t4_hi, carry)
	t4_hi, t4_lo = bits.Mul64(a1, b3)
	c4_lo, carry = bits.Add64(c4_lo, t4_lo, 0)
	c4_hi, _ = bits.Add64(c4_hi, t4_hi, carry)
	t4_hi, t4_lo = bits.Mul64(a0, b4)
	c4_lo, carry = bits.Add64(c4_lo, t4_lo, 0)
	c4_hi, _ = bits.Add64(c4_hi, t4_hi, carry)

	// How big are the c[i]? We have
	//
	//    c[i] < 2^(102 + 2*b) * (1+i + (4-i)*19)
	//         < 2^(102 + lg(1 + 4*19) + 2*b)
	//         < 2^(108.27 + 2*b)
	//
	// The carry (c[i] >> 51) fits into a u64 when
	//    108.27 + 2*b - 51 < 64
	//    2*b < 6.73
	//    b < 3.365.
	//
	// So we require b < 3 to ensure this fits.

	tmp := (c0_hi << (64 - 51)) | (c0_lo >> 51)
	c1_lo, carry = bits.Add64(c1_lo, tmp, 0)
	c1_hi, _ = bits.Add64(c1_hi, 0, carry)
	fe0 := c0_lo & low_51_bit_mask

	tmp = (c1_hi << (64 - 51)) | (c1_lo >> 51)
	c2_lo, carry = bits.Add64(c2_lo, tmp, 0)
	c2_hi, _ = bits.Add64(c2_hi, 0, carry)
	fe1 := c1_lo & low_51_bit_mask

	tmp = (c2_hi << (64 - 51)) | (c2_lo >> 51)
	c3_lo, carry = bits.Add64(c3_lo, tmp, 0)
	c3_hi, _ = bits.Add64(c3_hi, 0, carry)
	fe.inner[2] = c2_lo & low_51_bit_mask

	tmp = (c3_hi << (64 - 51)) | (c3_lo >> 51)
	c4_lo, carry = bits.Add64(c4_lo, tmp, 0)
	c4_hi, _ = bits.Add64(c4_hi, 0, carry)
	fe.inner[3] = c3_lo & low_51_bit_mask

	carry = (c4_hi << (64 - 51)) | (c4_lo >> 51)
	fe.inner[4] = c4_lo & low_51_bit_mask

	// To see that this does not overflow, we need fe[0] + carry * 19 < 2^64.
	//
	// c4 < a0*b4 + a1*b3 + a2*b2 + a3*b1 + a4*b0 + (carry from c3)
	//    < 5*(2^(51 + b) * 2^(51 + b)) + (carry from c3)
	//    < 2^(102 + 2*b + lg(5)) + 2^64.
	//
	// When b < 3 we get
	//
	// c4 < 2^110.33  so that carry < 2^59.33
	//
	// so that
	//
	// fe[0] + carry * 19 < 2^51 + 19 * 2^59.33 < 2^63.58
	//
	// and there is no overflow.
	fe0 = fe0 + carry*19

	// Now fe[1] < 2^51 + 2^(64 -51) = 2^51 + 2^13 < 2^(51 + epsilon).
	fe.inner[1] = fe1 + (fe0 >> 51)
	fe.inner[0] = fe0 & low_51_bit_mask

	// Now fe[i] < 2^(51 + epsilon) for all i.
}

// Mul121666 sets `fe = t * 121666`, and returns fe.
func (fe *Element) Mul121666(t *Element) *Element {
	a0, a1, a2, a3, a4 := t.inner[0], t.inner[1], t.inner[2], t.inner[3], t.inner[4]
	b0 := uint64(121666)

	// Multiply to get 128-bit coefficients of output

	c0_hi, c0_lo := bits.Mul64(a0, b0)
	c1_hi, c1_lo := bits.Mul64(a1, b0)
	c2_hi, c2_lo := bits.Mul64(a2, b0)
	c3_hi, c3_lo := bits.Mul64(a3, b0)
	c4_hi, c4_lo := bits.Mul64(a4, b0)

	// Just do a weak reduction like in the multiply, the range
	// analysis trivially holds due `b0` being tiny.

	var carry uint64

	tmp := (c0_hi << (64 - 51)) | (c0_lo >> 51)
	c1_lo, carry = bits.Add64(c1_lo, tmp, 0)
	c1_hi, _ = bits.Add64(c1_hi, 0, carry)
	fe0 := c0_lo & low_51_bit_mask

	tmp = (c1_hi << (64 - 51)) | (c1_lo >> 51)
	c2_lo, carry = bits.Add64(c2_lo, tmp, 0)
	c2_hi, _ = bits.Add64(c2_hi, 0, carry)
	fe1 := c1_lo & low_51_bit_mask

	tmp = (c2_hi << (64 - 51)) | (c2_lo >> 51)
	c3_lo, carry = bits.Add64(c3_lo, tmp, 0)
	c3_hi, _ = bits.Add64(c3_hi, 0, carry)
	fe.inner[2] = c2_lo & low_51_bit_mask

	tmp = (c3_hi << (64 - 51)) | (c3_lo >> 51)
	c4_lo, carry = bits.Add64(c4_lo, tmp, 0)
	c4_hi, _ = bits.Add64(c4_hi, 0, carry)
	fe.inner[3] = c3_lo & low_51_bit_mask

	carry = (c4_hi << (64 - 51)) | (c4_lo >> 51)
	fe.inner[4] = c4_lo & low_51_bit_mask

	fe0 = fe0 + carry*19

	fe.inner[1] = fe1 + (fe0 >> 51)
	fe.inner[0] = fe0 & low_51_bit_mask

	return fe
}

// Neg sets `fe = -t`, and returns fe.
func (fe *Element) Neg(t *Element) *Element {
	// See commentary in the Sub impl.
	return fe.reduce(&[5]uint64{
		p_times_sixteen_0 - t.inner[0],
		p_times_sixteen_1234 - t.inner[1],
		p_times_sixteen_1234 - t.inner[2],
		p_times_sixteen_1234 - t.inner[3],
		p_times_sixteen_1234 - t.inner[4],
	})
}

// ConditionalSelect sets the field element to a iff choice == 0 and
// b iff choice == 1.
func (fe *Element) ConditionalSelect(a, b *Element, choice int) {
	fe.inner[0] = subtle.ConstantTimeSelectUint64(choice, b.inner[0], a.inner[0])
	fe.inner[1] = subtle.ConstantTimeSelectUint64(choice, b.inner[1], a.inner[1])
	fe.inner[2] = subtle.ConstantTimeSelectUint64(choice, b.inner[2], a.inner[2])
	fe.inner[3] = subtle.ConstantTimeSelectUint64(choice, b.inner[3], a.inner[3])
	fe.inner[4] = subtle.ConstantTimeSelectUint64(choice, b.inner[4], a.inner[4])
}

// ConditionalSwap conditionally swaps the field elements according to choice.
func (fe *Element) ConditionalSwap(other *Element, choice int) {
	subtle.ConstantTimeSwapUint64(choice, &other.inner[0], &fe.inner[0])
	subtle.ConstantTimeSwapUint64(choice, &other.inner[1], &fe.inner[1])
	subtle.ConstantTimeSwapUint64(choice, &other.inner[2], &fe.inner[2])
	subtle.ConstantTimeSwapUint64(choice, &other.inner[3], &fe.inner[3])
	subtle.ConstantTimeSwapUint64(choice, &other.inner[4], &fe.inner[4])
}

// ConditionalAssign conditionally assigns the field element according to choice.
func (fe *Element) ConditionalAssign(other *Element, choice int) {
	fe.inner[0] = subtle.ConstantTimeSelectUint64(choice, other.inner[0], fe.inner[0])
	fe.inner[1] = subtle.ConstantTimeSelectUint64(choice, other.inner[1], fe.inner[1])
	fe.inner[2] = subtle.ConstantTimeSelectUint64(choice, other.inner[2], fe.inner[2])
	fe.inner[3] = subtle.ConstantTimeSelectUint64(choice, other.inner[3], fe.inner[3])
	fe.inner[4] = subtle.ConstantTimeSelectUint64(choice, other.inner[4], fe.inner[4])
}

// One sets the fe to one, and returns fe.
func (fe *Element) One() *Element {
	*fe = NewElement51(1, 0, 0, 0, 0)
	return fe
}

// MinusOne sets fe to -1, and returns fe.
func (fe *Element) MinusOne() *Element {
	*fe = NewElement51(
		2251799813685228, 2251799813685247, 2251799813685247, 2251799813685247, 2251799813685247,
	)
	return fe
}

func (fe *Element) reduce(limbs *[5]uint64) *Element {
	// Since the input limbs are bounded by 2^64, the biggest
	// carry-out is bounded by 2^13.
	//
	// The biggest carry-in is c4 * 19, resulting in
	//
	// 2^51 + 19*2^13 < 2^51.0000000001
	//
	// Because we don't need to canonicalize, only to reduce the
	// limb sizes, it's OK to do a "weak reduction", where we
	// compute the carry-outs in parallel.

	l0, l1, l2, l3, l4 := limbs[0], limbs[1], limbs[2], limbs[3], limbs[4]

	c0 := l0 >> 51
	c1 := l1 >> 51
	c2 := l2 >> 51
	c3 := l3 >> 51
	c4 := l4 >> 51

	l0 &= low_51_bit_mask
	l1 &= low_51_bit_mask
	l2 &= low_51_bit_mask
	l3 &= low_51_bit_mask
	l4 &= low_51_bit_mask

	fe.inner[0] = l0 + c4*19
	fe.inner[1] = l1 + c0
	fe.inner[2] = l2 + c1
	fe.inner[3] = l3 + c2
	fe.inner[4] = l4 + c3

	return fe
}

// SetBytes loads a field element from the low 255 bits of a 256 bit input.
//
// WARNING: This function does not check that the input used the canonical
// representative.  It masks the high bit, but it will happily decode
// 2^255 - 18 to 1.  Applications that require a canonical encoding of
// every field element should decode, re-encode to the canonical encoding,
// and check that the input was canonical.
func (fe *Element) SetBytes(in []byte) (*Element, error) {
	if len(in) != ElementSize {
		return nil, fmt.Errorf("internal/field/u64: unexpected input size")
	}

	_ = in[31]
	*fe = Element{
		inner: [5]uint64{
			// load bits [  0, 64), no shift
			binary.LittleEndian.Uint64(in[0:8]) & low_51_bit_mask,
			// load bits [ 48,112), shift to [ 51,112)
			(binary.LittleEndian.Uint64(in[6:14]) >> 3) & low_51_bit_mask,
			// load bits [ 96,160), shift to [102,160)
			(binary.LittleEndian.Uint64(in[12:20]) >> 6) & low_51_bit_mask,
			// load bits [152,216), shift to [153,216)
			(binary.LittleEndian.Uint64(in[19:27]) >> 1) & low_51_bit_mask,
			// load bits [192,256), shift to [204,112)
			(binary.LittleEndian.Uint64(in[24:32]) >> 12) & low_51_bit_mask,
		},
	}

	return fe, nil
}

// SetBytesWide loads a field element from a 512-bit little-endian input.
func (fe *Element) SetBytesWide(in []byte) (*Element, error) {
	if len(in) != ElementWideSize {
		return nil, fmt.Errorf("internal/field/u64: unexpected input size")
	}

	var lo, hi Element
	if _, err := lo.SetBytes(in[:ElementSize]); err != nil {
		return nil, fmt.Errorf("internal/field/u64: failed to deserialize lo: %w", err)
	}
	if _, err := hi.SetBytes(in[ElementSize:]); err != nil {
		return nil, fmt.Errorf("internal/field/u64: failed to deserialize hi: %w", err)
	}

	// Handle the 256th and 512th bits (MSB of lo and hi) explicitly
	// as SetBytes ignores them.
	lo.inner[0] += uint64(in[31]>>7)*19 + uint64(in[63]>>7)*2*19*19

	lo.inner[0] += 2 * 19 * hi.inner[0]
	lo.inner[1] += 2 * 19 * hi.inner[1]
	lo.inner[2] += 2 * 19 * hi.inner[2]
	lo.inner[3] += 2 * 19 * hi.inner[3]
	lo.inner[4] += 2 * 19 * hi.inner[4]

	fe.reduce(&lo.inner)

	return fe, nil
}

// ToBytes packs the field element into 32 bytes.  The encoding is canonical.
func (fe *Element) ToBytes(out []byte) error {
	if len(out) != ElementSize {
		return fmt.Errorf("internal/field/u64: unexpected output size")
	}

	// Let h = limbs[0] + limbs[1]*2^51 + ... + limbs[4]*2^204.
	//
	// Write h = pq + r with 0 <= r < p.
	//
	// We want to compute r = h mod p.
	//
	// If h < 2*p = 2^256 - 38,
	// then q = 0 or 1,
	//
	// with q = 0 when h < p
	//  and q = 1 when h >= p.
	//
	// Notice that h >= p <==> h + 19 >= p + 19 <==> h + 19 >= 2^255.
	// Therefore q can be computed as the carry bit of h + 19.

	// First, reduce the limbs to ensure h < 2*p.
	var reduced Element
	reduced.reduce(&fe.inner)
	l0, l1, l2, l3, l4 := reduced.inner[0], reduced.inner[1], reduced.inner[2], reduced.inner[3], reduced.inner[4]

	q := (l0 + 19) >> 51
	q = (l1 + q) >> 51
	q = (l2 + q) >> 51
	q = (l3 + q) >> 51
	q = (l4 + q) >> 51

	// Now we can compute r as r = h - pq = r - (2^255-19)q = r + 19q - 2^255q

	l0 += 19 * q

	// Now carry the result to compute r + 19q ...
	l1 += l0 >> 51
	l0 = l0 & low_51_bit_mask
	l2 += l1 >> 51
	l1 = l1 & low_51_bit_mask
	l3 += l2 >> 51
	l2 = l2 & low_51_bit_mask
	l4 += l3 >> 51
	l3 = l3 & low_51_bit_mask
	// ... but instead of carrying (l4 >> 51) = 2^255q
	// into another limb, discard it, subtracting the value
	l4 = l4 & low_51_bit_mask

	out[0] = byte(l0)
	out[1] = byte(l0 >> 8)
	out[2] = byte(l0 >> 16)
	out[3] = byte(l0 >> 24)
	out[4] = byte(l0 >> 32)
	out[5] = byte(l0 >> 40)
	out[6] = byte((l0 >> 48) | (l1 << 3))
	out[7] = byte(l1 >> 5)
	out[8] = byte(l1 >> 13)
	out[9] = byte(l1 >> 21)
	out[10] = byte(l1 >> 29)
	out[11] = byte(l1 >> 37)
	out[12] = byte((l1 >> 45) | (l2 << 6))
	out[13] = byte(l2 >> 2)
	out[14] = byte(l2 >> 10)
	out[15] = byte(l2 >> 18)
	out[16] = byte(l2 >> 26)
	out[17] = byte(l2 >> 34)
	out[18] = byte(l2 >> 42)
	out[19] = byte((l2 >> 50) | (l3 << 1))
	out[20] = byte(l3 >> 7)
	out[21] = byte(l3 >> 15)
	out[22] = byte(l3 >> 23)
	out[23] = byte(l3 >> 31)
	out[24] = byte(l3 >> 39)
	out[25] = byte((l3 >> 47) | (l4 << 4))
	out[26] = byte(l4 >> 4)
	out[27] = byte(l4 >> 12)
	out[28] = byte(l4 >> 20)
	out[29] = byte(l4 >> 28)
	out[30] = byte(l4 >> 36)
	out[31] = byte(l4 >> 44)

	return nil
}

// Pow2k sets `fe = t^(2^k)`, given `k > 0`, and returns fe
func (fe *Element) Pow2k(t *Element, k uint) *Element {
	if k == 0 {
		panic("internal/field/u64: k out of bounds")
	}

	fePow2k(fe, t, k)
	return fe
}

func fePow2kGeneric(fe, t *Element, k uint) { //nolint:unused,deadcode
	a0, a1, a2, a3, a4 := t.inner[0], t.inner[1], t.inner[2], t.inner[3], t.inner[4]

	for {
		// Precondition: assume input limbs a[i] are bounded as
		//
		// a[i] < 2^(51 + b)
		//
		// where b is a real parameter measuring the "bit excess" of the limbs.

		// Precomputation: 64-bit multiply by 19.
		//
		// This fits into a u64 whenever 51 + b + lg(19) < 64.
		//
		// Since 51 + b + lg(19) < 51 + 4.25 + b
		//                       = 55.25 + b,
		// this fits if b < 8.75.
		a3_19 := 19 * a3
		a4_19 := 19 * a4

		// Multiply to get 128-bit coefficients of output.
		//
		// Note: dalek just uses 128-bit multiplication here instead of
		// doing some precomputation.  Since Go does not have an actual
		// 128-bit integer type, this will opt for precomputing, primarily
		// for the sake of readability.
		//
		// This fits into a u64 whenever 51 + b + lg(1) < 64.

		d0 := 2 * a0
		d1 := 2 * a1
		d2 := 2 * a2
		d4 := 2 * a4

		var carry uint64

		c0_hi, c0_lo := bits.Mul64(a0, a0)
		t0_hi, t0_lo := bits.Mul64(d1, a4_19)
		c0_lo, carry = bits.Add64(c0_lo, t0_lo, 0)
		c0_hi, _ = bits.Add64(c0_hi, t0_hi, carry)
		t0_hi, t0_lo = bits.Mul64(d2, a3_19)
		c0_lo, carry = bits.Add64(c0_lo, t0_lo, 0)
		c0_hi, _ = bits.Add64(c0_hi, t0_hi, carry)

		c1_hi, c1_lo := bits.Mul64(a3, a3_19)
		t1_hi, t1_lo := bits.Mul64(d0, a1)
		c1_lo, carry = bits.Add64(c1_lo, t1_lo, 0)
		c1_hi, _ = bits.Add64(c1_hi, t1_hi, carry)
		t1_hi, t1_lo = bits.Mul64(d2, a4_19)
		c1_lo, carry = bits.Add64(c1_lo, t1_lo, 0)
		c1_hi, _ = bits.Add64(c1_hi, t1_hi, carry)

		c2_hi, c2_lo := bits.Mul64(a1, a1)
		t2_hi, t2_lo := bits.Mul64(d0, a2)
		c2_lo, carry = bits.Add64(c2_lo, t2_lo, 0)
		c2_hi, _ = bits.Add64(c2_hi, t2_hi, carry)
		t2_hi, t2_lo = bits.Mul64(d4, a3_19)
		c2_lo, carry = bits.Add64(c2_lo, t2_lo, 0)
		c2_hi, _ = bits.Add64(c2_hi, t2_hi, carry)

		c3_hi, c3_lo := bits.Mul64(a4, a4_19)
		t3_hi, t3_lo := bits.Mul64(d0, a3)
		c3_lo, carry = bits.Add64(c3_lo, t3_lo, 0)
		c3_hi, _ = bits.Add64(c3_hi, t3_hi, carry)
		t3_hi, t3_lo = bits.Mul64(d1, a2)
		c3_lo, carry = bits.Add64(c3_lo, t3_lo, 0)
		c3_hi, _ = bits.Add64(c3_hi, t3_hi, carry)

		c4_hi, c4_lo := bits.Mul64(a2, a2)
		t4_hi, t4_lo := bits.Mul64(d0, a4)
		c4_lo, carry = bits.Add64(c4_lo, t4_lo, 0)
		c4_hi, _ = bits.Add64(c4_hi, t4_hi, carry)
		t4_hi, t4_lo = bits.Mul64(d1, a3)
		c4_lo, carry = bits.Add64(c4_lo, t4_lo, 0)
		c4_hi, _ = bits.Add64(c4_hi, t4_hi, carry)

		// Same bound as in multiply:
		//    c[i] < 2^(102 + 2*b) * (1+i + (4-i)*19)
		//         < 2^(102 + lg(1 + 4*19) + 2*b)
		//         < 2^(108.27 + 2*b)
		//
		// The carry (c[i] >> 51) fits into a u64 when
		//    108.27 + 2*b - 51 < 64
		//    2*b < 6.73
		//    b < 3.365.
		//
		// So we require b < 3 to ensure this fits.

		tmp := (c0_hi << (64 - 51)) | (c0_lo >> 51)
		c1_lo, carry = bits.Add64(c1_lo, tmp, 0)
		c1_hi, _ = bits.Add64(c1_hi, 0, carry)
		a0 = c0_lo & low_51_bit_mask

		tmp = (c1_hi << (64 - 51)) | (c1_lo >> 51)
		c2_lo, carry = bits.Add64(c2_lo, tmp, 0)
		c2_hi, _ = bits.Add64(c2_hi, 0, carry)
		a1 = c1_lo & low_51_bit_mask

		tmp = (c2_hi << (64 - 51)) | (c2_lo >> 51)
		c3_lo, carry = bits.Add64(c3_lo, tmp, 0)
		c3_hi, _ = bits.Add64(c3_hi, 0, carry)
		a2 = c2_lo & low_51_bit_mask

		tmp = (c3_hi << (64 - 51)) | (c3_lo >> 51)
		c4_lo, carry = bits.Add64(c4_lo, tmp, 0)
		c4_hi, _ = bits.Add64(c4_hi, 0, carry)
		a3 = c3_lo & low_51_bit_mask

		carry = (c4_hi << (64 - 51)) | (c4_lo >> 51)
		a4 = c4_lo & low_51_bit_mask

		// To see that this does not overflow, we need a[0] + carry * 19 < 2^64.
		//
		// c4 < a2^2 + 2*a0*a4 + 2*a1*a3 + (carry from c3)
		//    < 2^(102 + 2*b + lg(5)) + 2^64.
		//
		// When b < 3 we get
		//
		// c4 < 2^110.33  so that carry < 2^59.33
		//
		// so that
		//
		// a[0] + carry * 19 < 2^51 + 19 * 2^59.33 < 2^63.58
		//
		// and there is no overflow.
		a0 = a0 + carry*19

		// Now a[1] < 2^51 + 2^(64 -51) = 2^51 + 2^13 < 2^(51 + epsilon).
		a1 += a0 >> 51
		a0 &= low_51_bit_mask

		// Now all a[i] < 2^(51 + epsilon) and a = self^(2^k).

		k--
		if k == 0 {
			break
		}
	}

	fe.inner[0], fe.inner[1], fe.inner[2], fe.inner[3], fe.inner[4] = a0, a1, a2, a3, a4
}

// Square sets `fe = t^2`, and returns fe.
func (fe *Element) Square(t *Element) *Element {
	fePow2k(fe, t, 1)
	return fe
}

// Square2 sets `fe = 2*t^2`, and returns fe.
func (fe *Element) Square2(t *Element) *Element {
	fePow2k(fe, t, 1)
	for i := 0; i < 5; i++ {
		fe.inner[i] *= 2
	}
	return fe
}

// UnsafeInner exposes the inner limbs to allow for the vector implementation.
func (fe *Element) UnsafeInner() *[5]uint64 {
	return &fe.inner
}

// NewElement51 constructs a field element from its raw component limbs.
func NewElement51(l0, l1, l2, l3, l4 uint64) Element {
	return Element{
		inner: [5]uint64{
			l0, l1, l2, l3, l4,
		},
	}
}
