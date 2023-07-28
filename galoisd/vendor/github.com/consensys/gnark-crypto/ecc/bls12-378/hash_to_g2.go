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

package bls12378

import (
	"github.com/consensys/gnark-crypto/ecc/bls12-378/fp"
	"github.com/consensys/gnark-crypto/ecc/bls12-378/internal/fptower"
)

// https://tools.ietf.org/html/draft-irtf-cfrg-hash-to-curve-06#section-4.1
// Shallue and van de Woestijne method, works for any elliptic curve in Weierstrass curve
func svdwMapG2(u fptower.E2) G2Affine {

	var res G2Affine

	// constants
	// sage script to find z: https://tools.ietf.org/html/draft-irtf-cfrg-hash-to-curve-06#appendix-E.1
	var z, c1, c2, c3, c4 fptower.E2
	z.A0.SetOne()
	z.A1.SetOne()
	c1.A0.SetString("605248206075306171733248481581800960739847691770924913753520744034740935903401304776283802348837311170974282940403")
	c1.A1.SetString("605248206075306171733248481581800960739847691770924913753520744034740935903401304776283802348837311170974282940416")
	c2.A0.SetString("302624103037653085866624240790900480369923845885462456876760372017370467951700652388141901174418655585487141470208")
	c2.A1.SetString("302624103037653085866624240790900480369923845885462456876760372017370467951700652388141901174418655585487141470208")
	c3.A0.SetString("296552843788751288906244499216725356684281694271241895700730864223961612014909088554048735457137528455181151573749")
	c3.A1.SetString("181388265705333345538985517067130917207305732282979825233670477511990909086507141331244586890249042878909613862256")
	c4.A0.SetString("224166002250113396938240178363629985459202848804046264353155831123978124408667149917142149018087893026286771459412")
	c4.A1.SetString("313832403150158755713536249709081979642883988325664770094418163573569374172134009883999008625323050236801480043178")

	var tv1, tv2, tv3, tv4, one, x1, gx1, x2, gx2, x3, x, gx, y fptower.E2
	one.SetOne()
	tv1.Square(&u).Mul(&tv1, &c1)
	tv2.Add(&one, &tv1)
	tv1.Sub(&one, &tv1)
	tv3.Mul(&tv2, &tv1).Inverse(&tv3)
	tv4.Mul(&u, &tv1)
	tv4.Mul(&tv4, &tv3)
	tv4.Mul(&tv4, &c3)
	x1.Sub(&c2, &tv4)
	gx1.Square(&x1)
	// 12. gx1 = gx1 + A
	gx1.Mul(&gx1, &x1)
	gx1.Add(&gx1, &bTwistCurveCoeff)
	e1 := gx1.Legendre()
	x2.Add(&c2, &tv4)
	gx2.Square(&x2)
	// 18. gx2 = gx2 + A
	gx2.Mul(&gx2, &x2)
	gx2.Add(&gx2, &bTwistCurveCoeff)
	e2 := gx2.Legendre() - e1 // 2 if is_square(gx2) AND NOT e1
	x3.Square(&tv2)
	x3.Mul(&x3, &tv3)
	x3.Square(&x3)
	x3.Mul(&x3, &c4)
	x3.Add(&x3, &z)
	if e1 == 1 {
		x.Set(&x1)
	} else {
		x.Set(&x3)
	}
	if e2 == 2 {
		x.Set(&x2)
	}
	gx.Square(&x)
	// gx = gx + A
	gx.Mul(&gx, &x)
	gx.Add(&gx, &bTwistCurveCoeff)
	y.Sqrt(&gx)
	e3 := sign0(u.A0) && sign0(y.A0)
	if !e3 {
		y.Neg(&y)
	}
	res.X.Set(&x)
	res.Y.Set(&y)

	return res
}

// MapToG2 maps an fp.Element to a point on the curve using the Shallue and van de Woestijne map
// https://tools.ietf.org/html/draft-irtf-cfrg-hash-to-curve-06#section-2.2.1
func MapToG2(t fptower.E2) G2Affine {
	res := svdwMapG2(t)
	res.ClearCofactor(&res)
	return res
}

// EncodeToG2 maps an fp.Element to a point on the curve using the Shallue and van de Woestijne map
// https://tools.ietf.org/html/draft-irtf-cfrg-hash-to-curve-06#section-2.2.2
func EncodeToG2(msg, dst []byte) (G2Affine, error) {
	var res G2Affine
	_t, err := fp.Hash(msg, dst, 2)
	if err != nil {
		return res, err
	}
	var t fptower.E2
	t.A0.Set(&_t[0])
	t.A1.Set(&_t[1])
	res = MapToG2(t)
	return res, nil
}

// HashToG2 maps an fp.Element to a point on the curve using the Shallue and van de Woestijne map
// https://tools.ietf.org/html/draft-irtf-cfrg-hash-to-curve-06#section-3
func HashToG2(msg, dst []byte) (G2Affine, error) {
	var res G2Affine
	u, err := fp.Hash(msg, dst, 4)
	if err != nil {
		return res, err
	}
	var u0, u1 fptower.E2
	u0.A0.Set(&u[0])
	u0.A1.Set(&u[1])
	u1.A0.Set(&u[2])
	u1.A1.Set(&u[3])
	Q0 := MapToG2(u0)
	Q1 := MapToG2(u1)
	var _Q0, _Q1, _res G2Jac
	_Q0.FromAffine(&Q0)
	_Q1.FromAffine(&Q1)
	_res.Set(&_Q1).AddAssign(&_Q0)
	res.FromJacobian(&_res)
	return res, nil
}

// returns false if u>-u when seen as a bigInt
func sign0(u fp.Element) bool {
	return !u.LexicographicallyLargest()
}
