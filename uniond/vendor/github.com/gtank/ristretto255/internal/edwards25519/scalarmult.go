// Copyright (c) 2019 Henry de Valence.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package edwards25519

import (
	"github.com/gtank/ristretto255/internal/scalar"
)

// Set v to x*B, where B is the Ed25519 basepoint, and return v.
//
// The scalar multiplication is done in constant time.
func (v *ProjP3) BasepointMul(x *scalar.Scalar) *ProjP3 {
	// Write x = sum(x_i * 16^i) so  x*B = sum( B*x_i*16^i )
	// as described in the Ed25519 paper
	//
	// Group even and odd coefficients
	// x*B     = x_0*16^0*B + x_2*16^2*B + ... + x_62*16^62*B
	//         + x_1*16^1*B + x_3*16^3*B + ... + x_63*16^63*B
	// x*B     = x_0*16^0*B + x_2*16^2*B + ... + x_62*16^62*B
	//    + 16*( x_1*16^0*B + x_3*16^2*B + ... + x_63*16^62*B)
	//
	// We use a lookup table for each i to get x_i*16^(2*i)*B
	// and do four doublings to multiply by 16.
	digits := x.SignedRadix16()

	multiple := &AffineCached{}
	tmp1 := &ProjP1xP1{}
	tmp2 := &ProjP2{}

	// Accumulate the odd components first
	v.Zero()
	for i := 1; i < 64; i += 2 {
		basepointTable[i/2].SelectInto(multiple, digits[i])
		tmp1.AddAffine(v, multiple)
		v.FromP1xP1(tmp1)
	}

	// Multiply by 16
	tmp2.FromP3(v)       // tmp2 =    v in P2 coords
	tmp1.Double(tmp2)    // tmp1 =  2*v in P1xP1 coords
	tmp2.FromP1xP1(tmp1) // tmp2 =  2*v in P2 coords
	tmp1.Double(tmp2)    // tmp1 =  4*v in P1xP1 coords
	tmp2.FromP1xP1(tmp1) // tmp2 =  4*v in P2 coords
	tmp1.Double(tmp2)    // tmp1 =  8*v in P1xP1 coords
	tmp2.FromP1xP1(tmp1) // tmp2 =  8*v in P2 coords
	tmp1.Double(tmp2)    // tmp1 = 16*v in P1xP1 coords
	v.FromP1xP1(tmp1)    // now v = 16*(odd components)

	// Accumulate the even components
	for i := 0; i < 64; i += 2 {
		basepointTable[i/2].SelectInto(multiple, digits[i])
		tmp1.AddAffine(v, multiple)
		v.FromP1xP1(tmp1)
	}

	return v
}

// Set v to x*Q, and return v.  v and q may alias.
//
// The scalar multiplication is done in constant time.
func (v *ProjP3) ScalarMul(x *scalar.Scalar, q *ProjP3) *ProjP3 {
	var table projLookupTable
	table.FromP3(q)
	// v and q could alias, but once the table is built we can clobber v.
	v.Zero()

	// Write x = sum(x_i * 16^i)
	// so  x*Q = sum( Q*x_i*16^i )
	//         = Q*x_0 + 16*(Q*x_1 + 16*( ... + Q*x_63) ... )
	//           <------compute inside out---------
	//
	// We use the lookup table to get the x_i*Q values
	// and do four doublings to compute 16*Q
	digits := x.SignedRadix16()

	// Unwrap first loop iteration to save computing 16*identity
	multiple := &ProjCached{}
	tmp1 := &ProjP1xP1{}
	tmp2 := &ProjP2{}
	table.SelectInto(multiple, digits[63])
	tmp1.Add(v, multiple) // tmp1 = x_63*Q in P1xP1 coords
	for i := 62; i >= 0; i-- {
		tmp2.FromP1xP1(tmp1) // tmp2 =    (prev) in P2 coords
		tmp1.Double(tmp2)    // tmp1 =  2*(prev) in P1xP1 coords
		tmp2.FromP1xP1(tmp1) // tmp2 =  2*(prev) in P2 coords
		tmp1.Double(tmp2)    // tmp1 =  4*(prev) in P1xP1 coords
		tmp2.FromP1xP1(tmp1) // tmp2 =  4*(prev) in P2 coords
		tmp1.Double(tmp2)    // tmp1 =  8*(prev) in P1xP1 coords
		tmp2.FromP1xP1(tmp1) // tmp2 =  8*(prev) in P2 coords
		tmp1.Double(tmp2)    // tmp1 = 16*(prev) in P1xP1 coords
		v.FromP1xP1(tmp1)    //    v = 16*(prev) in P3 coords
		table.SelectInto(multiple, digits[i])
		tmp1.Add(v, multiple) // tmp1 = x_i*Q + 16*(prev) in P1xP1 coords
	}
	v.FromP1xP1(tmp1)
	return v
}

// Set v to the result of a multiscalar multiplication and return v.
//
// The multiscalar multiplication is sum(scalars[i]*points[i]).
//
// The multiscalar multiplication is performed in constant time.
func (v *ProjP3) MultiscalarMul(scalars []scalar.Scalar, points []*ProjP3) *ProjP3 {
	if len(scalars) != len(points) {
		panic("called MultiscalarMul with different size inputs")
	}

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
		digits[i] = scalars[i].SignedRadix16()
	}

	// Unwrap first loop iteration to save computing 16*identity
	multiple := &ProjCached{}
	tmp1 := &ProjP1xP1{}
	tmp2 := &ProjP2{}
	// Lookup-and-add the appropriate multiple of each input point
	for j := range tables {
		tables[j].SelectInto(multiple, digits[j][63])
		tmp1.Add(v, multiple) // tmp1 = v + x_(j,63)*Q in P1xP1 coords
		v.FromP1xP1(tmp1)     // update v
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
		v.FromP1xP1(tmp1)    //    v = 16*(prev) in P3 coords
		// Lookup-and-add the appropriate multiple of each input point
		for j := range tables {
			tables[j].SelectInto(multiple, digits[j][i])
			tmp1.Add(v, multiple) // tmp1 = v + x_(j,i)*Q in P1xP1 coords
			v.FromP1xP1(tmp1)     // update v
		}
		tmp2.FromP3(v) // set up tmp2 = v in P2 coords for next iteration
	}
	return v
}

// Set v to a*A + b*B, where B is the Ed25519 basepoint, and return v.
//
// The scalar multiplication is done in variable time.
func (v *ProjP3) VartimeDoubleBaseMul(a *scalar.Scalar, A *ProjP3, b *scalar.Scalar) *ProjP3 {
	// Similarly to the single variable-base approach, we compute
	// digits and use them with a lookup table.  However, because
	// we are allowed to do variable-time operations, we don't
	// need constant-time lookups or constant-time digit
	// computations.
	//
	// So we use a non-adjacent form of some width w instead of
	// radix 16.  This is like a binary representation (one digit
	// for each binary place) but we allow the digits to grow in
	// magnitude up to 2^{w-1} so that the nonzero digits are as
	// sparse as possible.  Intuitively, this "condenses" the
	// "mass" of the scalar onto sparse coefficients (meaning
	// fewer additions).

	var aTable nafLookupTable5
	aTable.FromP3(A)
	// Because the basepoint is fixed, we can use a wider NAF
	// corresponding to a bigger table.
	aNaf := a.NonAdjacentForm(5)
	bNaf := b.NonAdjacentForm(8)

	// Find the first nonzero coefficient.
	i := 255
	for j := i; j >= 0; j-- {
		if aNaf[j] != 0 || bNaf[j] != 0 {
			break
		}
	}

	multA := &ProjCached{}
	multB := &AffineCached{}
	tmp1 := &ProjP1xP1{}
	tmp2 := &ProjP2{}
	tmp2.Zero()
	v.Zero()

	// Move from high to low bits, doubling the accumulator
	// at each iteration and checking whether there is a nonzero
	// coefficient to look up a multiple of.
	for ; i >= 0; i-- {
		tmp1.Double(tmp2)

		// Only update v if we have a nonzero coeff to add in.
		if aNaf[i] > 0 {
			v.FromP1xP1(tmp1)
			aTable.SelectInto(multA, aNaf[i])
			tmp1.Add(v, multA)
		} else if aNaf[i] < 0 {
			v.FromP1xP1(tmp1)
			aTable.SelectInto(multA, -aNaf[i])
			tmp1.Sub(v, multA)
		}

		if bNaf[i] > 0 {
			v.FromP1xP1(tmp1)
			basepointNafTable.SelectInto(multB, bNaf[i])
			tmp1.AddAffine(v, multB)
		} else if bNaf[i] < 0 {
			v.FromP1xP1(tmp1)
			basepointNafTable.SelectInto(multB, -bNaf[i])
			tmp1.SubAffine(v, multB)
		}

		tmp2.FromP1xP1(tmp1)
	}

	v.FromP2(tmp2)
	return v
}

// Set v to the result of a multiscalar multiplication and return v.
//
// The multiscalar multiplication is sum(scalars[i]*points[i]).
//
// The multiscalar multiplication is performed in variable time.
func (v *ProjP3) VartimeMultiscalarMul(scalars []scalar.Scalar, points []*ProjP3) *ProjP3 {
	if len(scalars) != len(points) {
		panic("called MultiscalarMul with different size inputs")
	}

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
		nafs[i] = scalars[i].NonAdjacentForm(5)
	}

	multiple := &ProjCached{}
	tmp1 := &ProjP1xP1{}
	tmp2 := &ProjP2{}
	tmp2.Zero()
	v.Zero()

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
				v.FromP1xP1(tmp1)
				tables[j].SelectInto(multiple, nafs[j][i])
				tmp1.Add(v, multiple)
			} else if nafs[j][i] < 0 {
				v.FromP1xP1(tmp1)
				tables[j].SelectInto(multiple, -nafs[j][i])
				tmp1.Sub(v, multiple)
			}
		}

		tmp2.FromP1xP1(tmp1)
	}

	v.FromP2(tmp2)
	return v
}
