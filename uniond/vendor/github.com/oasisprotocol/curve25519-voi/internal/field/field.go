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

// Package field implements field arithmetic modulo p = 2^255 - 19.
package field

import "github.com/oasisprotocol/curve25519-voi/internal/subtle"

const (
	// ElementSize is the size of a field element in bytes.
	ElementSize = 32

	// ElementWideSize is the size of a wide field element in bytes.
	ElementWideSize = 64
)

var (
	// One is the field element one.
	One = func() Element {
		var one Element
		one.One()
		return one
	}()

	// MinusOne is the field element -1.
	MinusOne = func() Element {
		var minusOne Element
		minusOne.MinusOne()
		return minusOne
	}()

	// Two is the field element two.
	Two = func() Element {
		var two Element
		two.Add(&One, &One)
		return two
	}()
)

// Set sets fe to t, and returns fe.
func (fe *Element) Set(t *Element) *Element {
	*fe = *t
	return fe
}

// Zero sets fe to zero, and returns fe.
func (fe *Element) Zero() *Element {
	for i := range fe.inner {
		fe.inner[i] = 0
	}
	return fe
}

// Equal returns 1 iff the field elements are equal, 0
// otherwise.  This function will execute in constant-time.
func (fe *Element) Equal(other *Element) int {
	var selfBytes, otherBytes [ElementSize]byte
	_ = fe.ToBytes(selfBytes[:])
	_ = other.ToBytes(otherBytes[:])

	return subtle.ConstantTimeCompareBytes(selfBytes[:], otherBytes[:])
}

// IsNegative returns 1 iff the field element is negative, 0 otherwise.
func (fe *Element) IsNegative() int {
	var selfBytes [ElementSize]byte
	_ = fe.ToBytes(selfBytes[:])

	return int(selfBytes[0] & 1)
}

// ConditionalNegate negates the field element iff choice == 1, leaves
// it unchanged otherwise.
func (fe *Element) ConditionalNegate(choice int) {
	var feNeg Element
	fe.ConditionalAssign(feNeg.Neg(fe), choice)
}

// IsZero returns 1 iff the field element is zero, 0 otherwise.
func (fe *Element) IsZero() int {
	var selfBytes, zeroBytes [ElementSize]byte
	_ = fe.ToBytes(selfBytes[:])

	return subtle.ConstantTimeCompareBytes(selfBytes[:], zeroBytes[:])
}

// Invert sets fe to the multiplicative inverse of t, and returns fe.
//
// The inverse is computed as self^(p-2), since x^(p-2)x = x^(p-1) = 1 (mod p).
//
// On input zero, the field element is set to zero.
func (fe *Element) Invert(t *Element) *Element {
	// The bits of p-2 = 2^255 -19 -2 are 11010111111...11.
	//
	//                       nonzero bits of exponent
	tmp, t3 := t.pow22501()  // t19: 249..0 ; t3: 3,1,0
	tmp.Pow2k(&tmp, 5)       // 254..5
	return fe.Mul(&tmp, &t3) // 254..5,3,1,0
}

// SqrtRatioI sets the fe to either `sqrt(u/v)` or `sqrt(i*u/v)` in constant
// time, and returns fe.  This function always selects the nonnegative square
// root.
func (fe *Element) SqrtRatioI(u, v *Element) (*Element, int) {
	// This uses the improved formula from the RFC 8032 eratta.
	//
	// See:
	//  * https://vox.distorted.org.uk/mdw/2017/05/simpler-quosqrt.html
	//  * https://www.rfc-editor.org/errata/eid5758
	//  * https://mailarchive.ietf.org/arch/msg/cfrg/qlKpMBqxXZYmDpXXIx6LO3Oznv4/
	var w Element
	w.Mul(u, v)

	var r Element
	w.pow_p58()
	r.Mul(u, &w)

	var check Element
	check.Square(&r)
	check.Mul(&check, v)

	var neg_u, neg_u_i Element
	neg_u.Neg(u)
	neg_u_i.Mul(&neg_u, &SQRT_M1)

	correct_sign_sqrt := check.Equal(u)
	flipped_sign_sqrt := check.Equal(&neg_u)
	flipped_sign_sqrt_i := check.Equal(&neg_u_i)

	var r_prime Element
	r_prime.Mul(&r, &SQRT_M1)
	r.ConditionalAssign(&r_prime, flipped_sign_sqrt|flipped_sign_sqrt_i)

	// Chose the nonnegative square root.
	r_is_negative := r.IsNegative()
	r.ConditionalNegate(r_is_negative)

	fe.Set(&r)

	return fe, correct_sign_sqrt | flipped_sign_sqrt
}

// InvSqrt attempts to set `fe = sqrt(1/self)` in constant time, and return fe.
// This function always selects the nonnegative square root.
func (fe *Element) InvSqrt() (*Element, int) {
	return fe.SqrtRatioI(&One, fe)
}

// pow22501 returns (self^(2^250-1), self^11), used as a helper function
// within Invert() and pow_p58().
func (fe *Element) pow22501() (Element, Element) {
	var t3, t19, tmp0, tmp1, tmp2 Element

	tmp0.Square(fe)        // t0 = fe^2
	tmp1.Pow2k(&tmp0, 2)   // t1 = t0^(2^2)
	tmp1.Mul(fe, &tmp1)    // t2 = fe * t1
	t3.Mul(&tmp0, &tmp1)   // t3 = t0 * t2
	tmp0.Square(&t3)       // t4 = t3^2
	tmp1.Mul(&tmp1, &tmp0) // t5 = t2 * t4
	tmp0.Pow2k(&tmp1, 5)   // t6 = t5^(2^5)
	tmp2.Mul(&tmp0, &tmp1) // t7 = t6 * t5
	tmp0.Pow2k(&tmp2, 10)  // t8 = t7^(2^10)
	tmp1.Mul(&tmp0, &tmp2) // t9 = t8 * t7
	tmp0.Pow2k(&tmp1, 20)  // t10 = t9^(2^20)
	tmp1.Mul(&tmp0, &tmp1) // t11 = t10 * t9
	tmp0.Pow2k(&tmp1, 10)  // t12 = t11^(2^10)
	tmp2.Mul(&tmp0, &tmp2) // t13 = t12 * t7
	tmp0.Pow2k(&tmp2, 50)  // t14 = t13^(2^50)
	tmp0.Mul(&tmp0, &tmp2) // t15 = t14 * t13
	tmp1.Pow2k(&tmp0, 100) // t16 = t15^(2^100)
	tmp0.Mul(&tmp1, &tmp0) // t17 = t16 * t15
	tmp0.Pow2k(&tmp0, 50)  // t18 = t17^(2*50)
	t19.Mul(&tmp0, &tmp2)  // t19 = t18 * t13

	return t19, t3
}

// pow_p58 raises the field element to the power (p-5)/8 = 2^252 - 3.
func (fe *Element) pow_p58() {
	// The bits of (p-5)/8 are 101111.....11.
	//
	//                      nonzero bits of exponent
	tmp, _ := fe.pow22501() // 249..0
	tmp.Pow2k(&tmp, 2)      // 251..2
	fe.Mul(fe, &tmp)        // 251..2,0
}

// BatchInvert computes the inverses of slice of `Elements`s
// in a batch, and replaces each element by its inverse.
//
// When an input Element is zero, its value is unchanged.
func BatchInvert(inputs []*Element) {
	// Montgomery’s Trick and Fast Implementation of Masked AES
	// Genelle, Prouff and Quisquater
	// Section 3.2
	n := len(inputs)
	scratch := make([]Element, n)
	for i := range scratch {
		scratch[i].One()
	}

	// Keep an accumulator of all of the previous products.
	var acc Element
	acc.One()

	// Pass through the input vector, recording the previous
	// products in the scratch space.
	var tmp Element
	for i, input := range inputs {
		scratch[i].Set(&acc)
		// acc <- acc * input, but skipping zeros (constant-time)
		tmp.Mul(&acc, input)
		acc.ConditionalSelect(&tmp, &acc, input.IsZero())
	}

	// Compute the inverse of all products.
	acc.Invert(&acc)

	// Pass through the vector backwards to compute the inverses
	// in place.
	var tmp2 Element
	for i := n - 1; i >= 0; i-- {
		input, scratch := inputs[i], scratch[i]
		tmp.Mul(&acc, input)
		tmp2.Mul(&acc, &scratch)

		// input <- acc * scratch, then acc <- tmp
		// Again, we skip zeros in a constant-time way
		iz := input.IsZero()

		inputs[i].ConditionalSelect(&tmp2, inputs[i], iz)
		acc.ConditionalSelect(&tmp, &acc, iz)
	}
}
