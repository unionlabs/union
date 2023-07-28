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

import "github.com/consensys/gnark-crypto/ecc/bls12-381/fp"

// used with !amd64, make staticcheck happier.
var (
	_ = mulGenericE2
	_ = squareGenericE2
)

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
	z.A0.Sub(&b, &c)
}

// Square sets z to the E2-product of x,x returns z
func squareGenericE2(z, x *E2) *E2 {
	// adapted from algo 22 https://eprint.iacr.org/2010/354.pdf
	var a, b fp.Element
	a.Add(&x.A0, &x.A1)
	b.Sub(&x.A0, &x.A1)
	a.Mul(&a, &b)
	b.Mul(&x.A0, &x.A1).Double(&b)
	z.A0.Set(&a)
	z.A1.Set(&b)
	return z
}

var twoInv = fp.Element{
	1730508156817200468,
	9606178027640717313,
	7150789853162776431,
	7936136305760253186,
	15245073033536294050,
	1728177566264616342,
}

// MulByNonResidueInv multiplies a E2 by (1,1)^{-1}
func (z *E2) MulByNonResidueInv(x *E2) *E2 {

	var tmp fp.Element
	tmp.Add(&x.A0, &x.A1)
	z.A1.Sub(&x.A1, &x.A0).Mul(&z.A1, &twoInv)
	z.A0.Set(&tmp).Mul(&z.A0, &twoInv)

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

// MulBybTwistCurveCoeff multiplies by 4(1,1)
func (z *E2) MulBybTwistCurveCoeff(x *E2) *E2 {

	var res E2
	res.A0.Sub(&x.A0, &x.A1)
	res.A1.Add(&x.A0, &x.A1)
	z.Double(&res).
		Double(z)

	return z
}
