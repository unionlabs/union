// Copyright (c) 2017 George Tankersley. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// +build !amd64 noasm

package radix51

// Square sets v = x * x and returns v.
func (v *FieldElement) Square(x *FieldElement) *FieldElement {
	// Squaring needs only 15 mul instructions. Some inputs are multiplied by 2;
	// this is combined with multiplication by 19 where possible. The coefficient
	// reduction after squaring is the same as for multiplication.

	x0 := x[0]
	x1 := x[1]
	x2 := x[2]
	x3 := x[3]
	x4 := x[4]

	x0_2 := x0 << 1
	x1_2 := x1 << 1

	x1_38 := x1 * 38
	x2_38 := x2 * 38
	x3_38 := x3 * 38

	x3_19 := x3 * 19
	x4_19 := x4 * 19

	// r0 = x0*x0 + x1*38*x4 + x2*38*x3
	r00, r01 := madd64(0, 0, x0, x0)
	r00, r01 = madd64(r00, r01, x1_38, x4)
	r00, r01 = madd64(r00, r01, x2_38, x3)

	// r1 = x0*2*x1 + x2*38*x4 + x3*19*x3
	r10, r11 := madd64(0, 0, x0_2, x1)
	r10, r11 = madd64(r10, r11, x2_38, x4)
	r10, r11 = madd64(r10, r11, x3_19, x3)

	// r2 = x0*2*x2 + x1*x1 + x3*38*x4
	r20, r21 := madd64(0, 0, x0_2, x2)
	r20, r21 = madd64(r20, r21, x1, x1)
	r20, r21 = madd64(r20, r21, x3_38, x4)

	// r3 = x0*2*x3 + x1*2*x2 + x4*19*x4
	r30, r31 := madd64(0, 0, x0_2, x3)
	r30, r31 = madd64(r30, r31, x1_2, x2)
	r30, r31 = madd64(r30, r31, x4_19, x4)

	// r4 = x0*2*x4 + x1*2*x3 + x2*x2
	r40, r41 := madd64(0, 0, x0_2, x4)
	r40, r41 = madd64(r40, r41, x1_2, x3)
	r40, r41 = madd64(r40, r41, x2, x2)

	// Same reduction

	r01 = (r01 << 13) | (r00 >> 51)
	r00 &= maskLow51Bits

	r11 = (r11 << 13) | (r10 >> 51)
	r10 &= maskLow51Bits
	r10 += r01

	r21 = (r21 << 13) | (r20 >> 51)
	r20 &= maskLow51Bits
	r20 += r11

	r31 = (r31 << 13) | (r30 >> 51)
	r30 &= maskLow51Bits
	r30 += r21

	r41 = (r41 << 13) | (r40 >> 51)
	r40 &= maskLow51Bits
	r40 += r31

	r41 *= 19
	r00 += r41

	*v = FieldElement{r00, r10, r20, r30, r40}
	return v.carryPropagate1().carryPropagate2()
}
