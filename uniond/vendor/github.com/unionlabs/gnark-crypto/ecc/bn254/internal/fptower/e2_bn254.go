// Copyright 2020 ConsenSys AG
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
	"github.com/consensys/gnark-crypto/ecc/bn254/fp"
)

// declaring nonResInverse as global makes MulByNonResInv inlinable
var nonResInverse E2 = E2{
	A0: fp.Element{
		10477841894441615122,
		7327163185667482322,
		3635199979766503006,
		3215324977242306624,
	},
	A1: fp.Element{
		7515750141297360845,
		14746352163864140223,
		11319968037783994424,
		30185921062296004,
	},
}

// mulGenericE2 sets z to the E2-product of x,y, returns z
// note: do not rename, this is referenced in the x86 assembly impl
func mulGenericE2(z, x, y *E2) {
	var a, b, c fp.Element
	a.Add(&x.A0, &x.A1)
	b.Add(&y.A0, &y.A1)
	a.Mul(&a, &b)
	b.Mul(&x.A0, &y.A0)
	c.Mul(&x.A1, &y.A1)
	z.A1.Sub(&a, &b).Sub(&z.A1, &c)
	z.A0.Sub(&b, &c) // z.A0.MulByNonResidue(&c).Add(&z.A0, &b)
}

// squareGenericE2 sets z to the E2-product of x,x returns z
// note: do not rename, this is referenced in the x86 assembly impl
func squareGenericE2(z, x *E2) {
	// adapted from algo 22 https://eprint.iacr.org/2010/354.pdf
	var a, b fp.Element
	a.Add(&x.A0, &x.A1)
	b.Sub(&x.A0, &x.A1)
	a.Mul(&a, &b)
	b.Mul(&x.A0, &x.A1).Double(&b)
	z.A0.Set(&a)
	z.A1.Set(&b)
}

// MulByNonResidueInv multiplies a E2 by (9,1)^{-1}
func (z *E2) MulByNonResidueInv(x *E2) *E2 {
	z.Mul(x, &nonResInverse)
	return z
}

// Inverse sets z to the E2-inverse of x, returns z
//
// if x == 0, sets and returns z = x
func (z *E2) Inverse(x *E2) *E2 {
	// Algorithm 8 from https://eprint.iacr.org/2010/354.pdf
	var t0, t1 fp.Element
	t0.Square(&x.A0)
	t1.Square(&x.A1)
	t0.Add(&t0, &t1)
	t1.Inverse(&t0)
	z.A0.Mul(&x.A0, &t1)
	z.A1.Mul(&x.A1, &t1).Neg(&z.A1)

	return z
}

// norm sets x to the norm of z
func (z *E2) norm(x *fp.Element) {
	var tmp fp.Element
	x.Square(&z.A0)
	tmp.Square(&z.A1)
	x.Add(x, &tmp)
}

// MulBybTwistCurveCoeff multiplies by 3/(9,1)
func (z *E2) MulBybTwistCurveCoeff(x *E2) *E2 {

	var res E2
	res.MulByNonResidueInv(x)
	z.Double(&res).
		Add(&res, z)

	return z
}
