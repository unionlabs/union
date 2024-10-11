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

// MulBybTwistCurveCoeff multiplies by 1/(0,1)
func (z *E4) MulBybTwistCurveCoeff(x *E4) *E4 {

	var res E4
	res.B0.Set(&x.B1)
	res.B1.MulByNonResidueInv(&x.B0)
	z.Set(&res)

	return z
}
