package ecc

import (
	"math/big"
	"math/bits"
)

//-------------------------------------------------------
// Ate loop counter (not used for each curve)

// NafDecomposition gets the naf decomposition of a big number
func NafDecomposition(a *big.Int, result []int8) int {

	var zero, one, two, three big.Int

	one.SetUint64(1)
	two.SetUint64(2)
	three.SetUint64(3)

	length := 0

	// some buffers
	var buf, aCopy big.Int
	aCopy.Set(a)

	for aCopy.Cmp(&zero) != 0 {

		// if aCopy % 2 == 0
		buf.And(&aCopy, &one)

		// aCopy even
		if buf.Cmp(&zero) == 0 {
			result[length] = 0
		} else { // aCopy odd
			buf.And(&aCopy, &three)
			if buf.Cmp(&three) == 0 {
				result[length] = -1
				aCopy.Add(&aCopy, &one)
			} else {
				result[length] = 1
			}
		}
		aCopy.Rsh(&aCopy, 1)
		length++
	}
	return length
}

//-------------------------------------------------------
// GLV utils

// Lattice represents a Z module spanned by V1, V2.
// det is the associated determinant.
type Lattice struct {
	V1, V2      [2]big.Int
	Det, b1, b2 big.Int
}

// PrecomputeLattice res such that res.V1, res.V2
// are short vectors satisfying v11+v12.λ=v21+v22.λ=0[r].
// cf https://www.iacr.org/archive/crypto2001/21390189.pdf
func PrecomputeLattice(r, lambda *big.Int, res *Lattice) {

	var rst [2][3]big.Int
	var tmp [3]big.Int
	var quotient, remainder, sqroot, _r, _t big.Int

	rst[0][0].Set(r)
	rst[0][1].SetUint64(1)
	rst[0][2].SetUint64(0)

	rst[1][0].Set(lambda)
	rst[1][1].SetUint64(0)
	rst[1][2].SetUint64(1)

	sqroot.Sqrt(r)

	var one big.Int
	one.SetUint64(1)

	// r_i+1 = r_i-1 - q_i.r_i
	// s_i+1 = s_i-1 - q_i.s_i
	// t_i+1 = t_i-1 - q_i.s_i
	for rst[1][0].Cmp(&sqroot) >= 1 {

		quotient.Div(&rst[0][0], &rst[1][0])
		remainder.Mod(&rst[0][0], &rst[1][0])

		tmp[0].Set(&rst[1][0])
		tmp[1].Set(&rst[1][1])
		tmp[2].Set(&rst[1][2])

		rst[1][0].Set(&remainder)
		rst[1][1].Mul(&rst[1][1], &quotient).Sub(&rst[0][1], &rst[1][1])
		rst[1][2].Mul(&rst[1][2], &quotient).Sub(&rst[0][2], &rst[1][2])

		rst[0][0].Set(&tmp[0])
		rst[0][1].Set(&tmp[1])
		rst[0][2].Set(&tmp[2])
	}

	quotient.Div(&rst[0][0], &rst[1][0])
	remainder.Mod(&rst[0][0], &rst[1][0])
	_r.Set(&remainder)
	_t.Mul(&rst[1][2], &quotient).Sub(&rst[0][2], &_t)

	res.V1[0].Set(&rst[1][0])
	res.V1[1].Neg(&rst[1][2])

	// take the shorter of [rst[0][0], rst[0][2]], [_r, _t]
	tmp[1].Mul(&rst[0][2], &rst[0][2])
	tmp[0].Mul(&rst[0][0], &rst[0][0]).Add(&tmp[1], &tmp[0])
	tmp[2].Mul(&_r, &_r)
	tmp[1].Mul(&_t, &_t).Add(&tmp[2], &tmp[1])
	if tmp[0].Cmp(&tmp[1]) == 1 {
		res.V2[0].Set(&_r)
		res.V2[1].Neg(&_t)
	} else {
		res.V2[0].Set(&rst[0][0])
		res.V2[1].Neg(&rst[0][2])
	}

	// sets determinant
	tmp[0].Mul(&res.V1[1], &res.V2[0])
	res.Det.Mul(&res.V1[0], &res.V2[1]).Sub(&res.Det, &tmp[0])

	// sets roundings of 2^n*v21/d and 2^n*v11/d (where 2ⁿ > d)
	n := 2 * uint(((res.Det.BitLen()+32)>>6)<<6)
	res.b1.Lsh(&res.V2[1], n)
	rounding(&res.b1, &res.Det, &res.b1)
	res.b2.Lsh(&res.V1[1], n)
	rounding(&res.b2, &res.Det, &res.b2)
}

// SplitScalar outputs u,v such that u+vlambda=s[r].
// The method is to view s as (s,0) in ZxZ, and find a close
// vector w of (s,0) in <l>, where l is a sub Z-module of
// ker((a,b) → a+b.λ[r]): then (u,v)=w-(s,0), and
// u+v.λ=s[r].
// cf https://www.iacr.org/archive/crypto2001/21390189.pdf
func SplitScalar(s *big.Int, l *Lattice) [2]big.Int {

	var k1, k2 big.Int
	k1.Mul(s, &l.b1)
	k2.Mul(s, &l.b2).Neg(&k2)
	// right-shift instead of division by lattice determinant
	// this increases the bounds on k1 and k2 by 1
	// but we check this ScalarMultiplication alg. (not constant-time)
	n := 2 * uint(((l.Det.BitLen()+32)>>6)<<6)
	k1.Rsh(&k1, n)
	k2.Rsh(&k2, n)
	v := getVector(l, &k1, &k2)
	v[0].Sub(s, &v[0])
	v[1].Neg(&v[1])
	return v
}

// sets res to the closest integer from n/d
func rounding(n, d, res *big.Int) {
	var dshift, r, one big.Int
	one.SetUint64(1)
	dshift.Rsh(d, 1)
	r.Mod(n, d)
	res.Div(n, d)
	if r.Cmp(&dshift) == 1 {
		res.Add(res, &one)
	}
}

// getVector returns aV1 + bV2
func getVector(l *Lattice, a, b *big.Int) [2]big.Int {
	var res [2]big.Int
	var tmp big.Int
	tmp.Mul(b, &l.V2[0])
	res[0].Mul(a, &l.V1[0]).Add(&res[0], &tmp)
	tmp.Mul(b, &l.V2[1])
	res[1].Mul(a, &l.V1[1]).Add(&res[1], &tmp)
	return res
}

// NextPowerOfTwo returns the next power of 2 of n
func NextPowerOfTwo(n uint64) uint64 {
	c := bits.OnesCount64(n)
	if c == 0 {
		return 1
	}
	if c == 1 {
		return n
	}
	t := bits.LeadingZeros64(n)
	if t == 0 {
		panic("next power of 2 overflows uint64")
	}
	return uint64(1) << (64 - t)
}
