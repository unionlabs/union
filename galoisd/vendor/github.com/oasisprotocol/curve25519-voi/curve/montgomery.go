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

import (
	"fmt"

	"github.com/oasisprotocol/curve25519-voi/curve/scalar"
	"github.com/oasisprotocol/curve25519-voi/internal/field"
)

var errUCoordinateOnTwist = fmt.Errorf("curve/montgomery: Montgomery u-coordinate is on twist")

// MontgomeryPoint holds the u-coordinate of a point on the Montgomery
// form of Curve25519 or its twist.
type MontgomeryPoint [MontgomeryPointSize]byte

// SetBytes constructs a Montgomery u-coordinate from a byte representation.
func (p *MontgomeryPoint) SetBytes(in []byte) (*MontgomeryPoint, error) {
	if len(in) != CompressedPointSize {
		return nil, fmt.Errorf("curve/montgomery: unexpected input size")
	}

	copy(p[:], in)

	return p, nil
}

// Equal returns 1 iff the points are equal, 0 otherwise.  This function
// will execute in constant-time.
func (p *MontgomeryPoint) Equal(other *MontgomeryPoint) int {
	var selfFe, otherFe field.Element
	_, _ = selfFe.SetBytes(p[:])
	_, _ = otherFe.SetBytes(other[:])

	return selfFe.Equal(&otherFe)
}

// SetEdwards converts an EdwardsPoint to a MontgomeryPoint.
//
// This function has one exceptional case; the identity point of
// the edwards curve is set to the 2-torsion point (0, 0) on the Montgomery
// curve.
func (p *MontgomeryPoint) SetEdwards(edwardsPoint *EdwardsPoint) *MontgomeryPoint {
	// We have u = (1+y)/(1-y) = (Z+Y)/(Z-Y).
	//
	// The denominator is zero only when y=1, the identity point of
	// the Edwards curve.  Since 0.invert() = 0, in this case we
	// compute the 2-torsion point (0,0).
	var U, W, u field.Element
	U.Add(&edwardsPoint.inner.Z, &edwardsPoint.inner.Y)
	W.Sub(&edwardsPoint.inner.Z, &edwardsPoint.inner.Y)
	W.Invert(&W)
	u.Mul(&U, &W)

	_ = u.ToBytes(p[:])

	return p
}

// Mul sets `p = point * scalar` in constant-time, and returns p.
func (p *MontgomeryPoint) Mul(point *MontgomeryPoint, scalar *scalar.Scalar) *MontgomeryPoint {
	// Algorithm 8 of Costello-Smith 2017.
	var affineU field.Element
	_, _ = affineU.SetBytes(point[:])
	var x0, x1 montgomeryProjectivePoint
	x0.identity()
	x1.U.Set(&affineU)
	x1.W.One()

	bits := scalar.Bits()

	for i := 254; i >= 0; i-- {
		choice := int(bits[i+1] ^ bits[i])

		x0.conditionalSwap(&x1, choice)
		montgomeryDifferentialAddAndDouble(&x0, &x1, &affineU)
	}
	x0.conditionalSwap(&x1, int(bits[0]))

	return p.fromProjective(&x0)
}

func (p *MontgomeryPoint) fromProjective(pp *montgomeryProjectivePoint) *MontgomeryPoint {
	// Dehomogenize the projective point to affine coordinates.
	var u, wInv field.Element
	wInv.Invert(&pp.W)
	u.Mul(&pp.U, &wInv)

	_ = u.ToBytes(p[:])

	return p
}

// NewMontgomeryPoint constructs a new Montgomery point.
func NewMontgomeryPoint() *MontgomeryPoint {
	return &MontgomeryPoint{}
}

func montgomeryDifferentialAddAndDouble(P, Q *montgomeryProjectivePoint, affine_PmQ *field.Element) {
	var t0, t1, t2, t3 field.Element
	t0.Add(&P.U, &P.W)
	t1.Sub(&P.U, &P.W)
	t2.Add(&Q.U, &Q.W)
	t3.Sub(&Q.U, &Q.W)

	var t4, t5 field.Element
	t4.Square(&t0) // (U_P + W_P)^2 = U_P^2 + 2 U_P W_P + W_P^2
	t5.Square(&t1) // (U_P - W_P)^2 = U_P^2 - 2 U_P W_P + W_P^2

	var t6 field.Element
	t6.Sub(&t4, &t5) // 4 U_P W_P

	var t7, t8 field.Element
	t7.Mul(&t0, &t3) // (U_P + W_P) (U_Q - W_Q) = U_P U_Q + W_P U_Q - U_P W_Q - W_P W_Q
	t8.Mul(&t1, &t2) // (U_P - W_P) (U_Q + W_Q) = U_P U_Q - W_P U_Q + U_P W_Q - W_P W_Q

	// Note: dalek uses even more temporary variables, but eliminating them
	// is slightly faster since the Go compiler won't do that for us.

	Q.U.Add(&t7, &t8) // 2 (U_P U_Q - W_P W_Q): t9
	Q.W.Sub(&t7, &t8) // 2 (W_P U_Q - U_P W_Q): t10

	Q.U.Square(&Q.U) // 4 (U_P U_Q - W_P W_Q)^2: t11
	Q.W.Square(&Q.W) // 4 (W_P U_Q - U_P W_Q)^2: t12

	P.W.Mul121666(&t6) // (A + 2) U_P U_Q: t13

	P.U.Mul(&t4, &t5)  // ((U_P + W_P)(U_P - W_P))^2 = (U_P^2 - W_P^2)^2: t14
	P.W.Add(&P.W, &t5) // (U_P - W_P)^2 + (A + 2) U_P W_P: t15

	P.W.Mul(&t6, &P.W) // 4 (U_P W_P) ((U_P - W_P)^2 + (A + 2) U_P W_P): t16

	Q.W.Mul(affine_PmQ, &Q.W) // U_D * 4 (W_P U_Q - U_P W_Q)^2: t17
	// t18 := t11             // W_D * 4 (U_P U_Q - W_P W_Q)^2: t18

	// P.U = t14 // U_{P'} = (U_P + W_P)^2 (U_P - W_P)^2
	// P.W = t16 // W_{P'} = (4 U_P W_P) ((U_P - W_P)^2 + ((A + 2)/4) 4 U_P W_P)
	// Q.U = t18 // U_{Q'} = W_D * 4 (U_P U_Q - W_P W_Q)^2
	// Q.W = t17 // W_{Q'} = U_D * 4 (W_P U_Q - U_P W_Q)^2
}

type montgomeryProjectivePoint struct {
	U field.Element
	W field.Element
}

func (p *montgomeryProjectivePoint) identity() *montgomeryProjectivePoint {
	p.U.One()
	p.W.Zero()
	return p
}

func (p *montgomeryProjectivePoint) conditionalSwap(other *montgomeryProjectivePoint, choice int) {
	p.U.ConditionalSwap(&other.U, choice)
	p.W.ConditionalSwap(&other.W, choice)
}

// SetMontgomery attempts to convert a MontgomeryPoint into an EdwardsPoint
// using the supplied choice of sign for the EdwardsPoint.
func (p *EdwardsPoint) SetMontgomery(montgomeryU *MontgomeryPoint, sign uint8) (*EdwardsPoint, error) {
	// To decompress the Montgomery u coordinate to an
	// `EdwardsPoint`, we apply the birational map to obtain the
	// Edwards y coordinate, then do Edwards decompression.
	//
	// The birational map is y = (u-1)/(u+1).
	//
	// The exceptional points are the zeros of the denominator,
	// i.e., u = -1.
	//
	// But when u = -1, v^2 = u*(u^2+486662*u+1) = 486660.
	//
	// Since this is nonsquare mod p, u = -1 corresponds to a point
	// on the twist, not the curve, so we can reject it early.
	var u field.Element
	_, _ = u.SetBytes(montgomeryU[:])

	if u.Equal(&field.MinusOne) == 1 {
		return nil, errUCoordinateOnTwist
	}

	var uMinusOne, uPlusOne, y field.Element
	uMinusOne.Sub(&u, &field.One)
	uPlusOne.Add(&u, &field.One)
	uPlusOne.Invert(&uPlusOne)
	y.Mul(&uMinusOne, &uPlusOne)

	var yBytes CompressedEdwardsY
	_ = y.ToBytes(yBytes[:])
	yBytes[31] ^= sign << 7

	return p.SetCompressedY(&yBytes)
}
