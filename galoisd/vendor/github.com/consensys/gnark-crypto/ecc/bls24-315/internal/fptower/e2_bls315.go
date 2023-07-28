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
	"github.com/consensys/gnark-crypto/ecc/bls24-315/fp"
)

// used with !amd64, make staticcheck happier.
var (
	_ = mulGenericE2
)

func mulGenericE2(z, x, y *E2) {
	var a, b, c fp.Element
	a.Add(&x.A0, &x.A1)
	b.Add(&y.A0, &y.A1)
	a.Mul(&a, &b)
	b.Mul(&x.A0, &y.A0)
	c.Mul(&x.A1, &y.A1)
	z.A1.Sub(&a, &b).Sub(&z.A1, &c)
	fp.MulBy13(&c)
	z.A0.Add(&c, &b)
}

// Square sets z to the E2-product of x,x returns z
func (z *E2) Square(x *E2) *E2 {
	//algo 22 https://eprint.iacr.org/2010/354.pdf
	var c0, c2 fp.Element
	c0 = x.A1
	fp.MulBy13(&c0)
	c2.Add(&c0, &x.A0)
	c0.Add(&x.A0, &x.A1)
	c0.Mul(&c0, &c2) // (x1+x2)*(x1+(u**2)x2)
	z.A1.Mul(&x.A0, &x.A1).Double(&z.A1)
	c2.Double(&z.A1).Double(&c2).Double(&c2) // 8 z.A1
	z.A0.Sub(&c0, &c2).Add(&z.A0, &z.A1)

	return z
}

// MulByNonResidueInv multiplies a E2 by (0,1)^{-1}
func (z *E2) MulByNonResidueInv(x *E2) *E2 {
	a := x.A1
	thirteeninv := fp.Element{
		14835018474091022805,
		4059211274438447823,
		17174191230683291349,
		5795645494093750226,
		179263826259076473,
	}
	z.A1.Mul(&x.A0, &thirteeninv)
	z.A0 = a
	return z
}

// Inverse sets z to the E2-inverse of x, returns z
//
// if x == 0, sets and returns z = x
func (z *E2) Inverse(x *E2) *E2 {
	// Algorithm 8 from https://eprint.iacr.org/2010/354.pdf
	//var a, b, t0, t1, tmp fp.Element
	var t0, t1 fp.Element
	a := &x.A0 // creating the buffers a, b is faster than querying &x.A0, &x.A1 in the functions call below
	b := &x.A1
	t0.Square(a)
	t1.Square(b)
	fp.MulBy13(&t1)
	t0.Sub(&t0, &t1)
	t1.Inverse(&t0)
	z.A0.Mul(a, &t1)
	z.A1.Mul(b, &t1).Neg(&z.A1)

	return z
}

// norm sets x to the norm of z
func (z *E2) norm(x *fp.Element) {
	var tmp0, tmp1 fp.Element
	x.Square(&z.A1)
	tmp0.Double(x).Double(&tmp0)
	tmp1.Double(&tmp0).Add(&tmp1, &tmp0).Add(&tmp1, x)
	x.Square(&z.A0).Sub(x, &tmp1)
}
