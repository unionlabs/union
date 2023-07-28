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

package bls24317

import (
	"github.com/consensys/gnark-crypto/ecc/bls24-317/fp"
	"github.com/consensys/gnark-crypto/ecc/bls24-317/internal/fptower"
)

// https://tools.ietf.org/html/draft-irtf-cfrg-hash-to-curve-06#section-4.1
// Shallue and van de Woestijne method, works for any elliptic curve in Weierstrass curve
func svdwMapG2(u fptower.E4) G2Affine {

	var res G2Affine

	// constants
	// sage script to find z: https://tools.ietf.org/html/draft-irtf-cfrg-hash-to-curve-06#appendix-E.1
	var z, c1, c2, c3, c4 fptower.E4
	z.B0.A0.SetOne()
	z.B1.A0.SetOne()
	c1.B0.A0.SetString("4")
	c1.B0.A1.SetString("3")
	c1.B1.A0.SetString("8")
	c1.B1.A1.SetOne()
	c2.B0.A0.SetString("68196535552147955757549882954137028530972556060709796988605069651952986598616012809013078365525")
	c2.B1.A0.SetString("68196535552147955757549882954137028530972556060709796988605069651952986598616012809013078365525")
	c3.B0.A0.SetString("25710473854271083900266173357439657657737168361084633536126117969329631844210973452609964652920")
	c3.B0.A1.SetString("97726383423614678023078817471231282096435936120492353286347028233584612721291548146704405526838")
	c3.B1.A0.SetString("31017010388646627031356727289998252571046265059138887207088052022600004087627603083210545186274")
	c3.B1.A1.SetString("74637498440051236880963727555084502172097851690589624852957691761203766904143491322222931488114")
	c4.B0.A0.SetString("136393071104295911515099765908274057061945112121419593977210139303905973197232025618026156731039")
	c4.B0.A1.SetString("90928714069530607676733177272182704707963408080946395984806759535937315464821350412017437820690")
	c4.B1.A0.SetString("90928714069530607676733177272182704707963408080946395984806759535937315464821350412017437820710")
	c4.B1.A1.SetString("90928714069530607676733177272182704707963408080946395984806759535937315464821350412017437820706")

	var tv1, tv2, tv3, tv4, one, x1, gx1, x2, gx2, x3, x, gx, y fptower.E4
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
	e3 := sign0(u.B0.A0) && sign0(y.B0.A0)
	if !e3 {
		y.Neg(&y)
	}
	res.X.Set(&x)
	res.Y.Set(&y)

	return res
}

// MapToG2 maps an fp.Element to a point on the curve using the Shallue and van de Woestijne map
// https://tools.ietf.org/html/draft-irtf-cfrg-hash-to-curve-06#section-2.2.1
func MapToG2(t fptower.E4) G2Affine {
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
	var t fptower.E4
	t.B0.A0.Set(&_t[0])
	t.B1.A0.Set(&_t[1])
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
	var u0, u1 fptower.E4
	u0.B0.A0.Set(&u[0])
	u0.B1.A0.Set(&u[1])
	u1.B0.A0.Set(&u[2])
	u1.B1.A0.Set(&u[3])
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
