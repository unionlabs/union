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

import "github.com/consensys/gnark-crypto/ecc/bls12-377/fp"

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
	// 92949345220277864758624960506473182677953048909283248980960104381795901929519566951595905490535835115111760994353
	b := fp.Element{
		7981638599956744862,
		11830407261614897732,
		6308788297503259939,
		10596665404780565693,
		11693741422477421038,
		61545186993886319,
	}
	z.A0.Mul(&x.A0, &b)
	z.A1.Mul(&x.A1, &b)
	return z
}

// MulByNonResidue1Power2 set z=x*(0,1)^(2*(p^1-1)/6) and return z
func (z *E2) MulByNonResidue1Power2(x *E2) *E2 {
	// 80949648264912719408558363140637477264845294720710499478137287262712535938301461879813459410946
	b := fp.Element{
		6382252053795993818,
		1383562296554596171,
		11197251941974877903,
		6684509567199238270,
		6699184357838251020,
		19987743694136192,
	}
	z.A0.Mul(&x.A0, &b)
	z.A1.Mul(&x.A1, &b)
	return z
}

// MulByNonResidue1Power3 set z=x*(0,1)^(3*(p^1-1)/6) and return z
func (z *E2) MulByNonResidue1Power3(x *E2) *E2 {
	// 216465761340224619389371505802605247630151569547285782856803747159100223055385581585702401816380679166954762214499
	b := fp.Element{
		10965161018967488287,
		18251363109856037426,
		7036083669251591763,
		16109345360066746489,
		4679973768683352764,
		96952949334633821,
	}
	z.A0.Mul(&x.A0, &b)
	z.A1.Mul(&x.A1, &b)
	return z
}

// MulByNonResidue1Power4 set z=x*(0,1)^(4*(p^1-1)/6) and return z
func (z *E2) MulByNonResidue1Power4(x *E2) *E2 {
	// 80949648264912719408558363140637477264845294720710499478137287262712535938301461879813459410945
	b := fp.Element{
		15766275933608376691,
		15635974902606112666,
		1934946774703877852,
		18129354943882397960,
		15437979634065614942,
		101285514078273488,
	}
	z.A0.Mul(&x.A0, &b)
	z.A1.Mul(&x.A1, &b)
	return z
}

// MulByNonResidue1Power5 set z=x*(0,1)^(5*(p^1-1)/6) and return z
func (z *E2) MulByNonResidue1Power5(x *E2) *E2 {
	// 123516416119946754630746545296132064952198520638002533875843642777304321125866014634106496325844844051843001220146
	b := fp.Element{
		2983522419010743425,
		6420955848241139694,
		727295371748331824,
		5512679955286180796,
		11432976419915483342,
		35407762340747501,
	}
	z.A0.Mul(&x.A0, &b)
	z.A1.Mul(&x.A1, &b)
	return z
}

// MulByNonResidue2Power1 set z=x*(0,1)^(1*(p^2-1)/6) and return z
func (z *E2) MulByNonResidue2Power1(x *E2) *E2 {
	// 80949648264912719408558363140637477264845294720710499478137287262712535938301461879813459410946
	b := fp.Element{
		6382252053795993818,
		1383562296554596171,
		11197251941974877903,
		6684509567199238270,
		6699184357838251020,
		19987743694136192,
	}
	z.A0.Mul(&x.A0, &b)
	z.A1.Mul(&x.A1, &b)
	return z
}

// MulByNonResidue2Power2 set z=x*(0,1)^(2*(p^2-1)/6) and return z
func (z *E2) MulByNonResidue2Power2(x *E2) *E2 {
	// 80949648264912719408558363140637477264845294720710499478137287262712535938301461879813459410945
	b := fp.Element{
		15766275933608376691,
		15635974902606112666,
		1934946774703877852,
		18129354943882397960,
		15437979634065614942,
		101285514078273488,
	}
	z.A0.Mul(&x.A0, &b)
	z.A1.Mul(&x.A1, &b)
	return z
}

// MulByNonResidue2Power3 set z=x*(0,1)^(3*(p^2-1)/6) and return z
func (z *E2) MulByNonResidue2Power3(x *E2) *E2 {
	// 258664426012969094010652733694893533536393512754914660539884262666720468348340822774968888139573360124440321458176
	b := fp.Element{
		9384023879812382873,
		14252412606051516495,
		9184438906438551565,
		11444845376683159689,
		8738795276227363922,
		81297770384137296,
	}
	z.A0.Mul(&x.A0, &b)
	z.A1.Mul(&x.A1, &b)
	return z
}

// MulByNonResidue2Power4 set z=x*(0,1)^(4*(p^2-1)/6) and return z
func (z *E2) MulByNonResidue2Power4(x *E2) *E2 {
	// 258664426012969093929703085429980814127835149614277183275038967946009968870203535512256352201271898244626862047231
	b := fp.Element{
		3203870859294639911,
		276961138506029237,
		9479726329337356593,
		13645541738420943632,
		7584832609311778094,
		101110569012358506,
	}
	z.A0.Mul(&x.A0, &b)
	z.A1.Mul(&x.A1, &b)
	return z
}

// MulByNonResidue2Power5 set z=x*(0,1)^(5*(p^2-1)/6) and return z
func (z *E2) MulByNonResidue2Power5(x *E2) *E2 {
	// 258664426012969093929703085429980814127835149614277183275038967946009968870203535512256352201271898244626862047232
	b := fp.Element{
		12266591053191808654,
		4471292606164064357,
		295287422898805027,
		2200696361737783943,
		17292781406793965788,
		19812798628221209,
	}
	z.A0.Mul(&x.A0, &b)
	z.A1.Mul(&x.A1, &b)
	return z
}
