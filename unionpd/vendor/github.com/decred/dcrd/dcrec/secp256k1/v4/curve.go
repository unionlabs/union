// Copyright (c) 2015-2022 The Decred developers
// Copyright 2013-2014 The btcsuite developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package secp256k1

import (
	"encoding/hex"
	"math/bits"
)

// References:
//   [SECG]: Recommended Elliptic Curve Domain Parameters
//     https://www.secg.org/sec2-v2.pdf
//
//   [GECC]: Guide to Elliptic Curve Cryptography (Hankerson, Menezes, Vanstone)
//
//   [BRID]: On Binary Representations of Integers with Digits -1, 0, 1
//           (Prodinger, Helmut)
//
//   [STWS]: Secure-TWS: Authenticating Node to Multi-user Communication in
//           Shared Sensor Networks (Oliveira, Leonardo B. et al)

// All group operations are performed using Jacobian coordinates.  For a given
// (x, y) position on the curve, the Jacobian coordinates are (x1, y1, z1)
// where x = x1/z1^2 and y = y1/z1^3.

// hexToFieldVal converts the passed hex string into a FieldVal and will panic
// if there is an error.  This is only provided for the hard-coded constants so
// errors in the source code can be detected. It will only (and must only) be
// called with hard-coded values.
func hexToFieldVal(s string) *FieldVal {
	b, err := hex.DecodeString(s)
	if err != nil {
		panic("invalid hex in source file: " + s)
	}
	var f FieldVal
	if overflow := f.SetByteSlice(b); overflow {
		panic("hex in source file overflows mod P: " + s)
	}
	return &f
}

// hexToModNScalar converts the passed hex string into a ModNScalar and will
// panic if there is an error.  This is only provided for the hard-coded
// constants so errors in the source code can be detected. It will only (and
// must only) be called with hard-coded values.
func hexToModNScalar(s string) *ModNScalar {
	var isNegative bool
	if len(s) > 0 && s[0] == '-' {
		isNegative = true
		s = s[1:]
	}
	if len(s)%2 != 0 {
		s = "0" + s
	}
	b, err := hex.DecodeString(s)
	if err != nil {
		panic("invalid hex in source file: " + s)
	}
	var scalar ModNScalar
	if overflow := scalar.SetByteSlice(b); overflow {
		panic("hex in source file overflows mod N scalar: " + s)
	}
	if isNegative {
		scalar.Negate()
	}
	return &scalar
}

var (
	// The following constants are used to accelerate scalar point
	// multiplication through the use of the endomorphism:
	//
	// φ(Q) ⟼ λ*Q = (β*Q.x mod p, Q.y)
	//
	// See the code in the deriveEndomorphismParams function in genprecomps.go
	// for details on their derivation.
	//
	// Additionally, see the scalar multiplication function in this file for
	// details on how they are used.
	endoNegLambda = hexToModNScalar("-5363ad4cc05c30e0a5261c028812645a122e22ea20816678df02967c1b23bd72")
	endoBeta      = hexToFieldVal("7ae96a2b657c07106e64479eac3434e99cf0497512f58995c1396c28719501ee")
	endoNegB1     = hexToModNScalar("e4437ed6010e88286f547fa90abfe4c3")
	endoNegB2     = hexToModNScalar("-3086d221a7d46bcde86c90e49284eb15")
	endoZ1        = hexToModNScalar("3086d221a7d46bcde86c90e49284eb153daa8a1471e8ca7f")
	endoZ2        = hexToModNScalar("e4437ed6010e88286f547fa90abfe4c4221208ac9df506c6")

	// Alternatively, the following parameters are valid as well, however,
	// benchmarks show them to be about 2% slower in practice.
	// endoNegLambda = hexToModNScalar("-ac9c52b33fa3cf1f5ad9e3fd77ed9ba4a880b9fc8ec739c2e0cfc810b51283ce")
	// endoBeta      = hexToFieldVal("851695d49a83f8ef919bb86153cbcb16630fb68aed0a766a3ec693d68e6afa40")
	// endoNegB1     = hexToModNScalar("3086d221a7d46bcde86c90e49284eb15")
	// endoNegB2     = hexToModNScalar("-114ca50f7a8e2f3f657c1108d9d44cfd8")
	// endoZ1        = hexToModNScalar("114ca50f7a8e2f3f657c1108d9d44cfd95fbc92c10fddd145")
	// endoZ2        = hexToModNScalar("3086d221a7d46bcde86c90e49284eb153daa8a1471e8ca7f")
)

// JacobianPoint is an element of the group formed by the secp256k1 curve in
// Jacobian projective coordinates and thus represents a point on the curve.
type JacobianPoint struct {
	// The X coordinate in Jacobian projective coordinates.  The affine point is
	// X/z^2.
	X FieldVal

	// The Y coordinate in Jacobian projective coordinates.  The affine point is
	// Y/z^3.
	Y FieldVal

	// The Z coordinate in Jacobian projective coordinates.
	Z FieldVal
}

// MakeJacobianPoint returns a Jacobian point with the provided X, Y, and Z
// coordinates.
func MakeJacobianPoint(x, y, z *FieldVal) JacobianPoint {
	var p JacobianPoint
	p.X.Set(x)
	p.Y.Set(y)
	p.Z.Set(z)
	return p
}

// Set sets the Jacobian point to the provided point.
func (p *JacobianPoint) Set(other *JacobianPoint) {
	p.X.Set(&other.X)
	p.Y.Set(&other.Y)
	p.Z.Set(&other.Z)
}

// ToAffine reduces the Z value of the existing point to 1 effectively
// making it an affine coordinate in constant time.  The point will be
// normalized.
func (p *JacobianPoint) ToAffine() {
	// Inversions are expensive and both point addition and point doubling
	// are faster when working with points that have a z value of one.  So,
	// if the point needs to be converted to affine, go ahead and normalize
	// the point itself at the same time as the calculation is the same.
	var zInv, tempZ FieldVal
	zInv.Set(&p.Z).Inverse()  // zInv = Z^-1
	tempZ.SquareVal(&zInv)    // tempZ = Z^-2
	p.X.Mul(&tempZ)           // X = X/Z^2 (mag: 1)
	p.Y.Mul(tempZ.Mul(&zInv)) // Y = Y/Z^3 (mag: 1)
	p.Z.SetInt(1)             // Z = 1 (mag: 1)

	// Normalize the x and y values.
	p.X.Normalize()
	p.Y.Normalize()
}

// addZ1AndZ2EqualsOne adds two Jacobian points that are already known to have
// z values of 1 and stores the result in the provided result param.  That is to
// say result = p1 + p2.  It performs faster addition than the generic add
// routine since less arithmetic is needed due to the ability to avoid the z
// value multiplications.
//
// NOTE: The points must be normalized for this function to return the correct
// result.  The resulting point will be normalized.
func addZ1AndZ2EqualsOne(p1, p2, result *JacobianPoint) {
	// To compute the point addition efficiently, this implementation splits
	// the equation into intermediate elements which are used to minimize
	// the number of field multiplications using the method shown at:
	// https://hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#addition-mmadd-2007-bl
	//
	// In particular it performs the calculations using the following:
	// H = X2-X1, HH = H^2, I = 4*HH, J = H*I, r = 2*(Y2-Y1), V = X1*I
	// X3 = r^2-J-2*V, Y3 = r*(V-X3)-2*Y1*J, Z3 = 2*H
	//
	// This results in a cost of 4 field multiplications, 2 field squarings,
	// 6 field additions, and 5 integer multiplications.
	x1, y1 := &p1.X, &p1.Y
	x2, y2 := &p2.X, &p2.Y
	x3, y3, z3 := &result.X, &result.Y, &result.Z

	// When the x coordinates are the same for two points on the curve, the
	// y coordinates either must be the same, in which case it is point
	// doubling, or they are opposite and the result is the point at
	// infinity per the group law for elliptic curve cryptography.
	if x1.Equals(x2) {
		if y1.Equals(y2) {
			// Since x1 == x2 and y1 == y2, point doubling must be
			// done, otherwise the addition would end up dividing
			// by zero.
			DoubleNonConst(p1, result)
			return
		}

		// Since x1 == x2 and y1 == -y2, the sum is the point at
		// infinity per the group law.
		x3.SetInt(0)
		y3.SetInt(0)
		z3.SetInt(0)
		return
	}

	// Calculate X3, Y3, and Z3 according to the intermediate elements
	// breakdown above.
	var h, i, j, r, v FieldVal
	var negJ, neg2V, negX3 FieldVal
	h.Set(x1).Negate(1).Add(x2)                // H = X2-X1 (mag: 3)
	i.SquareVal(&h).MulInt(4)                  // I = 4*H^2 (mag: 4)
	j.Mul2(&h, &i)                             // J = H*I (mag: 1)
	r.Set(y1).Negate(1).Add(y2).MulInt(2)      // r = 2*(Y2-Y1) (mag: 6)
	v.Mul2(x1, &i)                             // V = X1*I (mag: 1)
	negJ.Set(&j).Negate(1)                     // negJ = -J (mag: 2)
	neg2V.Set(&v).MulInt(2).Negate(2)          // neg2V = -(2*V) (mag: 3)
	x3.Set(&r).Square().Add(&negJ).Add(&neg2V) // X3 = r^2-J-2*V (mag: 6)
	negX3.Set(x3).Negate(6)                    // negX3 = -X3 (mag: 7)
	j.Mul(y1).MulInt(2).Negate(2)              // J = -(2*Y1*J) (mag: 3)
	y3.Set(&v).Add(&negX3).Mul(&r).Add(&j)     // Y3 = r*(V-X3)-2*Y1*J (mag: 4)
	z3.Set(&h).MulInt(2)                       // Z3 = 2*H (mag: 6)

	// Normalize the resulting field values as needed.
	x3.Normalize()
	y3.Normalize()
	z3.Normalize()
}

// addZ1EqualsZ2 adds two Jacobian points that are already known to have the
// same z value and stores the result in the provided result param.  That is to
// say result = p1 + p2.  It performs faster addition than the generic add
// routine since less arithmetic is needed due to the known equivalence.
//
// NOTE: The points must be normalized for this function to return the correct
// result.  The resulting point will be normalized.
func addZ1EqualsZ2(p1, p2, result *JacobianPoint) {
	// To compute the point addition efficiently, this implementation splits
	// the equation into intermediate elements which are used to minimize
	// the number of field multiplications using a slightly modified version
	// of the method shown at:
	// https://hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#addition-zadd-2007-m
	//
	// In particular it performs the calculations using the following:
	// A = X2-X1, B = A^2, C=Y2-Y1, D = C^2, E = X1*B, F = X2*B
	// X3 = D-E-F, Y3 = C*(E-X3)-Y1*(F-E), Z3 = Z1*A
	//
	// This results in a cost of 5 field multiplications, 2 field squarings,
	// 9 field additions, and 0 integer multiplications.
	x1, y1, z1 := &p1.X, &p1.Y, &p1.Z
	x2, y2 := &p2.X, &p2.Y
	x3, y3, z3 := &result.X, &result.Y, &result.Z

	// When the x coordinates are the same for two points on the curve, the
	// y coordinates either must be the same, in which case it is point
	// doubling, or they are opposite and the result is the point at
	// infinity per the group law for elliptic curve cryptography.
	if x1.Equals(x2) {
		if y1.Equals(y2) {
			// Since x1 == x2 and y1 == y2, point doubling must be
			// done, otherwise the addition would end up dividing
			// by zero.
			DoubleNonConst(p1, result)
			return
		}

		// Since x1 == x2 and y1 == -y2, the sum is the point at
		// infinity per the group law.
		x3.SetInt(0)
		y3.SetInt(0)
		z3.SetInt(0)
		return
	}

	// Calculate X3, Y3, and Z3 according to the intermediate elements
	// breakdown above.
	var a, b, c, d, e, f FieldVal
	var negX1, negY1, negE, negX3 FieldVal
	negX1.Set(x1).Negate(1)                // negX1 = -X1 (mag: 2)
	negY1.Set(y1).Negate(1)                // negY1 = -Y1 (mag: 2)
	a.Set(&negX1).Add(x2)                  // A = X2-X1 (mag: 3)
	b.SquareVal(&a)                        // B = A^2 (mag: 1)
	c.Set(&negY1).Add(y2)                  // C = Y2-Y1 (mag: 3)
	d.SquareVal(&c)                        // D = C^2 (mag: 1)
	e.Mul2(x1, &b)                         // E = X1*B (mag: 1)
	negE.Set(&e).Negate(1)                 // negE = -E (mag: 2)
	f.Mul2(x2, &b)                         // F = X2*B (mag: 1)
	x3.Add2(&e, &f).Negate(2).Add(&d)      // X3 = D-E-F (mag: 4)
	negX3.Set(x3).Negate(4)                // negX3 = -X3 (mag: 5)
	y3.Set(y1).Mul(f.Add(&negE)).Negate(1) // Y3 = -(Y1*(F-E)) (mag: 2)
	y3.Add(e.Add(&negX3).Mul(&c))          // Y3 = C*(E-X3)+Y3 (mag: 3)
	z3.Mul2(z1, &a)                        // Z3 = Z1*A (mag: 1)

	// Normalize the resulting field values as needed.
	x3.Normalize()
	y3.Normalize()
	z3.Normalize()
}

// addZ2EqualsOne adds two Jacobian points when the second point is already
// known to have a z value of 1 (and the z value for the first point is not 1)
// and stores the result in the provided result param.  That is to say result =
// p1 + p2.  It performs faster addition than the generic add routine since
// less arithmetic is needed due to the ability to avoid multiplications by the
// second point's z value.
//
// NOTE: The points must be normalized for this function to return the correct
// result.  The resulting point will be normalized.
func addZ2EqualsOne(p1, p2, result *JacobianPoint) {
	// To compute the point addition efficiently, this implementation splits
	// the equation into intermediate elements which are used to minimize
	// the number of field multiplications using the method shown at:
	// https://hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#addition-madd-2007-bl
	//
	// In particular it performs the calculations using the following:
	// Z1Z1 = Z1^2, U2 = X2*Z1Z1, S2 = Y2*Z1*Z1Z1, H = U2-X1, HH = H^2,
	// I = 4*HH, J = H*I, r = 2*(S2-Y1), V = X1*I
	// X3 = r^2-J-2*V, Y3 = r*(V-X3)-2*Y1*J, Z3 = (Z1+H)^2-Z1Z1-HH
	//
	// This results in a cost of 7 field multiplications, 4 field squarings,
	// 9 field additions, and 4 integer multiplications.
	x1, y1, z1 := &p1.X, &p1.Y, &p1.Z
	x2, y2 := &p2.X, &p2.Y
	x3, y3, z3 := &result.X, &result.Y, &result.Z

	// When the x coordinates are the same for two points on the curve, the
	// y coordinates either must be the same, in which case it is point
	// doubling, or they are opposite and the result is the point at
	// infinity per the group law for elliptic curve cryptography.  Since
	// any number of Jacobian coordinates can represent the same affine
	// point, the x and y values need to be converted to like terms.  Due to
	// the assumption made for this function that the second point has a z
	// value of 1 (z2=1), the first point is already "converted".
	var z1z1, u2, s2 FieldVal
	z1z1.SquareVal(z1)                        // Z1Z1 = Z1^2 (mag: 1)
	u2.Set(x2).Mul(&z1z1).Normalize()         // U2 = X2*Z1Z1 (mag: 1)
	s2.Set(y2).Mul(&z1z1).Mul(z1).Normalize() // S2 = Y2*Z1*Z1Z1 (mag: 1)
	if x1.Equals(&u2) {
		if y1.Equals(&s2) {
			// Since x1 == x2 and y1 == y2, point doubling must be
			// done, otherwise the addition would end up dividing
			// by zero.
			DoubleNonConst(p1, result)
			return
		}

		// Since x1 == x2 and y1 == -y2, the sum is the point at
		// infinity per the group law.
		x3.SetInt(0)
		y3.SetInt(0)
		z3.SetInt(0)
		return
	}

	// Calculate X3, Y3, and Z3 according to the intermediate elements
	// breakdown above.
	var h, hh, i, j, r, rr, v FieldVal
	var negX1, negY1, negX3 FieldVal
	negX1.Set(x1).Negate(1)                // negX1 = -X1 (mag: 2)
	h.Add2(&u2, &negX1)                    // H = U2-X1 (mag: 3)
	hh.SquareVal(&h)                       // HH = H^2 (mag: 1)
	i.Set(&hh).MulInt(4)                   // I = 4 * HH (mag: 4)
	j.Mul2(&h, &i)                         // J = H*I (mag: 1)
	negY1.Set(y1).Negate(1)                // negY1 = -Y1 (mag: 2)
	r.Set(&s2).Add(&negY1).MulInt(2)       // r = 2*(S2-Y1) (mag: 6)
	rr.SquareVal(&r)                       // rr = r^2 (mag: 1)
	v.Mul2(x1, &i)                         // V = X1*I (mag: 1)
	x3.Set(&v).MulInt(2).Add(&j).Negate(3) // X3 = -(J+2*V) (mag: 4)
	x3.Add(&rr)                            // X3 = r^2+X3 (mag: 5)
	negX3.Set(x3).Negate(5)                // negX3 = -X3 (mag: 6)
	y3.Set(y1).Mul(&j).MulInt(2).Negate(2) // Y3 = -(2*Y1*J) (mag: 3)
	y3.Add(v.Add(&negX3).Mul(&r))          // Y3 = r*(V-X3)+Y3 (mag: 4)
	z3.Add2(z1, &h).Square()               // Z3 = (Z1+H)^2 (mag: 1)
	z3.Add(z1z1.Add(&hh).Negate(2))        // Z3 = Z3-(Z1Z1+HH) (mag: 4)

	// Normalize the resulting field values as needed.
	x3.Normalize()
	y3.Normalize()
	z3.Normalize()
}

// addGeneric adds two Jacobian points without any assumptions about the z
// values of the two points and stores the result in the provided result param.
// That is to say result = p1 + p2.  It is the slowest of the add routines due
// to requiring the most arithmetic.
//
// NOTE: The points must be normalized for this function to return the correct
// result.  The resulting point will be normalized.
func addGeneric(p1, p2, result *JacobianPoint) {
	// To compute the point addition efficiently, this implementation splits
	// the equation into intermediate elements which are used to minimize
	// the number of field multiplications using the method shown at:
	// https://hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#addition-add-2007-bl
	//
	// In particular it performs the calculations using the following:
	// Z1Z1 = Z1^2, Z2Z2 = Z2^2, U1 = X1*Z2Z2, U2 = X2*Z1Z1, S1 = Y1*Z2*Z2Z2
	// S2 = Y2*Z1*Z1Z1, H = U2-U1, I = (2*H)^2, J = H*I, r = 2*(S2-S1)
	// V = U1*I
	// X3 = r^2-J-2*V, Y3 = r*(V-X3)-2*S1*J, Z3 = ((Z1+Z2)^2-Z1Z1-Z2Z2)*H
	//
	// This results in a cost of 11 field multiplications, 5 field squarings,
	// 9 field additions, and 4 integer multiplications.
	x1, y1, z1 := &p1.X, &p1.Y, &p1.Z
	x2, y2, z2 := &p2.X, &p2.Y, &p2.Z
	x3, y3, z3 := &result.X, &result.Y, &result.Z

	// When the x coordinates are the same for two points on the curve, the
	// y coordinates either must be the same, in which case it is point
	// doubling, or they are opposite and the result is the point at
	// infinity.  Since any number of Jacobian coordinates can represent the
	// same affine point, the x and y values need to be converted to like
	// terms.
	var z1z1, z2z2, u1, u2, s1, s2 FieldVal
	z1z1.SquareVal(z1)                        // Z1Z1 = Z1^2 (mag: 1)
	z2z2.SquareVal(z2)                        // Z2Z2 = Z2^2 (mag: 1)
	u1.Set(x1).Mul(&z2z2).Normalize()         // U1 = X1*Z2Z2 (mag: 1)
	u2.Set(x2).Mul(&z1z1).Normalize()         // U2 = X2*Z1Z1 (mag: 1)
	s1.Set(y1).Mul(&z2z2).Mul(z2).Normalize() // S1 = Y1*Z2*Z2Z2 (mag: 1)
	s2.Set(y2).Mul(&z1z1).Mul(z1).Normalize() // S2 = Y2*Z1*Z1Z1 (mag: 1)
	if u1.Equals(&u2) {
		if s1.Equals(&s2) {
			// Since x1 == x2 and y1 == y2, point doubling must be
			// done, otherwise the addition would end up dividing
			// by zero.
			DoubleNonConst(p1, result)
			return
		}

		// Since x1 == x2 and y1 == -y2, the sum is the point at
		// infinity per the group law.
		x3.SetInt(0)
		y3.SetInt(0)
		z3.SetInt(0)
		return
	}

	// Calculate X3, Y3, and Z3 according to the intermediate elements
	// breakdown above.
	var h, i, j, r, rr, v FieldVal
	var negU1, negS1, negX3 FieldVal
	negU1.Set(&u1).Negate(1)               // negU1 = -U1 (mag: 2)
	h.Add2(&u2, &negU1)                    // H = U2-U1 (mag: 3)
	i.Set(&h).MulInt(2).Square()           // I = (2*H)^2 (mag: 1)
	j.Mul2(&h, &i)                         // J = H*I (mag: 1)
	negS1.Set(&s1).Negate(1)               // negS1 = -S1 (mag: 2)
	r.Set(&s2).Add(&negS1).MulInt(2)       // r = 2*(S2-S1) (mag: 6)
	rr.SquareVal(&r)                       // rr = r^2 (mag: 1)
	v.Mul2(&u1, &i)                        // V = U1*I (mag: 1)
	x3.Set(&v).MulInt(2).Add(&j).Negate(3) // X3 = -(J+2*V) (mag: 4)
	x3.Add(&rr)                            // X3 = r^2+X3 (mag: 5)
	negX3.Set(x3).Negate(5)                // negX3 = -X3 (mag: 6)
	y3.Mul2(&s1, &j).MulInt(2).Negate(2)   // Y3 = -(2*S1*J) (mag: 3)
	y3.Add(v.Add(&negX3).Mul(&r))          // Y3 = r*(V-X3)+Y3 (mag: 4)
	z3.Add2(z1, z2).Square()               // Z3 = (Z1+Z2)^2 (mag: 1)
	z3.Add(z1z1.Add(&z2z2).Negate(2))      // Z3 = Z3-(Z1Z1+Z2Z2) (mag: 4)
	z3.Mul(&h)                             // Z3 = Z3*H (mag: 1)

	// Normalize the resulting field values as needed.
	x3.Normalize()
	y3.Normalize()
	z3.Normalize()
}

// AddNonConst adds the passed Jacobian points together and stores the result in
// the provided result param in *non-constant* time.
//
// NOTE: The points must be normalized for this function to return the correct
// result.  The resulting point will be normalized.
func AddNonConst(p1, p2, result *JacobianPoint) {
	// The point at infinity is the identity according to the group law for
	// elliptic curve cryptography.  Thus, ∞ + P = P and P + ∞ = P.
	if (p1.X.IsZero() && p1.Y.IsZero()) || p1.Z.IsZero() {
		result.Set(p2)
		return
	}
	if (p2.X.IsZero() && p2.Y.IsZero()) || p2.Z.IsZero() {
		result.Set(p1)
		return
	}

	// Faster point addition can be achieved when certain assumptions are
	// met.  For example, when both points have the same z value, arithmetic
	// on the z values can be avoided.  This section thus checks for these
	// conditions and calls an appropriate add function which is accelerated
	// by using those assumptions.
	isZ1One := p1.Z.IsOne()
	isZ2One := p2.Z.IsOne()
	switch {
	case isZ1One && isZ2One:
		addZ1AndZ2EqualsOne(p1, p2, result)
		return
	case p1.Z.Equals(&p2.Z):
		addZ1EqualsZ2(p1, p2, result)
		return
	case isZ2One:
		addZ2EqualsOne(p1, p2, result)
		return
	}

	// None of the above assumptions are true, so fall back to generic
	// point addition.
	addGeneric(p1, p2, result)
}

// doubleZ1EqualsOne performs point doubling on the passed Jacobian point when
// the point is already known to have a z value of 1 and stores the result in
// the provided result param.  That is to say result = 2*p.  It performs faster
// point doubling than the generic routine since less arithmetic is needed due
// to the ability to avoid multiplication by the z value.
//
// NOTE: The resulting point will be normalized.
func doubleZ1EqualsOne(p, result *JacobianPoint) {
	// This function uses the assumptions that z1 is 1, thus the point
	// doubling formulas reduce to:
	//
	// X3 = (3*X1^2)^2 - 8*X1*Y1^2
	// Y3 = (3*X1^2)*(4*X1*Y1^2 - X3) - 8*Y1^4
	// Z3 = 2*Y1
	//
	// To compute the above efficiently, this implementation splits the
	// equation into intermediate elements which are used to minimize the
	// number of field multiplications in favor of field squarings which
	// are roughly 35% faster than field multiplications with the current
	// implementation at the time this was written.
	//
	// This uses a slightly modified version of the method shown at:
	// https://hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#doubling-mdbl-2007-bl
	//
	// In particular it performs the calculations using the following:
	// A = X1^2, B = Y1^2, C = B^2, D = 2*((X1+B)^2-A-C)
	// E = 3*A, F = E^2, X3 = F-2*D, Y3 = E*(D-X3)-8*C
	// Z3 = 2*Y1
	//
	// This results in a cost of 1 field multiplication, 5 field squarings,
	// 6 field additions, and 5 integer multiplications.
	x1, y1 := &p.X, &p.Y
	x3, y3, z3 := &result.X, &result.Y, &result.Z
	var a, b, c, d, e, f FieldVal
	z3.Set(y1).MulInt(2)                     // Z3 = 2*Y1 (mag: 2)
	a.SquareVal(x1)                          // A = X1^2 (mag: 1)
	b.SquareVal(y1)                          // B = Y1^2 (mag: 1)
	c.SquareVal(&b)                          // C = B^2 (mag: 1)
	b.Add(x1).Square()                       // B = (X1+B)^2 (mag: 1)
	d.Set(&a).Add(&c).Negate(2)              // D = -(A+C) (mag: 3)
	d.Add(&b).MulInt(2)                      // D = 2*(B+D)(mag: 8)
	e.Set(&a).MulInt(3)                      // E = 3*A (mag: 3)
	f.SquareVal(&e)                          // F = E^2 (mag: 1)
	x3.Set(&d).MulInt(2).Negate(16)          // X3 = -(2*D) (mag: 17)
	x3.Add(&f)                               // X3 = F+X3 (mag: 18)
	f.Set(x3).Negate(18).Add(&d).Normalize() // F = D-X3 (mag: 1)
	y3.Set(&c).MulInt(8).Negate(8)           // Y3 = -(8*C) (mag: 9)
	y3.Add(f.Mul(&e))                        // Y3 = E*F+Y3 (mag: 10)

	// Normalize the resulting field values as needed.
	x3.Normalize()
	y3.Normalize()
	z3.Normalize()
}

// doubleGeneric performs point doubling on the passed Jacobian point without
// any assumptions about the z value and stores the result in the provided
// result param.  That is to say result = 2*p.  It is the slowest of the point
// doubling routines due to requiring the most arithmetic.
//
// NOTE: The resulting point will be normalized.
func doubleGeneric(p, result *JacobianPoint) {
	// Point doubling formula for Jacobian coordinates for the secp256k1
	// curve:
	//
	// X3 = (3*X1^2)^2 - 8*X1*Y1^2
	// Y3 = (3*X1^2)*(4*X1*Y1^2 - X3) - 8*Y1^4
	// Z3 = 2*Y1*Z1
	//
	// To compute the above efficiently, this implementation splits the
	// equation into intermediate elements which are used to minimize the
	// number of field multiplications in favor of field squarings which
	// are roughly 35% faster than field multiplications with the current
	// implementation at the time this was written.
	//
	// This uses a slightly modified version of the method shown at:
	// https://hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#doubling-dbl-2009-l
	//
	// In particular it performs the calculations using the following:
	// A = X1^2, B = Y1^2, C = B^2, D = 2*((X1+B)^2-A-C)
	// E = 3*A, F = E^2, X3 = F-2*D, Y3 = E*(D-X3)-8*C
	// Z3 = 2*Y1*Z1
	//
	// This results in a cost of 1 field multiplication, 5 field squarings,
	// 6 field additions, and 5 integer multiplications.
	x1, y1, z1 := &p.X, &p.Y, &p.Z
	x3, y3, z3 := &result.X, &result.Y, &result.Z
	var a, b, c, d, e, f FieldVal
	z3.Mul2(y1, z1).MulInt(2)                // Z3 = 2*Y1*Z1 (mag: 2)
	a.SquareVal(x1)                          // A = X1^2 (mag: 1)
	b.SquareVal(y1)                          // B = Y1^2 (mag: 1)
	c.SquareVal(&b)                          // C = B^2 (mag: 1)
	b.Add(x1).Square()                       // B = (X1+B)^2 (mag: 1)
	d.Set(&a).Add(&c).Negate(2)              // D = -(A+C) (mag: 3)
	d.Add(&b).MulInt(2)                      // D = 2*(B+D)(mag: 8)
	e.Set(&a).MulInt(3)                      // E = 3*A (mag: 3)
	f.SquareVal(&e)                          // F = E^2 (mag: 1)
	x3.Set(&d).MulInt(2).Negate(16)          // X3 = -(2*D) (mag: 17)
	x3.Add(&f)                               // X3 = F+X3 (mag: 18)
	f.Set(x3).Negate(18).Add(&d).Normalize() // F = D-X3 (mag: 1)
	y3.Set(&c).MulInt(8).Negate(8)           // Y3 = -(8*C) (mag: 9)
	y3.Add(f.Mul(&e))                        // Y3 = E*F+Y3 (mag: 10)

	// Normalize the resulting field values as needed.
	x3.Normalize()
	y3.Normalize()
	z3.Normalize()
}

// DoubleNonConst doubles the passed Jacobian point and stores the result in the
// provided result parameter in *non-constant* time.
//
// NOTE: The point must be normalized for this function to return the correct
// result.  The resulting point will be normalized.
func DoubleNonConst(p, result *JacobianPoint) {
	// Doubling the point at infinity is still infinity.
	if p.Y.IsZero() || p.Z.IsZero() {
		result.X.SetInt(0)
		result.Y.SetInt(0)
		result.Z.SetInt(0)
		return
	}

	// Slightly faster point doubling can be achieved when the z value is 1
	// by avoiding the multiplication on the z value.  This section calls
	// a point doubling function which is accelerated by using that
	// assumption when possible.
	if p.Z.IsOne() {
		doubleZ1EqualsOne(p, result)
		return
	}

	// Fall back to generic point doubling which works with arbitrary z
	// values.
	doubleGeneric(p, result)
}

// mulAdd64 multiplies the two passed base 2^64 digits together, adds the given
// value to the result, and returns the 128-bit result via a (hi, lo) tuple
// where the upper half of the bits are returned in hi and the lower half in lo.
func mulAdd64(digit1, digit2, m uint64) (hi, lo uint64) {
	// Note the carry on the final add is safe to discard because the maximum
	// possible value is:
	//   (2^64 - 1)(2^64 - 1) + (2^64 - 1) = 2^128 - 2^64
	// and:
	//   2^128 - 2^64 < 2^128.
	var c uint64
	hi, lo = bits.Mul64(digit1, digit2)
	lo, c = bits.Add64(lo, m, 0)
	hi, _ = bits.Add64(hi, 0, c)
	return hi, lo
}

// mulAdd64Carry multiplies the two passed base 2^64 digits together, adds both
// the given value and carry to the result, and returns the 128-bit result via a
// (hi, lo) tuple where the upper half of the bits are returned in hi and the
// lower half in lo.
func mulAdd64Carry(digit1, digit2, m, c uint64) (hi, lo uint64) {
	// Note the carry on the high order add is safe to discard because the
	// maximum possible value is:
	//   (2^64 - 1)(2^64 - 1) + 2*(2^64 - 1) = 2^128 - 1
	// and:
	//   2^128 - 1 < 2^128.
	var c2 uint64
	hi, lo = mulAdd64(digit1, digit2, m)
	lo, c2 = bits.Add64(lo, c, 0)
	hi, _ = bits.Add64(hi, 0, c2)
	return hi, lo
}

// mul512Rsh320Round computes the full 512-bit product of the two given scalars,
// right shifts the result by 320 bits, rounds to the nearest integer, and
// returns the result in constant time.
//
// Note that despite the inputs and output being mod n scalars, the 512-bit
// product is NOT reduced mod N prior to the right shift.  This is intentional
// because it is used for replacing division with multiplication and thus the
// intermediate results must be done via a field extension to a larger field.
func mul512Rsh320Round(n1, n2 *ModNScalar) ModNScalar {
	// Convert n1 and n2 to base 2^64 digits.
	n1Digit0 := uint64(n1.n[0]) | uint64(n1.n[1])<<32
	n1Digit1 := uint64(n1.n[2]) | uint64(n1.n[3])<<32
	n1Digit2 := uint64(n1.n[4]) | uint64(n1.n[5])<<32
	n1Digit3 := uint64(n1.n[6]) | uint64(n1.n[7])<<32
	n2Digit0 := uint64(n2.n[0]) | uint64(n2.n[1])<<32
	n2Digit1 := uint64(n2.n[2]) | uint64(n2.n[3])<<32
	n2Digit2 := uint64(n2.n[4]) | uint64(n2.n[5])<<32
	n2Digit3 := uint64(n2.n[6]) | uint64(n2.n[7])<<32

	// Compute the full 512-bit product n1*n2.
	var r0, r1, r2, r3, r4, r5, r6, r7, c uint64

	// Terms resulting from the product of the first digit of the second number
	// by all digits of the first number.
	//
	// Note that r0 is ignored because it is not needed to compute the higher
	// terms and it is shifted out below anyway.
	c, _ = bits.Mul64(n2Digit0, n1Digit0)
	c, r1 = mulAdd64(n2Digit0, n1Digit1, c)
	c, r2 = mulAdd64(n2Digit0, n1Digit2, c)
	r4, r3 = mulAdd64(n2Digit0, n1Digit3, c)

	// Terms resulting from the product of the second digit of the second number
	// by all digits of the first number.
	//
	// Note that r1 is ignored because it is no longer needed to compute the
	// higher terms and it is shifted out below anyway.
	c, _ = mulAdd64(n2Digit1, n1Digit0, r1)
	c, r2 = mulAdd64Carry(n2Digit1, n1Digit1, r2, c)
	c, r3 = mulAdd64Carry(n2Digit1, n1Digit2, r3, c)
	r5, r4 = mulAdd64Carry(n2Digit1, n1Digit3, r4, c)

	// Terms resulting from the product of the third digit of the second number
	// by all digits of the first number.
	//
	// Note that r2 is ignored because it is no longer needed to compute the
	// higher terms and it is shifted out below anyway.
	c, _ = mulAdd64(n2Digit2, n1Digit0, r2)
	c, r3 = mulAdd64Carry(n2Digit2, n1Digit1, r3, c)
	c, r4 = mulAdd64Carry(n2Digit2, n1Digit2, r4, c)
	r6, r5 = mulAdd64Carry(n2Digit2, n1Digit3, r5, c)

	// Terms resulting from the product of the fourth digit of the second number
	// by all digits of the first number.
	//
	// Note that r3 is ignored because it is no longer needed to compute the
	// higher terms and it is shifted out below anyway.
	c, _ = mulAdd64(n2Digit3, n1Digit0, r3)
	c, r4 = mulAdd64Carry(n2Digit3, n1Digit1, r4, c)
	c, r5 = mulAdd64Carry(n2Digit3, n1Digit2, r5, c)
	r7, r6 = mulAdd64Carry(n2Digit3, n1Digit3, r6, c)

	// At this point the upper 256 bits of the full 512-bit product n1*n2 are in
	// r4..r7 (recall the low order results were discarded as noted above).
	//
	// Right shift the result 320 bits.  Note that the MSB of r4 determines
	// whether or not to round because it is the final bit that is shifted out.
	//
	// Also, notice that r3..r7 would also ordinarily be set to 0 as well for
	// the full shift, but that is skipped since they are no longer used as
	// their values are known to be zero.
	roundBit := r4 >> 63
	r2, r1, r0 = r7, r6, r5

	// Conditionally add 1 depending on the round bit in constant time.
	r0, c = bits.Add64(r0, roundBit, 0)
	r1, c = bits.Add64(r1, 0, c)
	r2, r3 = bits.Add64(r2, 0, c)

	// Finally, convert the result to a mod n scalar.
	//
	// No modular reduction is needed because the result is guaranteed to be
	// less than the group order given the group order is > 2^255 and the
	// maximum possible value of the result is 2^192.
	var result ModNScalar
	result.n[0] = uint32(r0)
	result.n[1] = uint32(r0 >> 32)
	result.n[2] = uint32(r1)
	result.n[3] = uint32(r1 >> 32)
	result.n[4] = uint32(r2)
	result.n[5] = uint32(r2 >> 32)
	result.n[6] = uint32(r3)
	result.n[7] = uint32(r3 >> 32)
	return result
}

// splitK returns two scalars (k1 and k2) that are a balanced length-two
// representation of the provided scalar such that k ≡ k1 + k2*λ (mod N), where
// N is the secp256k1 group order.
func splitK(k *ModNScalar) (ModNScalar, ModNScalar) {
	// The ultimate goal is to decompose k into two scalars that are around
	// half the bit length of k such that the following equation is satisfied:
	//
	// k1 + k2*λ ≡ k (mod n)
	//
	// The strategy used here is based on algorithm 3.74 from [GECC] with a few
	// modifications to make use of the more efficient mod n scalar type, avoid
	// some costly long divisions, and minimize the number of calculations.
	//
	// Start by defining a function that takes a vector v = <a,b> ∈ ℤ⨯ℤ:
	//
	// f(v) = a + bλ (mod n)
	//
	// Then, find two vectors, v1 = <a1,b1>, and v2 = <a2,b2> in ℤ⨯ℤ such that:
	// 1) v1 and v2 are linearly independent
	// 2) f(v1) = f(v2) = 0
	// 3) v1 and v2 have small Euclidean norm
	//
	// The vectors that satisfy these properties are found via the Euclidean
	// algorithm and are precomputed since both n and λ are fixed values for the
	// secp256k1 curve.  See genprecomps.go for derivation details.
	//
	// Next, consider k as a vector <k, 0> in ℚ⨯ℚ and by linear algebra write:
	//
	// <k, 0> = g1*v1 + g2*v2, where g1, g2 ∈ ℚ
	//
	// Note that, per above, the components of vector v1 are a1 and b1 while the
	// components of vector v2 are a2 and b2.  Given the vectors v1 and v2 were
	// generated such that a1*b2 - a2*b1 = n, solving the equation for g1 and g2
	// yields:
	//
	// g1 = b2*k / n
	// g2 = -b1*k / n
	//
	// Observe:
	// <k, 0> = g1*v1 + g2*v2
	//        = (b2*k/n)*<a1,b1> + (-b1*k/n)*<a2,b2>              | substitute
	//        = <a1*b2*k/n, b1*b2*k/n> + <-a2*b1*k/n, -b2*b1*k/n> | scalar mul
	//        = <a1*b2*k/n - a2*b1*k/n, b1*b2*k/n - b2*b1*k/n>    | vector add
	//        = <[a1*b2*k - a2*b1*k]/n, 0>                        | simplify
	//        = <k*[a1*b2 - a2*b1]/n, 0>                          | factor out k
	//        = <k*n/n, 0>                                        | substitute
	//        = <k, 0>                                            | simplify
	//
	// Now, consider an integer-valued vector v:
	//
	// v = c1*v1 + c2*v2, where c1, c2 ∈ ℤ (mod n)
	//
	// Since vectors v1 and v2 are linearly independent and were generated such
	// that f(v1) = f(v2) = 0, all possible scalars c1 and c2 also produce a
	// vector v such that f(v) = 0.
	//
	// In other words, c1 and c2 can be any integers and the resulting
	// decomposition will still satisfy the required equation.  However, since
	// the goal is to produce a balanced decomposition that provides a
	// performance advantage by minimizing max(k1, k2), c1 and c2 need to be
	// integers close to g1 and g2, respectively, so the resulting vector v is
	// an integer-valued vector that is close to <k, 0>.
	//
	// Finally, consider the vector u:
	//
	// u  = <k, 0> - v
	//
	// It follows that f(u) = k and thus the two components of vector u satisfy
	// the required equation:
	//
	// k1 + k2*λ ≡ k (mod n)
	//
	// Choosing c1 and c2:
	// -------------------
	//
	// As mentioned above, c1 and c2 need to be integers close to g1 and g2,
	// respectively.  The algorithm in [GECC] chooses the following values:
	//
	// c1 = round(g1) = round(b2*k / n)
	// c2 = round(g2) = round(-b1*k / n)
	//
	// However, as section 3.4.2 of [STWS] notes, the aforementioned approach
	// requires costly long divisions that can be avoided by precomputing
	// rounded estimates as follows:
	//
	// t = bitlen(n) + 1
	// z1 = round(2^t * b2 / n)
	// z2 = round(2^t * -b1 / n)
	//
	// Then, use those precomputed estimates to perform a multiplication by k
	// along with a floored division by 2^t, which is a simple right shift by t:
	//
	// c1 = floor(k * z1 / 2^t) = (k * z1) >> t
	// c2 = floor(k * z2 / 2^t) = (k * z2) >> t
	//
	// Finally, round up if last bit discarded in the right shift by t is set by
	// adding 1.
	//
	// As a further optimization, rather than setting t = bitlen(n) + 1 = 257 as
	// stated by [STWS], this implementation uses a higher precision estimate of
	// t = bitlen(n) + 64 = 320 because it allows simplification of the shifts
	// in the internal calculations that are done via uint64s and also allows
	// the use of floor in the precomputations.
	//
	// Thus, the calculations this implementation uses are:
	//
	// z1 = floor(b2<<320 / n)                                     | precomputed
	// z2 = floor((-b1)<<320) / n)                                 | precomputed
	// c1 = ((k * z1) >> 320) + (((k * z1) >> 319) & 1)
	// c2 = ((k * z2) >> 320) + (((k * z2) >> 319) & 1)
	//
	// Putting it all together:
	// ------------------------
	//
	// Calculate the following vectors using the values discussed above:
	//
	// v = c1*v1 + c2*v2
	// u = <k, 0> - v
	//
	// The two components of the resulting vector v are:
	// va = c1*a1 + c2*a2
	// vb = c1*b1 + c2*b2
	//
	// Thus, the two components of the resulting vector u are:
	// k1 = k - va
	// k2 = 0 - vb = -vb
	//
	// As some final optimizations:
	//
	// 1) Note that k1 + k2*λ ≡ k (mod n) means that k1 ≡ k - k2*λ (mod n).
	//    Therefore, the computation of va can be avoided to save two
	//    field multiplications and a field addition.
	//
	// 2) Since k1 = k - k2*λ = k + k2*(-λ), an additional field negation is
	//    saved by storing and using the negative version of λ.
	//
	// 3) Since k2 = -vb = -(c1*b1 + c2*b2) = c1*(-b1) + c2*(-b2), one more
	//    field negation is saved by storing and using the negative versions of
	//    b1 and b2.
	//
	// k2 = c1*(-b1) + c2*(-b2)
	// k1 = k + k2*(-λ)
	var k1, k2 ModNScalar
	c1 := mul512Rsh320Round(k, endoZ1)
	c2 := mul512Rsh320Round(k, endoZ2)
	k2.Add2(c1.Mul(endoNegB1), c2.Mul(endoNegB2))
	k1.Mul2(&k2, endoNegLambda).Add(k)
	return k1, k2
}

// nafScalar represents a positive integer up to a maximum value of 2^256 - 1
// encoded in non-adjacent form.
//
// NAF is a signed-digit representation where each digit can be +1, 0, or -1.
//
// In order to efficiently encode that information, this type uses two arrays, a
// "positive" array where set bits represent the +1 signed digits and a
// "negative" array where set bits represent the -1 signed digits.  0 is
// represented by neither array having a bit set in that position.
//
// The Pos and Neg methods return the aforementioned positive and negative
// arrays, respectively.
type nafScalar struct {
	// pos houses the positive portion of the representation.  An additional
	// byte is required for the positive portion because the NAF encoding can be
	// up to 1 bit longer than the normal binary encoding of the value.
	//
	// neg houses the negative portion of the representation.  Even though the
	// additional byte is not required for the negative portion, since it can
	// never exceed the length of the normal binary encoding of the value,
	// keeping the same length for positive and negative portions simplifies
	// working with the representation and allows extra conditional branches to
	// be avoided.
	//
	// start and end specify the starting and ending index to use within the pos
	// and neg arrays, respectively.  This allows fixed size arrays to be used
	// versus needing to dynamically allocate space on the heap.
	//
	// NOTE: The fields are defined in the order that they are to minimize the
	// padding on 32-bit and 64-bit platforms.
	pos        [33]byte
	start, end uint8
	neg        [33]byte
}

// Pos returns the bytes of the encoded value with bits set in the positions
// that represent a signed digit of +1.
func (s *nafScalar) Pos() []byte {
	return s.pos[s.start:s.end]
}

// Neg returns the bytes of the encoded value with bits set in the positions
// that represent a signed digit of -1.
func (s *nafScalar) Neg() []byte {
	return s.neg[s.start:s.end]
}

// naf takes a positive integer up to a maximum value of 2^256 - 1 and returns
// its non-adjacent form (NAF), which is a unique signed-digit representation
// such that no two consecutive digits are nonzero.  See the documentation for
// the returned type for details on how the representation is encoded
// efficiently and how to interpret it
//
// NAF is useful in that it has the fewest nonzero digits of any signed digit
// representation, only 1/3rd of its digits are nonzero on average, and at least
// half of the digits will be 0.
//
// The aforementioned properties are particularly beneficial for optimizing
// elliptic curve point multiplication because they effectively minimize the
// number of required point additions in exchange for needing to perform a mix
// of fewer point additions and subtractions and possibly one additional point
// doubling.  This is an excellent tradeoff because subtraction of points has
// the same computational complexity as addition of points and point doubling is
// faster than both.
func naf(k []byte) nafScalar {
	// Strip leading zero bytes.
	for len(k) > 0 && k[0] == 0x00 {
		k = k[1:]
	}

	// The non-adjacent form (NAF) of a positive integer k is an expression
	// k = ∑_(i=0, l-1) k_i * 2^i where k_i ∈ {0,±1}, k_(l-1) != 0, and no two
	// consecutive digits k_i are nonzero.
	//
	// The traditional method of computing the NAF of a positive integer is
	// given by algorithm 3.30 in [GECC].  It consists of repeatedly dividing k
	// by 2 and choosing the remainder so that the quotient (k−r)/2 is even
	// which ensures the next NAF digit is 0.  This requires log_2(k) steps.
	//
	// However, in [BRID], Prodinger notes that a closed form expression for the
	// NAF representation is the bitwise difference 3k/2 - k/2.  This is more
	// efficient as it can be computed in O(1) versus the O(log(n)) of the
	// traditional approach.
	//
	// The following code makes use of that formula to compute the NAF more
	// efficiently.
	//
	// To understand the logic here, observe that the only way the NAF has a
	// nonzero digit at a given bit is when either 3k/2 or k/2 has a bit set in
	// that position, but not both.  In other words, the result of a bitwise
	// xor.  This can be seen simply by considering that when the bits are the
	// same, the subtraction is either 0-0 or 1-1, both of which are 0.
	//
	// Further, observe that the "+1" digits in the result are contributed by
	// 3k/2 while the "-1" digits are from k/2.  So, they can be determined by
	// taking the bitwise and of each respective value with the result of the
	// xor which identifies which bits are nonzero.
	//
	// Using that information, this loops backwards from the least significant
	// byte to the most significant byte while performing the aforementioned
	// calculations by propagating the potential carry and high order bit from
	// the next word during the right shift.
	kLen := len(k)
	var result nafScalar
	var carry uint8
	for byteNum := kLen - 1; byteNum >= 0; byteNum-- {
		// Calculate k/2.  Notice the carry from the previous word is added and
		// the low order bit from the next word is shifted in accordingly.
		kc := uint16(k[byteNum]) + uint16(carry)
		var nextWord uint8
		if byteNum > 0 {
			nextWord = k[byteNum-1]
		}
		halfK := kc>>1 | uint16(nextWord<<7)

		// Calculate 3k/2 and determine the non-zero digits in the result.
		threeHalfK := kc + halfK
		nonZeroResultDigits := threeHalfK ^ halfK

		// Determine the signed digits {0, ±1}.
		result.pos[byteNum+1] = uint8(threeHalfK & nonZeroResultDigits)
		result.neg[byteNum+1] = uint8(halfK & nonZeroResultDigits)

		// Propagate the potential carry from the 3k/2 calculation.
		carry = uint8(threeHalfK >> 8)
	}
	result.pos[0] = carry

	// Set the starting and ending positions within the fixed size arrays to
	// identify the bytes that are actually used.  This is important since the
	// encoding is big endian and thus trailing zero bytes changes its value.
	result.start = 1 - carry
	result.end = uint8(kLen + 1)
	return result
}

// ScalarMultNonConst multiplies k*P where k is a scalar modulo the curve order
// and P is a point in Jacobian projective coordinates and stores the result in
// the provided Jacobian point.
//
// NOTE: The point must be normalized for this function to return the correct
// result.  The resulting point will be normalized.
func ScalarMultNonConst(k *ModNScalar, point, result *JacobianPoint) {
	// -------------------------------------------------------------------------
	// This makes use of the following efficiently-computable endomorphism to
	// accelerate the computation:
	//
	// φ(P) ⟼ λ*P = (β*P.x mod p, P.y)
	//
	// In other words, there is a special scalar λ that every point on the
	// elliptic curve can be multiplied by that will result in the same point as
	// performing a single field multiplication of the point's X coordinate by
	// the special value β.
	//
	// This is useful because scalar point multiplication is significantly more
	// expensive than a single field multiplication given the former involves a
	// series of point doublings and additions which themselves consist of a
	// combination of several field multiplications, squarings, and additions.
	//
	// So, the idea behind making use of the endomorphism is thus to decompose
	// the scalar into two scalars that are each about half the bit length of
	// the original scalar such that:
	//
	// k ≡ k1 + k2*λ (mod n)
	//
	// This in turn allows the scalar point multiplication to be performed as a
	// sum of two smaller half-length multiplications as follows:
	//
	// k*P = (k1 + k2*λ)*P
	//     = k1*P + k2*λ*P
	//     = k1*P + k2*φ(P)
	//
	// Thus, a speedup is achieved so long as it's faster to decompose the
	// scalar, compute φ(P), and perform a simultaneous multiply of the
	// half-length point multiplications than it is to compute a full width
	// point multiplication.
	//
	// In practice, benchmarks show the current implementation provides a
	// speedup of around 30-35% versus not using the endomorphism.
	//
	// See section 3.5 in [GECC] for a more rigorous treatment.
	// -------------------------------------------------------------------------

	// Per above, the main equation here to remember is:
	//   k*P = k1*P + k2*φ(P)
	//
	// p1 below is P in the equation while p2 is φ(P) in the equation.
	//
	// NOTE: φ(x,y) = (β*x,y).  The Jacobian z coordinates are the same, so this
	// math goes through.
	//
	// Also, calculate -p1 and -p2 for use in the NAF optimization.
	p1, p1Neg := new(JacobianPoint), new(JacobianPoint)
	p1.Set(point)
	p1Neg.Set(p1)
	p1Neg.Y.Negate(1).Normalize()
	p2, p2Neg := new(JacobianPoint), new(JacobianPoint)
	p2.Set(p1)
	p2.X.Mul(endoBeta).Normalize()
	p2Neg.Set(p2)
	p2Neg.Y.Negate(1).Normalize()

	// Decompose k into k1 and k2 such that k = k1 + k2*λ (mod n) where k1 and
	// k2 are around half the bit length of k in order to halve the number of EC
	// operations.
	//
	// Notice that this also flips the sign of the scalars and points as needed
	// to minimize the bit lengths of the scalars k1 and k2.
	//
	// This is done because the scalars are operating modulo the group order
	// which means that when they would otherwise be a small negative magnitude
	// they will instead be a large positive magnitude.  Since the goal is for
	// the scalars to have a small magnitude to achieve a performance boost, use
	// their negation when they are greater than the half order of the group and
	// flip the positive and negative values of the corresponding point that
	// will be multiplied by to compensate.
	//
	// In other words, transform the calc when k1 is over the half order to:
	//   k1*P = -k1*-P
	//
	// Similarly, transform the calc when k2 is over the half order to:
	//   k2*φ(P) = -k2*-φ(P)
	k1, k2 := splitK(k)
	if k1.IsOverHalfOrder() {
		k1.Negate()
		p1, p1Neg = p1Neg, p1
	}
	if k2.IsOverHalfOrder() {
		k2.Negate()
		p2, p2Neg = p2Neg, p2
	}

	// Convert k1 and k2 into their NAF representations since NAF has a lot more
	// zeros overall on average which minimizes the number of required point
	// additions in exchange for a mix of fewer point additions and subtractions
	// at the cost of one additional point doubling.
	//
	// This is an excellent tradeoff because subtraction of points has the same
	// computational complexity as addition of points and point doubling is
	// faster than both.
	//
	// Concretely, on average, 1/2 of all bits will be non-zero with the normal
	// binary representation whereas only 1/3rd of the bits will be non-zero
	// with NAF.
	//
	// The Pos version of the bytes contain the +1s and the Neg versions contain
	// the -1s.
	k1Bytes, k2Bytes := k1.Bytes(), k2.Bytes()
	k1NAF, k2NAF := naf(k1Bytes[:]), naf(k2Bytes[:])
	k1PosNAF, k1NegNAF := k1NAF.Pos(), k1NAF.Neg()
	k2PosNAF, k2NegNAF := k2NAF.Pos(), k2NAF.Neg()
	k1Len, k2Len := len(k1PosNAF), len(k2PosNAF)

	// Add left-to-right using the NAF optimization.  See algorithm 3.77 from
	// [GECC].
	//
	// Point Q = ∞ (point at infinity).
	var q JacobianPoint
	m := k1Len
	if m < k2Len {
		m = k2Len
	}
	for i := 0; i < m; i++ {
		// Since k1 and k2 are potentially different lengths and the calculation
		// is being done left to right, pad the front of the shorter one with
		// 0s.
		var k1BytePos, k1ByteNeg, k2BytePos, k2ByteNeg byte
		if i >= m-k1Len {
			k1BytePos, k1ByteNeg = k1PosNAF[i-(m-k1Len)], k1NegNAF[i-(m-k1Len)]
		}
		if i >= m-k2Len {
			k2BytePos, k2ByteNeg = k2PosNAF[i-(m-k2Len)], k2NegNAF[i-(m-k2Len)]
		}

		for mask := uint8(1 << 7); mask > 0; mask >>= 1 {
			// Q = 2 * Q
			DoubleNonConst(&q, &q)

			// Add or subtract the first point based on the signed digit of the
			// NAF representation of k1 at this bit position.
			//
			// +1: Q = Q + p1
			// -1: Q = Q - p1
			//  0: Q = Q (no change)
			if k1BytePos&mask == mask {
				AddNonConst(&q, p1, &q)
			} else if k1ByteNeg&mask == mask {
				AddNonConst(&q, p1Neg, &q)
			}

			// Add or subtract the second point based on the signed digit of the
			// NAF representation of k2 at this bit position.
			//
			// +1: Q = Q + p2
			// -1: Q = Q - p2
			//  0: Q = Q (no change)
			if k2BytePos&mask == mask {
				AddNonConst(&q, p2, &q)
			} else if k2ByteNeg&mask == mask {
				AddNonConst(&q, p2Neg, &q)
			}
		}
	}

	result.Set(&q)
}

// ScalarBaseMultNonConst multiplies k*G where k is a scalar modulo the curve
// order and G is the base point of the group and stores the result in the
// provided Jacobian point.
//
// NOTE: The resulting point will be normalized.
func ScalarBaseMultNonConst(k *ModNScalar, result *JacobianPoint) {
	bytePoints := s256BytePoints()

	// Start with the point at infinity.
	result.X.Zero()
	result.Y.Zero()
	result.Z.Zero()

	// bytePoints has all 256 byte points for each 8-bit window.  The strategy
	// is to add up the byte points.  This is best understood by expressing k in
	// base-256 which it already sort of is.  Each "digit" in the 8-bit window
	// can be looked up using bytePoints and added together.
	kb := k.Bytes()
	for i := 0; i < len(kb); i++ {
		pt := &bytePoints[i][kb[i]]
		AddNonConst(result, pt, result)
	}
}

// isOnCurve returns whether or not the affine point (x,y) is on the curve.
func isOnCurve(fx, fy *FieldVal) bool {
	// Elliptic curve equation for secp256k1 is: y^2 = x^3 + 7
	y2 := new(FieldVal).SquareVal(fy).Normalize()
	result := new(FieldVal).SquareVal(fx).Mul(fx).AddInt(7).Normalize()
	return y2.Equals(result)
}

// DecompressY attempts to calculate the Y coordinate for the given X coordinate
// such that the result pair is a point on the secp256k1 curve.  It adjusts Y
// based on the desired oddness and returns whether or not it was successful
// since not all X coordinates are valid.
//
// The magnitude of the provided X coordinate field val must be a max of 8 for a
// correct result.  The resulting Y field val will have a max magnitude of 2.
func DecompressY(x *FieldVal, odd bool, resultY *FieldVal) bool {
	// The curve equation for secp256k1 is: y^2 = x^3 + 7.  Thus
	// y = +-sqrt(x^3 + 7).
	//
	// The x coordinate must be invalid if there is no square root for the
	// calculated rhs because it means the X coordinate is not for a point on
	// the curve.
	x3PlusB := new(FieldVal).SquareVal(x).Mul(x).AddInt(7)
	if hasSqrt := resultY.SquareRootVal(x3PlusB); !hasSqrt {
		return false
	}
	if resultY.Normalize().IsOdd() != odd {
		resultY.Negate(1)
	}
	return true
}
