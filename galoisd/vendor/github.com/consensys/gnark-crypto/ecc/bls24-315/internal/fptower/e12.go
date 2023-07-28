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
)

// E12 is a degree three finite field extension of fp4
type E12 struct {
	C0, C1, C2 E4
}

// Equal returns true if z equals x, fasle otherwise
func (z *E12) Equal(x *E12) bool {
	return z.C0.Equal(&x.C0) && z.C1.Equal(&x.C1) && z.C2.Equal(&x.C2)
}

// String puts E12 elmt in string form
func (z *E12) String() string {
	return (z.C0.String() + "+(" + z.C1.String() + ")*w+(" + z.C2.String() + ")*w**2")
}

// SetString sets a E12 elmt from stringf
func (z *E12) SetString(s0, s1, s2, s3, s4, s5, s6, s7, s8, s9, s10, s11 string) *E12 {
	z.C0.SetString(s0, s1, s2, s3)
	z.C1.SetString(s4, s5, s6, s7)
	z.C2.SetString(s8, s9, s10, s11)
	return z
}

// Set Sets a E12 elmt form another E12 elmt
func (z *E12) Set(x *E12) *E12 {
	z.C0 = x.C0
	z.C1 = x.C1
	z.C2 = x.C2
	return z
}

// SetOne sets z to 1 in Montgomery form and returns z
func (z *E12) SetOne() *E12 {
	*z = E12{}
	z.C0.B0.A0.SetOne()
	return z
}

// SetRandom set z to a random elmt
func (z *E12) SetRandom() (*E12, error) {
	if _, err := z.C0.SetRandom(); err != nil {
		return nil, err
	}
	if _, err := z.C1.SetRandom(); err != nil {
		return nil, err
	}
	if _, err := z.C2.SetRandom(); err != nil {
		return nil, err
	}
	return z, nil
}

// IsZero returns true if the two elements are equal, fasle otherwise
func (z *E12) IsZero() bool {
	return z.C0.IsZero() && z.C1.IsZero() && z.C2.IsZero()
}

func (z *E12) IsOne() bool {
	return z.C0.IsOne() && z.C1.IsZero() && z.C2.IsZero()
}

// Add adds two elements of E12
func (z *E12) Add(x, y *E12) *E12 {
	z.C0.Add(&x.C0, &y.C0)
	z.C1.Add(&x.C1, &y.C1)
	z.C2.Add(&x.C2, &y.C2)
	return z
}

// Neg negates the E12 number
func (z *E12) Neg(x *E12) *E12 {
	z.C0.Neg(&x.C0)
	z.C1.Neg(&x.C1)
	z.C2.Neg(&x.C2)
	return z
}

// Sub two elements of E12
func (z *E12) Sub(x, y *E12) *E12 {
	z.C0.Sub(&x.C0, &y.C0)
	z.C1.Sub(&x.C1, &y.C1)
	z.C2.Sub(&x.C2, &y.C2)
	return z
}

// Double doubles an element in E12
func (z *E12) Double(x *E12) *E12 {
	z.C0.Double(&x.C0)
	z.C1.Double(&x.C1)
	z.C2.Double(&x.C2)
	return z
}

// MulByNonResidue mul x by (0,1,0)
func (z *E12) MulByNonResidue(x *E12) *E12 {
	z.C2, z.C1, z.C0 = x.C1, x.C0, x.C2
	z.C0.MulByNonResidue(&z.C0)
	return z
}

// Mul sets z to the E12 product of x,y, returns z
func (z *E12) Mul(x, y *E12) *E12 {
	// Algorithm 13 from https://eprint.iacr.org/2010/354.pdf
	var t0, t1, t2, c0, c1, c2, tmp E4
	t0.Mul(&x.C0, &y.C0)
	t1.Mul(&x.C1, &y.C1)
	t2.Mul(&x.C2, &y.C2)

	c0.Add(&x.C1, &x.C2)
	tmp.Add(&y.C1, &y.C2)
	c0.Mul(&c0, &tmp).Sub(&c0, &t1).Sub(&c0, &t2).MulByNonResidue(&c0).Add(&c0, &t0)

	c1.Add(&x.C0, &x.C1)
	tmp.Add(&y.C0, &y.C1)
	c1.Mul(&c1, &tmp).Sub(&c1, &t0).Sub(&c1, &t1)
	tmp.MulByNonResidue(&t2)
	c1.Add(&c1, &tmp)

	tmp.Add(&x.C0, &x.C2)
	c2.Add(&y.C0, &y.C2).Mul(&c2, &tmp).Sub(&c2, &t0).Sub(&c2, &t2).Add(&c2, &t1)

	z.C0.Set(&c0)
	z.C1.Set(&c1)
	z.C2.Set(&c2)

	return z
}

// Square sets z to the E12 product of x,x, returns z
func (z *E12) Square(x *E12) *E12 {

	// Algorithm 16 from https://eprint.iacr.org/2010/354.pdf
	var c4, c5, c1, c2, c3, c0 E4
	c4.Mul(&x.C0, &x.C1).Double(&c4)
	c5.Square(&x.C2)
	c1.MulByNonResidue(&c5).Add(&c1, &c4)
	c2.Sub(&c4, &c5)
	c3.Square(&x.C0)
	c4.Sub(&x.C0, &x.C1).Add(&c4, &x.C2)
	c5.Mul(&x.C1, &x.C2).Double(&c5)
	c4.Square(&c4)
	c0.MulByNonResidue(&c5).Add(&c0, &c3)
	z.C2.Add(&c2, &c4).Add(&z.C2, &c5).Sub(&z.C2, &c3)
	z.C0.Set(&c0)
	z.C1.Set(&c1)

	return z
}

// Inverse an element in E12
//
// if x == 0, sets and returns z = x
func (z *E12) Inverse(x *E12) *E12 {
	// Algorithm 17 from https://eprint.iacr.org/2010/354.pdf
	// step 9 is wrong in the paper it's t1-t4
	var t0, t1, t2, t3, t4, t5, t6, c0, c1, c2, d1, d2 E4
	t0.Square(&x.C0)
	t1.Square(&x.C1)
	t2.Square(&x.C2)
	t3.Mul(&x.C0, &x.C1)
	t4.Mul(&x.C0, &x.C2)
	t5.Mul(&x.C1, &x.C2)
	c0.MulByNonResidue(&t5).Sub(&t0, &c0)
	c1.MulByNonResidue(&t2).Sub(&c1, &t3)
	c2.Sub(&t1, &t4)
	t6.Mul(&x.C0, &c0)
	d1.Mul(&x.C2, &c1)
	d2.Mul(&x.C1, &c2)
	d1.Add(&d1, &d2).MulByNonResidue(&d1)
	t6.Add(&t6, &d1)
	t6.Inverse(&t6)
	z.C0.Mul(&c0, &t6)
	z.C1.Mul(&c1, &t6)
	z.C2.Mul(&c2, &t6)

	return z
}

// BatchInvertE12 returns a new slice with every element inverted.
// Uses Montgomery batch inversion trick
//
// if a[i] == 0, returns result[i] = a[i]
func BatchInvertE12(a []E12) []E12 {
	res := make([]E12, len(a))
	if len(a) == 0 {
		return res
	}

	zeroes := make([]bool, len(a))
	var accumulator E12
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

// Exp sets z=xᵏ (mod q¹²) and returns it
// uses 2-bits windowed method
func (z *E12) Exp(x E12, k *big.Int) *E12 {
	if k.IsUint64() && k.Uint64() == 0 {
		return z.SetOne()
	}

	e := k
	if k.Sign() == -1 {
		// negative k, we invert
		// if k < 0: xᵏ (mod q¹²) == (x⁻¹)ᵏ (mod q¹²)
		x.Inverse(&x)

		// we negate k in a temp big.Int since
		// Int.Bit(_) of k and -k is different
		e = bigIntPool.Get().(*big.Int)
		defer bigIntPool.Put(e)
		e.Neg(k)
	}

	var res E12
	var ops [3]E12

	res.SetOne()
	ops[0].Set(&x)
	ops[1].Square(&ops[0])
	ops[2].Set(&ops[0]).Mul(&ops[2], &ops[1])

	b := e.Bytes()
	for i := range b {
		w := b[i]
		mask := byte(0xc0)
		for j := 0; j < 4; j++ {
			res.Square(&res).Square(&res)
			c := (w & mask) >> (6 - 2*j)
			if c != 0 {
				res.Mul(&res, &ops[c-1])
			}
			mask = mask >> 2
		}
	}
	z.Set(&res)

	return z
}

// InverseUnitary inverse a unitary element
func (z *E12) InverseUnitary(x *E12) *E12 {
	return z.Conjugate(x)
}

// Conjugate set z to x conjugated and return z
func (z *E12) Conjugate(x *E12) *E12 {
	z.C0.Conjugate(&x.C0)
	z.C1.Conjugate(&x.C1).Neg(&z.C1)
	z.C2.Conjugate(&x.C2)
	return z
}

// MulBy01 multiplication by sparse element (c0,c1,0)
func (z *E12) MulBy01(c0, c1 *E4) *E12 {

	var a, b, tmp, t0, t1, t2 E4

	a.Mul(&z.C0, c0)
	b.Mul(&z.C1, c1)

	tmp.Add(&z.C1, &z.C2)
	t0.Mul(c1, &tmp)
	t0.Sub(&t0, &b)
	t0.MulByNonResidue(&t0)
	t0.Add(&t0, &a)

	tmp.Add(&z.C0, &z.C2)
	t2.Mul(c0, &tmp)
	t2.Sub(&t2, &a)
	t2.Add(&t2, &b)

	t1.Add(c0, c1)
	tmp.Add(&z.C0, &z.C1)
	t1.Mul(&t1, &tmp)
	t1.Sub(&t1, &a)
	t1.Sub(&t1, &b)

	z.C0.Set(&t0)
	z.C1.Set(&t1)
	z.C2.Set(&t2)

	return z
}

// MulByE2 multiplies an element in E12 by an element in E2
func (z *E12) MulByE2(x *E12, y *E4) *E12 {
	var yCopy E4
	yCopy.Set(y)
	z.C0.Mul(&x.C0, &yCopy)
	z.C1.Mul(&x.C1, &yCopy)
	z.C2.Mul(&x.C2, &yCopy)
	return z
}
