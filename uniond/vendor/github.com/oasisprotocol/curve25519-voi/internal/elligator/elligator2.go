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

package elligator

import (
	"github.com/oasisprotocol/curve25519-voi/curve"
	"github.com/oasisprotocol/curve25519-voi/internal/field"
)

var constFieldZero field.Element

// SetEdwardsFromXY sets the EdwardsPoint to that corresponding to the x
// and y coordinates.
func SetEdwardsFromXY(p *curve.EdwardsPoint, x, y *field.Element) *curve.EdwardsPoint {
	// While being able to create a curve.EdwardsPoint from the x and y
	// coordinate (`(x, y, 1, x*y)`) than doing decompression, not having
	// to have something ugly like EdwardsPoint.InternalSetXY that exposes
	// the presence of the internal field package is probably better.
	var pCompressed curve.CompressedEdwardsY
	_ = y.ToBytes(pCompressed[:])
	pCompressed[31] ^= byte(x.IsNegative()) << 7

	// EdwardsFlavor ensures that this cannot fail.
	if _, err := p.SetCompressedY(&pCompressed); err != nil {
		panic("internal/elligator: failed to decompress point: " + err.Error())
	}

	return p
}

// EdwardsFlavor computes EdwardsPoint corresponding to the provided Elligator 2
// representative.
func EdwardsFlavor(r *field.Element) *curve.EdwardsPoint {
	u, v := montgomeryFlavor(r)

	// Per RFC 7748: (x, y) = (sqrt(-486664)*u/v, (u-1)/(u+1))

	var x field.Element
	x.Invert(&v)
	x.Mul(&x, &u)
	x.Mul(&x, &constMONTGOMERY_SQRT_NEG_A_PLUS_TWO)

	var uMinusOne, uPlusOne, y field.Element
	uMinusOne.Sub(&u, &field.One)
	uPlusOne.Add(&u, &field.One)
	uPlusOneIsZero := uPlusOne.IsZero()
	uPlusOne.Invert(&uPlusOne)
	y.Mul(&uMinusOne, &uPlusOne)

	// This does something slightly different than EdwardsPoint.SetMontgomery
	// as that conversion routine bails if u == -1.  However the hash-to-curve
	// specification for this mapping requires:
	//
	//   This mapping is undefined when t == 0 or s == -1, i.e., when the
	//   denominator of either of the above rational functions is zero.
	//   Implementations MUST detect exceptional cases and return the value
	//   (v, w) = (0, 1), which is the identity point on all twisted Edwards
	//   curves.
	resultUndefined := uPlusOneIsZero | v.IsZero()
	x.ConditionalAssign(&constFieldZero, resultUndefined)
	y.ConditionalAssign(&field.One, resultUndefined)

	var p curve.EdwardsPoint
	return SetEdwardsFromXY(&p, &x, &y)
}

// montgomeryFlavor computes Montgomery u and v coordinates corresponding
// to the provided Elligator 2 representative.
func montgomeryFlavor(r *field.Element) (field.Element, field.Element) {
	// This is based off the public domain python implementation by
	// Loup Vaillant, taken from the Monocypher package
	// (tests/gen/elligator.py).
	//
	// The choice of base implementation is primarily because it was
	// convenient, and because they appear to be one of the people
	// that have given the most thought regarding how to implement
	// this correctly, with a readable implementation that I can
	// wrap my brain around.

	var (
		t1, t2, t3, u, v field.Element
		isSquare         int
	)

	t1.Square2(r)          // r1
	u.Add(&t1, &field.One) // r2
	t2.Square(&u)

	t3.Mul(&constMONTGOMERY_A_SQUARED, &t1) // numerator
	t3.Sub(&t3, &t2)
	t3.Mul(&t3, &constMONTGOMERY_A)

	t1.Mul(&t2, &u) // denominator

	t1.Mul(&t1, &t3)
	_, isSquare = t1.InvSqrt()

	u.Square(r)
	u.Mul(&u, &constMONTGOMERY_U_FACTOR)

	v.Mul(r, &constMONTGOMERY_V_FACTOR)

	u.ConditionalAssign(&field.One, isSquare)
	v.ConditionalAssign(&field.One, isSquare)

	v.Mul(&v, &t3)
	v.Mul(&v, &t1)

	t1.Square(&t1)

	u.Mul(&u, &constMONTGOMERY_NEG_A)
	u.Mul(&u, &t3)
	u.Mul(&u, &t2)
	u.Mul(&u, &t1)

	v.ConditionalNegate(isSquare ^ v.IsNegative())

	return u, v
}
