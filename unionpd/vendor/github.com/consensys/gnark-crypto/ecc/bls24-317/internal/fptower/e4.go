// Copyright 2020 ConsenSys Software Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package fptower

import (
	"math/big"

	"github.com/consensys/gnark-crypto/ecc/bls24-317/fp"
)

// E4 is a degree two finite field extension of fp2
type E4 struct {
	B0, B1 E2
}

// Equal returns true if z equals x, fasle otherwise
func (z *E4) Equal(x *E4) bool {
	return z.B0.Equal(&x.B0) && z.B1.Equal(&x.B1)
}

// Cmp compares (lexicographic order) z and x and returns:
//
//	-1 if z <  x
//	 0 if z == x
//	+1 if z >  x
func (z *E4) Cmp(x *E4) int {
	if a1 := z.B1.Cmp(&x.B1); a1 != 0 {
		return a1
	}
	return z.B0.Cmp(&x.B0)
}

// LexicographicallyLargest returns true if this element is strictly lexicographically
// larger than its negation, false otherwise
func (z *E4) LexicographicallyLargest() bool {
	// adapted from github.com/zkcrypto/bls12_381
	if z.B1.IsZero() {
		return z.B0.LexicographicallyLargest()
	}
	return z.B1.LexicographicallyLargest()
}

// String puts E4 in string form
func (z *E4) String() string {
	return (z.B0.String() + "+(" + z.B1.String() + ")*v")
}

// SetString sets a E4 from string
func (z *E4) SetString(s0, s1, s2, s3 string) *E4 {
	z.B0.SetString(s0, s1)
	z.B1.SetString(s2, s3)
	return z
}

// Set copies x into z and returns z
func (z *E4) Set(x *E4) *E4 {
	z.B0 = x.B0
	z.B1 = x.B1
	return z
}

// SetZero sets an E4 elmt to zero
func (z *E4) SetZero() *E4 {
	z.B0.SetZero()
	z.B1.SetZero()
	return z
}

// SetOne sets z to 1 in Montgomery form and returns z
func (z *E4) SetOne() *E4 {
	*z = E4{}
	z.B0.A0.SetOne()
	return z
}

// MulByElement multiplies an element in E4 by an element in fp
func (z *E4) MulByElement(x *E4, y *fp.Element) *E4 {
	var yCopy fp.Element
	yCopy.Set(y)
	z.B0.MulByElement(&x.B0, &yCopy)
	z.B1.MulByElement(&x.B1, &yCopy)
	return z
}

// Add set z=x+y in E4 and return z
func (z *E4) Add(x, y *E4) *E4 {
	z.B0.Add(&x.B0, &y.B0)
	z.B1.Add(&x.B1, &y.B1)
	return z
}

// Sub sets z to x sub y and return z
func (z *E4) Sub(x, y *E4) *E4 {
	z.B0.Sub(&x.B0, &y.B0)
	z.B1.Sub(&x.B1, &y.B1)
	return z
}

// Double sets z=2*x and returns z
func (z *E4) Double(x *E4) *E4 {
	z.B0.Double(&x.B0)
	z.B1.Double(&x.B1)
	return z
}

// Neg negates an E4 element
func (z *E4) Neg(x *E4) *E4 {
	z.B0.Neg(&x.B0)
	z.B1.Neg(&x.B1)
	return z
}

// SetRandom used only in tests
func (z *E4) SetRandom() (*E4, error) {
	if _, err := z.B0.SetRandom(); err != nil {
		return nil, err
	}
	if _, err := z.B1.SetRandom(); err != nil {
		return nil, err
	}
	return z, nil
}

// IsZero returns true if the element is zero, fasle otherwise
func (z *E4) IsZero() bool {
	return z.B0.IsZero() && z.B1.IsZero()
}

func (z *E4) IsOne() bool {
	return z.B0.IsOne() && z.B1.IsZero()
}

// MulByNonResidue mul x by (0,1)
func (z *E4) MulByNonResidue(x *E4) *E4 {
	z.B1, z.B0 = x.B0, x.B1
	z.B0.MulByNonResidue(&z.B0)
	return z
}

// MulByNonResidueInv mul x by (0,1)⁻¹
func (z *E4) MulByNonResidueInv(x *E4) *E4 {
	a := x.B1
	var uInv E2
	uInv.A0.SetString("68196535552147955757549882954137028530972556060709796988605069651952986598616012809013078365526")
	uInv.A1.SetString("68196535552147955757549882954137028530972556060709796988605069651952986598616012809013078365525")
	z.B1.Mul(&x.B0, &uInv)
	z.B0 = a
	return z
}

// Mul set z=x*y in E4 and return z
func (z *E4) Mul(x, y *E4) *E4 {
	var a, b, c E2
	a.Add(&x.B0, &x.B1)
	b.Add(&y.B0, &y.B1)
	a.Mul(&a, &b)
	b.Mul(&x.B0, &y.B0)
	c.Mul(&x.B1, &y.B1)
	z.B1.Sub(&a, &b).Sub(&z.B1, &c)
	z.B0.MulByNonResidue(&c).Add(&z.B0, &b)
	return z
}

// Square set z=x*x in E4 and return z
func (z *E4) Square(x *E4) *E4 {

	//Algorithm 22 from https://eprint.iacr.org/2010/354.pdf
	var c0, c2, c3 E2
	c0.Sub(&x.B0, &x.B1)
	c3.MulByNonResidue(&x.B1).Sub(&x.B0, &c3)
	c2.Mul(&x.B0, &x.B1)
	c0.Mul(&c0, &c3).Add(&c0, &c2)
	z.B1.Double(&c2)
	c2.MulByNonResidue(&c2)
	z.B0.Add(&c0, &c2)

	return z
}

// Inverse set z to the inverse of x in E4 and return z
//
// if x == 0, sets and returns z = x
func (z *E4) Inverse(x *E4) *E4 {
	// Algorithm 23 from https://eprint.iacr.org/2010/354.pdf

	var t0, t1, tmp E2
	t0.Square(&x.B0)
	t1.Square(&x.B1)
	tmp.MulByNonResidue(&t1)
	t0.Sub(&t0, &tmp)
	t1.Inverse(&t0)
	z.B0.Mul(&x.B0, &t1)
	z.B1.Mul(&x.B1, &t1).Neg(&z.B1)

	return z
}

// Exp sets z=xᵏ (mod q⁴) and returns it
func (z *E4) Exp(x E4, k *big.Int) *E4 {
	if k.IsUint64() && k.Uint64() == 0 {
		return z.SetOne()
	}

	e := k
	if k.Sign() == -1 {
		// negative k, we invert
		// if k < 0: xᵏ (mod q⁴) == (x⁻¹)ᵏ (mod q⁴)
		x.Inverse(&x)

		// we negate k in a temp big.Int since
		// Int.Bit(_) of k and -k is different
		e = bigIntPool.Get().(*big.Int)
		defer bigIntPool.Put(e)
		e.Neg(k)
	}

	z.SetOne()
	b := e.Bytes()
	for i := 0; i < len(b); i++ {
		w := b[i]
		for j := 0; j < 8; j++ {
			z.Square(z)
			if (w & (0b10000000 >> j)) != 0 {
				z.Mul(z, &x)
			}
		}
	}

	return z
}

// Conjugate set z to x conjugated and return z
func (z *E4) Conjugate(x *E4) *E4 {
	z.B0 = x.B0
	z.B1.Neg(&x.B1)
	return z
}

func (z *E4) Halve() {

	z.B0.A0.Halve()
	z.B0.A1.Halve()
	z.B1.A0.Halve()
	z.B1.A1.Halve()
}

// norm sets x to the norm of z
func (z *E4) norm(x *E2) {
	var tmp E2
	tmp.Square(&z.B1).MulByNonResidue(&tmp)
	x.Square(&z.B0).Sub(x, &tmp)
}

// Legendre returns the Legendre symbol of z
func (z *E4) Legendre() int {
	var n E2
	z.norm(&n)
	return n.Legendre()
}

// Sqrt sets z to the square root of and returns z
// The function does not test wether the square root
// exists or not, it's up to the caller to call
// Legendre beforehand.
// cf https://eprint.iacr.org/2012/685.pdf (algo 10)
func (z *E4) Sqrt(x *E4) *E4 {

	// precomputation
	var b, c, d, e, f, x0, _g E4
	var _b, o E2

	// c must be a non square (works for p=1 mod 12 hence 1 mod 4, only bls377 has such a p currently)
	c.B1.SetOne()

	q := fp.Modulus()
	var exp, one big.Int
	one.SetUint64(1)
	exp.Mul(q, q).Sub(&exp, &one).Rsh(&exp, 1)
	d.Exp(c, &exp)
	e.Mul(&d, &c).Inverse(&e)
	f.Mul(&d, &c).Square(&f)

	// computation
	exp.Rsh(&exp, 1)
	b.Exp(*x, &exp)
	b.norm(&_b)
	o.SetOne()
	if _b.Equal(&o) {
		x0.Square(&b).Mul(&x0, x)
		_b.Set(&x0.B0).Sqrt(&_b)
		_g.B0.Set(&_b)
		z.Conjugate(&b).Mul(z, &_g)
		return z
	}
	x0.Square(&b).Mul(&x0, x).Mul(&x0, &f)
	_b.Set(&x0.B0).Sqrt(&_b)
	_g.B0.Set(&_b)
	z.Conjugate(&b).Mul(z, &_g).Mul(z, &e)

	return z
}

// BatchInvertE4 returns a new slice with every element inverted.
// Uses Montgomery batch inversion trick
//
// if a[i] == 0, returns result[i] = a[i]
func BatchInvertE4(a []E4) []E4 {
	res := make([]E4, len(a))
	if len(a) == 0 {
		return res
	}

	zeroes := make([]bool, len(a))
	var accumulator E4
	accumulator.SetOne()

	for i := 0; i < len(a); i++ {
		if a[i].IsZero() {
			zeroes[i] = true
			continue
		}
		res[i].Set(&accumulator)
		accumulator.Mul(&accumulator, &a[i])
	}

	accumulator.Inverse(&accumulator)

	for i := len(a) - 1; i >= 0; i-- {
		if zeroes[i] {
			continue
		}
		res[i].Mul(&res[i], &accumulator)
		accumulator.Mul(&accumulator, &a[i])
	}

	return res
}

func (z *E4) Div(x *E4, y *E4) *E4 {
	var r E4
	r.Inverse(y).Mul(x, &r)
	return z.Set(&r)
}
