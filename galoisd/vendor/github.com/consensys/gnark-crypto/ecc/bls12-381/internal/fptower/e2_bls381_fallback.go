//go:build !amd64
// +build !amd64

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

// MulByNonResidue multiplies a E2 by (1,1)
func (z *E2) MulByNonResidue(x *E2) *E2 {
	var a fp.Element
	a.Sub(&x.A0, &x.A1)
	z.A1.Add(&x.A0, &x.A1)
	z.A0.Set(&a)
	return z
}

// Mul sets z to the E2-product of x,y, returns z
func (z *E2) Mul(x, y *E2) *E2 {
	mulGenericE2(z, x, y)
	return z
}

// Square sets z to the E2-product of x,x returns z
func (z *E2) Square(x *E2) *E2 {
	squareGenericE2(z, x)
	return z
}
