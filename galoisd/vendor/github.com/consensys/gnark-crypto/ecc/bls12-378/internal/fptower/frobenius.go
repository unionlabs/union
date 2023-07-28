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

import "github.com/consensys/gnark-crypto/ecc/bls12-378/fp"

// Frobenius set z to Frobenius(x), return z
func (z *E12) Frobenius(x *E12) *E12 {
	// Algorithm 28 from https://eprint.iacr.org/2010/354.pdf (beware typos!)
	var t [6]E2

	// Frobenius acts on fp2 by conjugation
	t[0].Conjugate(&x.C0.B0)
	t[1].Conjugate(&x.C0.B1)
	t[2].Conjugate(&x.C0.B2)
	t[3].Conjugate(&x.C1.B0)
	t[4].Conjugate(&x.C1.B1)
	t[5].Conjugate(&x.C1.B2)

	t[1].MulByNonResidue1Power2(&t[1])
	t[2].MulByNonResidue1Power4(&t[2])
	t[3].MulByNonResidue1Power1(&t[3])
	t[4].MulByNonResidue1Power3(&t[4])
	t[5].MulByNonResidue1Power5(&t[5])

	z.C0.B0 = t[0]
	z.C0.B1 = t[1]
	z.C0.B2 = t[2]
	z.C1.B0 = t[3]
	z.C1.B1 = t[4]
	z.C1.B2 = t[5]

	return z
}

// FrobeniusSquare set z to Frobenius^2(x), and return z
func (z *E12) FrobeniusSquare(x *E12) *E12 {
	// Algorithm 29 from https://eprint.iacr.org/2010/354.pdf (beware typos!)
	var t [6]E2

	t[1].MulByNonResidue2Power2(&x.C0.B1)
	t[2].MulByNonResidue2Power4(&x.C0.B2)
	t[3].MulByNonResidue2Power1(&x.C1.B0)
	t[4].MulByNonResidue2Power3(&x.C1.B1)
	t[5].MulByNonResidue2Power5(&x.C1.B2)

	z.C0.B0 = x.C0.B0
	z.C0.B1 = t[1]
	z.C0.B2 = t[2]
	z.C1.B0 = t[3]
	z.C1.B1 = t[4]
	z.C1.B2 = t[5]

	return z
}

// MulByNonResidue1Power1 set z=x*(0,1)^(1*(p^1-1)/6) and return z
func (z *E2) MulByNonResidue1Power1(x *E2) *E2 {
	b := fp.Element{
		9424304261440581301,
		15622662318784019360,
		5704744713545767383,
		7376930514650170538,
		2328236726423359970,
		256435709676028998,
	}
	z.A0.Mul(&x.A0, &b)
	z.A1.Mul(&x.A1, &b)
	return z
}

// MulByNonResidue1Power2 set z=x*(0,1)^(2*(p^1-1)/6) and return z
func (z *E2) MulByNonResidue1Power2(x *E2) *E2 {
	b := fp.Element{
		1263886799460835702,
		3481310115429540252,
		1430516082310201521,
		10760454131030452261,
		15881431079209118478,
		56234068425139279,
	}
	z.A0.Mul(&x.A0, &b)
	z.A1.Mul(&x.A1, &b)
	return z
}

// MulByNonResidue1Power3 set z=x*(0,1)^(3*(p^1-1)/6) and return z
func (z *E2) MulByNonResidue1Power3(x *E2) *E2 {
	b := fp.Element{
		6315024805150803022,
		16048962212196301574,
		10554832649293981783,
		14109148363171599309,
		4153042273623539198,
		250647462785784749,
	}
	z.A0.Mul(&x.A0, &b)
	z.A1.Mul(&x.A1, &b)
	return z
}

// MulByNonResidue1Power4 set z=x*(0,1)^(4*(p^1-1)/6) and return z
func (z *E2) MulByNonResidue1Power4(x *E2) *E2 {
	b := fp.Element{
		18229265454137549239,
		11882161740266529218,
		12635080069402934820,
		1928134709134316785,
		2524500224088382290,
		27735392882694645,
	}
	z.A0.Mul(&x.A0, &b)
	z.A1.Mul(&x.A1, &b)
	return z
}

// MulByNonResidue1Power5 set z=x*(0,1)^(5*(p^1-1)/6) and return z
func (z *E2) MulByNonResidue1Power5(x *E2) *E2 {
	b := fp.Element{
		7935976750720062874,
		15312939023531261798,
		15806716224795225087,
		16245402142124945993,
		7862827682069246910,
		277569374620018935,
	}
	z.A0.Mul(&x.A0, &b)
	z.A1.Mul(&x.A1, &b)
	return z
}

// MulByNonResidue2Power1 set z=x*(0,1)^(1*(p^2-1)/6) and return z
func (z *E2) MulByNonResidue2Power1(x *E2) *E2 {
	b := fp.Element{
		1263886799460835702,
		3481310115429540252,
		1430516082310201521,
		10760454131030452261,
		15881431079209118478,
		56234068425139279,
	}
	z.A0.Mul(&x.A0, &b)
	z.A1.Mul(&x.A1, &b)
	return z
}

// MulByNonResidue2Power2 set z=x*(0,1)^(2*(p^2-1)/6) and return z
func (z *E2) MulByNonResidue2Power2(x *E2) *E2 {
	b := fp.Element{
		18229265454137549239,
		11882161740266529218,
		12635080069402934820,
		1928134709134316785,
		2524500224088382290,
		27735392882694645,
	}
	z.A0.Mul(&x.A0, &b)
	z.A1.Mul(&x.A1, &b)
	return z
}

// MulByNonResidue2Power3 set z=x*(0,1)^(3*(p^2-1)/6) and return z
func (z *E2) MulByNonResidue2Power3(x *E2) *E2 {
	b := fp.Element{
		9563890787977003074,
		4840746681246416935,
		3714448202430192371,
		680864871707381747,
		11127835353457883110,
		254858945967818549,
	}
	z.A0.Mul(&x.A0, &b)
	z.A1.Mul(&x.A1, &b)
	return z
}

// MulByNonResidue2Power4 set z=x*(0,1)^(4*(p^2-1)/6) and return z
func (z *E2) MulByNonResidue2Power4(x *E2) *E2 {
	b := fp.Element{
		9781369407549005451,
		11405329014689439332,
		9526112206736809166,
		17199474236282616577,
		8603335129369500819,
		227123553085123904,
	}
	z.A0.Mul(&x.A0, &b)
	z.A1.Mul(&x.A1, &b)
	return z
}

// MulByNonResidue2Power5 set z=x*(0,1)^(5*(p^2-1)/6) and return z
func (z *E2) MulByNonResidue2Power5(x *E2) *E2 {
	b := fp.Element{
		11262734826581843530,
		3004477389852450365,
		16768292293353627483,
		7585049584469200436,
		3513521910780685392,
		255622228627568539,
	}
	z.A0.Mul(&x.A0, &b)
	z.A1.Mul(&x.A1, &b)
	return z
}
