// Copyright (c) 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package edwards25519

// This file contains additional functionality that is not included in the
// upstream crypto/internal/edwards25519 package.

import (
	"errors"

	"filippo.io/edwards25519/field"
)

// ExtendedCoordinates returns v in extended coordinates (X:Y:Z:T) where
// x = X/Z, y = Y/Z, and xy = T/Z as in https://eprint.iacr.org/2008/522.
func (v *Point) ExtendedCoordinates() (X, Y, Z, T *field.Element) {
	// This function is outlined to make the allocations inline in the caller
	// rather than happen on the heap. Don't change the style without making
	// sure it doesn't increase the inliner cost.
	var e [4]field.Element
	X, Y, Z, T = v.extendedCoordinates(&e)
	return
}

func (v *Point) extendedCoordinates(e *[4]field.Element) (X, Y, Z, T *field.Element) {
	checkInitialized(v)
	X = e[0].Set(&v.x)
	Y = e[1].Set(&v.y)
	Z = e[2].Set(&v.z)
	T = e[3].Set(&v.t)
	return
}

// SetExtendedCoordinates sets v = (X:Y:Z:T) in extended coordinates where
// x = X/Z, y = Y/Z, and xy = T/Z as in https://eprint.iacr.org/2008/522.
//
// If the coordinates are invalid or don't represent a valid point on the curve,
// SetExtendedCoordinates returns nil and an error and the receiver is
// unchanged. Otherwise, SetExtendedCoordinates returns v.
func (v *Point) SetExtendedCoordinates(X, Y, Z, T *field.Element) (*Point, error) {
	if !isOnCurve(X, Y, Z, T) {
		return nil, errors.New("edwards25519: invalid point coordinates")
	}
	v.x.Set(X)
	v.y.Set(Y)
	v.z.Set(Z)
	v.t.Set(T)
	return v, nil
}

func isOnCurve(X, Y, Z, T *field.Element) bool {
	var lhs, rhs field.Element
	XX := new(field.Element).Square(X)
	YY := new(field.Element).Square(Y)
	ZZ := new(field.Element).Square(Z)
	TT := new(field.Element).Square(T)
	// -x² + y² = 1 + dx²y²
	// -(X/Z)² + (Y/Z)² = 1 + d(T/Z)²
	// -X² + Y² = Z² + dT²
	lhs.Subtract(YY, XX)
	rhs.Multiply(d, TT).Add(&rhs, ZZ)
	if lhs.Equal(&rhs) != 1 {
		return false
	}
	// xy = T/Z
	// XY/Z² = T/Z
	// XY = TZ
	lhs.Multiply(X, Y)
	rhs.Multiply(T, Z)
	return lhs.Equal(&rhs) == 1
}

// BytesMontgomery converts v to a point on the birationally-equivalent
// Curve25519 Montgomery curve, and returns its canonical 32 bytes encoding
// according to RFC 7748.
//
// Note that BytesMontgomery only encodes the u-coordinate, so v and -v encode
// to the same value. If v is the identity point, BytesMontgomery returns 32
// zero bytes, analogously to the X25519 function.
//
// The lack of an inverse operation (such as SetMontgomeryBytes) is deliberate:
// while every valid edwards25519 point has a unique u-coordinate Montgomery
// encoding, X25519 accepts inputs on the quadratic twist, which don't correspond
// to any edwards25519 point, and every other X25519 input corresponds to two
// edwards25519 points.
func (v *Point) BytesMontgomery() []byte {
	// This function is outlined to make the allocations inline in the caller
	// rather than happen on the heap.
	var buf [32]byte
	return v.bytesMontgomery(&buf)
}

func (v *Point) bytesMontgomery(buf *[32]byte) []byte {
	checkInitialized(v)

	// RFC 7748, Section 4.1 provides the bilinear map to calculate the
	// Montgomery u-coordinate
	//
	//              u = (1 + y) / (1 - y)
	//
	// where y = Y / Z.

	var y, recip, u field.Element

	y.Multiply(&v.y, y.Invert(&v.z))        // y = Y / Z
	recip.Invert(recip.Subtract(feOne, &y)) // r = 1/(1 - y)
	u.Multiply(u.Add(feOne, &y), &recip)    // u = (1 + y)*r

	return copyFieldElement(buf, &u)
}

// MultByCofactor sets v = 8 * p, and returns v.
func (v *Point) MultByCofactor(p *Point) *Point {
	checkInitialized(p)
	result := projP1xP1{}
	pp := (&projP2{}).FromP3(p)
	result.Double(pp)
	pp.FromP1xP1(&result)
	result.Double(pp)
	pp.FromP1xP1(&result)
	result.Double(pp)
	return v.fromP1xP1(&result)
}

// Given k > 0, set s = s**(2*i).
func (s *Scalar) pow2k(k int) {
	for i := 0; i < k; i++ {
		s.Multiply(s, s)
	}
}

// Invert sets s to the inverse of a nonzero scalar v, and returns s.
//
// If t is zero, Invert returns zero.
func (s *Scalar) Invert(t *Scalar) *Scalar {
	// Uses a hardcoded sliding window of width 4.
	var table [8]Scalar
	var tt Scalar
	tt.Multiply(t, t)
	table[0] = *t
	for i := 0; i < 7; i++ {
		table[i+1].Multiply(&table[i], &tt)
	}
	// Now table = [t**1, t**3, t**5, t**7, t**9, t**11, t**13, t**15]
	// so t**k = t[k/2] for odd k

	// To compute the sliding window digits, use the following Sage script:

	// sage: import itertools
	// sage: def sliding_window(w,k):
	// ....:     digits = []
	// ....:     while k > 0:
	// ....:         if k % 2 == 1:
	// ....:             kmod = k % (2**w)
	// ....:             digits.append(kmod)
	// ....:             k = k - kmod
	// ....:         else:
	// ....:             digits.append(0)
	// ....:         k = k // 2
	// ....:     return digits

	// Now we can compute s roughly as follows:

	// sage: s = 1
	// sage: for coeff in reversed(sliding_window(4,l-2)):
	// ....:     s = s*s
	// ....:     if coeff > 0 :
	// ....:         s = s*t**coeff

	// This works on one bit at a time, with many runs of zeros.
	// The digits can be collapsed into [(count, coeff)] as follows:

	// sage: [(len(list(group)),d) for d,group in itertools.groupby(sliding_window(4,l-2))]

	// Entries of the form (k, 0) turn into pow2k(k)
	// Entries of the form (1, coeff) turn into a squaring and then a table lookup.
	// We can fold the squaring into the previous pow2k(k) as pow2k(k+1).

	*s = table[1/2]
	s.pow2k(127 + 1)
	s.Multiply(s, &table[1/2])
	s.pow2k(4 + 1)
	s.Multiply(s, &table[9/2])
	s.pow2k(3 + 1)
	s.Multiply(s, &table[11/2])
	s.pow2k(3 + 1)
	s.Multiply(s, &table[13/2])
	s.pow2k(3 + 1)
	s.Multiply(s, &table[15/2])
	s.pow2k(4 + 1)
	s.Multiply(s, &table[7/2])
	s.pow2k(4 + 1)
	s.Multiply(s, &table[15/2])
	s.pow2k(3 + 1)
	s.Multiply(s, &table[5/2])
	s.pow2k(3 + 1)
	s.Multiply(s, &table[1/2])
	s.pow2k(4 + 1)
	s.Multiply(s, &table[15/2])
	s.pow2k(4 + 1)
	s.Multiply(s, &table[15/2])
	s.pow2k(4 + 1)
	s.Multiply(s, &table[7/2])
	s.pow2k(3 + 1)
	s.Multiply(s, &table[3/2])
	s.pow2k(4 + 1)
	s.Multiply(s, &table[11/2])
	s.pow2k(5 + 1)
	s.Multiply(s, &table[11/2])
	s.pow2k(9 + 1)
	s.Multiply(s, &table[9/2])
	s.pow2k(3 + 1)
	s.Multiply(s, &table[3/2])
	s.pow2k(4 + 1)
	s.Multiply(s, &table[3/2])
	s.pow2k(4 + 1)
	s.Multiply(s, &table[3/2])
	s.pow2k(4 + 1)
	s.Multiply(s, &table[9/2])
	s.pow2k(3 + 1)
	s.Multiply(s, &table[7/2])
	s.pow2k(3 + 1)
	s.Multiply(s, &table[3/2])
	s.pow2k(3 + 1)
	s.Multiply(s, &table[13/2])
	s.pow2k(3 + 1)
	s.Multiply(s, &table[7/2])
	s.pow2k(4 + 1)
	s.Multiply(s, &table[9/2])
	s.pow2k(3 + 1)
	s.Multiply(s, &table[15/2])
	s.pow2k(4 + 1)
	s.Multiply(s, &table[11/2])

	return s
}

// MultiScalarMult sets v = sum(scalars[i] * points[i]), and returns v.
//
// Execution time depends only on the lengths of the two slices, which must match.
func (v *Point) MultiScalarMult(scalars []*Scalar, points []*Point) *Point {
	if len(scalars) != len(points) {
		panic("edwards25519: called MultiScalarMult with different size inputs")
	}
	checkInitialized(points...)

	// Proceed as in the single-base case, but share doublings
	// between each point in the multiscalar equation.

	// Build lookup tables for each point
	tables := make([]projLookupTable, len(points))
	for i := range tables {
		tables[i].FromP3(points[i])
	}
	// Compute signed radix-16 digits for each scalar
	digits := make([][64]int8, len(scalars))
	for i := range digits {
		digits[i] = scalars[i].signedRadix16()
	}

	// Unwrap first loop iteration to save computing 16*identity
	multiple := &projCached{}
	tmp1 := &projP1xP1{}
	tmp2 := &projP2{}
	// Lookup-and-add the appropriate multiple of each input point
	for j := range tables {
		tables[j].SelectInto(multiple, digits[j][63])
		tmp1.Add(v, multiple) // tmp1 = v + x_(j,63)*Q in P1xP1 coords
		v.fromP1xP1(tmp1)     // update v
	}
	tmp2.FromP3(v) // set up tmp2 = v in P2 coords for next iteration
	for i := 62; i >= 0; i-- {
		tmp1.Double(tmp2)    // tmp1 =  2*(prev) in P1xP1 coords
		tmp2.FromP1xP1(tmp1) // tmp2 =  2*(prev) in P2 coords
		tmp1.Double(tmp2)    // tmp1 =  4*(prev) in P1xP1 coords
		tmp2.FromP1xP1(tmp1) // tmp2 =  4*(prev) in P2 coords
		tmp1.Double(tmp2)    // tmp1 =  8*(prev) in P1xP1 coords
		tmp2.FromP1xP1(tmp1) // tmp2 =  8*(prev) in P2 coords
		tmp1.Double(tmp2)    // tmp1 = 16*(prev) in P1xP1 coords
		v.fromP1xP1(tmp1)    //    v = 16*(prev) in P3 coords
		// Lookup-and-add the appropriate multiple of each input point
		for j := range tables {
			tables[j].SelectInto(multiple, digits[j][i])
			tmp1.Add(v, multiple) // tmp1 = v + x_(j,i)*Q in P1xP1 coords
			v.fromP1xP1(tmp1)     // update v
		}
		tmp2.FromP3(v) // set up tmp2 = v in P2 coords for next iteration
	}
	return v
}

// VarTimeMultiScalarMult sets v = sum(scalars[i] * points[i]), and returns v.
//
// Execution time depends on the inputs.
func (v *Point) VarTimeMultiScalarMult(scalars []*Scalar, points []*Point) *Point {
	if len(scalars) != len(points) {
		panic("edwards25519: called VarTimeMultiScalarMult with different size inputs")
	}
	checkInitialized(points...)

	// Generalize double-base NAF computation to arbitrary sizes.
	// Here all the points are dynamic, so we only use the smaller
	// tables.

	// Build lookup tables for each point
	tables := make([]nafLookupTable5, len(points))
	for i := range tables {
		tables[i].FromP3(points[i])
	}
	// Compute a NAF for each scalar
	nafs := make([][256]int8, len(scalars))
	for i := range nafs {
		nafs[i] = scalars[i].nonAdjacentForm(5)
	}

	multiple := &projCached{}
	tmp1 := &projP1xP1{}
	tmp2 := &projP2{}
	tmp2.Zero()

	// Move from high to low bits, doubling the accumulator
	// at each iteration and checking whether there is a nonzero
	// coefficient to look up a multiple of.
	//
	// Skip trying to find the first nonzero coefficent, because
	// searching might be more work than a few extra doublings.
	for i := 255; i >= 0; i-- {
		tmp1.Double(tmp2)

		for j := range nafs {
			if nafs[j][i] > 0 {
				v.fromP1xP1(tmp1)
				tables[j].SelectInto(multiple, nafs[j][i])
				tmp1.Add(v, multiple)
			} else if nafs[j][i] < 0 {
				v.fromP1xP1(tmp1)
				tables[j].SelectInto(multiple, -nafs[j][i])
				tmp1.Sub(v, multiple)
			}
		}

		tmp2.FromP1xP1(tmp1)
	}

	v.fromP2(tmp2)
	return v
}
