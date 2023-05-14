// Copyright 2020 ConsenSys AG
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://wwwApache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package fptower

import (
	"github.com/consensys/gnark-crypto/ecc/bls24-317/fp"
)

// Frobenius sets z in E4 to x^q, returns z
func (z *E4) Frobenius(x *E4) *E4 {

	var t [2]E2

	// (u+1)^((p-1)/2)
	frobCoeffA := fp.Element{
		9105899941191937594,
		12184646476002172414,
		7321502160543123086,
		6035789969373957152,
		33356129992723801,
	}
	t[0].Conjugate(&x.B0)
	t[1].Conjugate(&x.B1).MulByNonResidue(&t[1]).MulByElement(&t[1], &frobCoeffA)

	z.B0 = t[0]
	z.B1 = t[1]

	return z
}

// Frobenius set z to Frobenius(x), return z
func (z *E24) Frobenius(x *E24) *E24 {
	var t [12]E2
	var frobCoeff1, frobCoeff3, frobCoeff4 E2
	var frobCoeff5 E4
	frobCoeff0 := fp.Element{
		9105899941191937594,
		12184646476002172414,
		7321502160543123086,
		6035789969373957152,
		33356129992723801,
	}
	frobCoeff1.A0 = fp.Element{
		2497921667672846212,
		17809570827777133368,
		182875672615776868,
		8141251192822526532,
		541317525405759051,
	}
	frobCoeff1.A1 = fp.Element{
		7685049513262119719,
		16125960441679836230,
		1445846185330098657,
		9337154780097699317,
		636596026397922017,
	}
	frobCoeff2 := fp.Element{
		12480069551231893946,
		13145883874818081857,
		17830246446819370377,
		1479988843601531044,
		728901529575278942,
	}
	frobCoeff3.A1 = fp.Element{
		9386911782805384298,
		2712061974843046954,
		6996308026534275575,
		4433323304681870390,
		1128774284724624429,
	}
	frobCoeff4.A0 = fp.Element{
		11603821608864783806,
		11547473230069754166,
		7504377833158899955,
		14177041162196483684,
		574673655398482852,
	}
	frobCoeff4.A1 = fp.Element{
		17025893645779733741,
		3941313965677663815,
		12571088098496527187,
		3301364810723742164,
		603239896405198216,
	}
	frobCoeff5.B1.A0 = fp.Element{
		16800646172467814206,
		346280723711515920,
		16347809494184080414,
		7631911081188324677,
		9467827575491553,
	}
	frobCoeff5.B1.A1 = fp.Element{
		11829069082176703341,
		15142506472035902061,
		3727656437471346728,
		9846494891731901171,
		1168445724228189515,
	}

	t[0].Conjugate(&x.D0.C0.B0)
	t[1].Conjugate(&x.D0.C0.B1).MulByNonResidue(&t[1]).MulByElement(&t[1], &frobCoeff0)
	t[2].Conjugate(&x.D0.C1.B0).Mul(&t[2], &frobCoeff1)
	t[3].Conjugate(&x.D0.C1.B1).MulByElement(&t[3], &frobCoeff2)
	t[4].Conjugate(&x.D0.C2.B0).Mul(&t[4], &frobCoeff3)
	t[5].Conjugate(&x.D0.C2.B1).Mul(&t[5], &frobCoeff4)
	t[6].Conjugate(&x.D1.C0.B0)
	t[7].Conjugate(&x.D1.C0.B1).MulByNonResidue(&t[7]).MulByElement(&t[7], &frobCoeff0)
	t[8].Conjugate(&x.D1.C1.B0).Mul(&t[8], &frobCoeff1)
	t[9].Conjugate(&x.D1.C1.B1).MulByElement(&t[9], &frobCoeff2)
	t[10].Conjugate(&x.D1.C2.B0).Mul(&t[10], &frobCoeff3)
	t[11].Conjugate(&x.D1.C2.B1).Mul(&t[11], &frobCoeff4)

	z.D0.C0.B0 = t[0]
	z.D0.C0.B1 = t[1]
	z.D0.C1.B0 = t[2]
	z.D0.C1.B1 = t[3]
	z.D0.C2.B0 = t[4]
	z.D0.C2.B1 = t[5]
	z.D1.C0.B0 = t[6]
	z.D1.C0.B1 = t[7]
	z.D1.C1.B0 = t[8]
	z.D1.C1.B1 = t[9]
	z.D1.C2.B0 = t[10]
	z.D1.C2.B1 = t[11]

	z.D1.C0.Mul(&z.D1.C0, &frobCoeff5)
	z.D1.C1.Mul(&z.D1.C1, &frobCoeff5)
	z.D1.C2.Mul(&z.D1.C2, &frobCoeff5)

	return z
}

// FrobeniusSquare set z to Frobenius^2(x), return z
func (z *E24) FrobeniusSquare(x *E24) *E24 {
	var t [12]E4
	var frobCoeff3 E4
	frobCoeff0 := fp.Element{
		796059398129581633,
		12776725220904371028,
		13079157905121151567,
		13045082668238355458,
		49139267079056639,
	}
	frobCoeff1 := fp.Element{
		16149645703412623601,
		2342903320929336124,
		2245219484836056765,
		15998417129318694804,
		449012022228402126,
	}
	frobCoeff3.B0.A1 = fp.Element{
		12480069551231893946,
		13145883874818081857,
		17830246446819370377,
		1479988843601531044,
		728901529575278942,
	}
	t[0].Conjugate(&x.D0.C0)
	t[1].Conjugate(&x.D0.C1).MulByElement(&t[1], &frobCoeff0)
	t[2].Conjugate(&x.D0.C2).MulByElement(&t[2], &frobCoeff1)
	t[3].Conjugate(&x.D1.C0).Mul(&t[3], &frobCoeff3)
	t[4].Conjugate(&x.D1.C1).MulByElement(&t[4], &frobCoeff0).Mul(&t[4], &frobCoeff3)
	t[5].Conjugate(&x.D1.C2).MulByElement(&t[5], &frobCoeff1).Mul(&t[5], &frobCoeff3)

	z.D0.C0 = t[0]
	z.D0.C1 = t[1]
	z.D0.C2 = t[2]
	z.D1.C0 = t[3]
	z.D1.C1 = t[4]
	z.D1.C2 = t[5]

	return z
}

// FrobeniusQuad set z to Frobenius^4(x), return z
func (z *E24) FrobeniusQuad(x *E24) *E24 {
	var t [12]E4
	frobCoeff0 := fp.Element{
		16149645703412623601,
		2342903320929336124,
		2245219484836056765,
		15998417129318694804,
		449012022228402126,
	}
	frobCoeff1 := fp.Element{
		9386911782805384298,
		2712061974843046954,
		6996308026534275575,
		4433323304681870390,
		1128774284724624429,
	}
	frobCoeff2 := fp.Element{
		796059398129581633,
		12776725220904371028,
		13079157905121151567,
		13045082668238355458,
		49139267079056639,
	}

	t[0].Set(&x.D0.C0)
	t[1].MulByElement(&x.D0.C1, &frobCoeff0)
	t[2].MulByElement(&x.D0.C2, &frobCoeff1)
	t[3].MulByElement(&x.D1.C0, &frobCoeff2)
	t[4].MulByElement(&x.D1.C1, &frobCoeff0).MulByElement(&t[4], &frobCoeff2)
	t[5].MulByElement(&x.D1.C2, &frobCoeff1).MulByElement(&t[5], &frobCoeff2)

	z.D0.C0 = t[0]
	z.D0.C1 = t[1]
	z.D0.C2 = t[2]
	z.D1.C0 = t[3]
	z.D1.C1 = t[4]
	z.D1.C2 = t[5]

	return z
}
