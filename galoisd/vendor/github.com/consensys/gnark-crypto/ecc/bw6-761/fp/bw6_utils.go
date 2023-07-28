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

package fp

// MulByNonResidue multiplies a fp.Element by -4
func (z *Element) MulByNonResidue(x *Element) *Element {
	z.Double(x).Double(z).Neg(z)
	return z
}

// MulByNonResidueInv multiplies a fp.Element by (-4)**-1
func (z *Element) MulByNonResidueInv(x *Element) *Element {

	nrInv := Element{
		8571757465769615091,
		6221412002326125864,
		16781361031322833010,
		18148962537424854844,
		6497335359600054623,
		17630955688667215145,
		15638647242705587201,
		830917065158682257,
		6848922060227959954,
		4142027113657578586,
		12050453106507568375,
		55644342162350184,
	}

	z.Mul(x, &nrInv)

	return z
}
