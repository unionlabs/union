// Copyright 2020 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package bls12381

// swuMapG1 is implementation of Simplified Shallue-van de Woestijne-Ulas Method
// follows the implementation at draft-irtf-cfrg-hash-to-curve-06.
func swuMapG1(u *fe) (*fe, *fe) {
	var params = swuParamsForG1
	var tv [4]*fe
	for i := 0; i < 4; i++ {
		tv[i] = new(fe)
	}
	square(tv[0], u)
	mul(tv[0], tv[0], params.z)
	square(tv[1], tv[0])
	x1 := new(fe)
	add(x1, tv[0], tv[1])
	inverse(x1, x1)
	e1 := x1.isZero()
	one := new(fe).one()
	add(x1, x1, one)
	if e1 {
		x1.set(params.zInv)
	}
	mul(x1, x1, params.minusBOverA)
	gx1 := new(fe)
	square(gx1, x1)
	add(gx1, gx1, params.a)
	mul(gx1, gx1, x1)
	add(gx1, gx1, params.b)
	x2 := new(fe)
	mul(x2, tv[0], x1)
	mul(tv[1], tv[0], tv[1])
	gx2 := new(fe)
	mul(gx2, gx1, tv[1])
	e2 := !isQuadraticNonResidue(gx1)
	x, y2 := new(fe), new(fe)
	if e2 {
		x.set(x1)
		y2.set(gx1)
	} else {
		x.set(x2)
		y2.set(gx2)
	}
	y := new(fe)
	sqrt(y, y2)
	if y.sign() != u.sign() {
		neg(y, y)
	}
	return x, y
}

// swuMapG2 is implementation of Simplified Shallue-van de Woestijne-Ulas Method
// defined at draft-irtf-cfrg-hash-to-curve-06.
func swuMapG2(e *fp2, u *fe2) (*fe2, *fe2) {
	if e == nil {
		e = newFp2()
	}
	params := swuParamsForG2
	var tv [4]*fe2
	for i := 0; i < 4; i++ {
		tv[i] = e.new()
	}
	e.square(tv[0], u)
	e.mul(tv[0], tv[0], params.z)
	e.square(tv[1], tv[0])
	x1 := e.new()
	e.add(x1, tv[0], tv[1])
	e.inverse(x1, x1)
	e1 := x1.isZero()
	e.add(x1, x1, e.one())
	if e1 {
		x1.set(params.zInv)
	}
	e.mul(x1, x1, params.minusBOverA)
	gx1 := e.new()
	e.square(gx1, x1)
	e.add(gx1, gx1, params.a)
	e.mul(gx1, gx1, x1)
	e.add(gx1, gx1, params.b)
	x2 := e.new()
	e.mul(x2, tv[0], x1)
	e.mul(tv[1], tv[0], tv[1])
	gx2 := e.new()
	e.mul(gx2, gx1, tv[1])
	e2 := !e.isQuadraticNonResidue(gx1)
	x, y2 := e.new(), e.new()
	if e2 {
		x.set(x1)
		y2.set(gx1)
	} else {
		x.set(x2)
		y2.set(gx2)
	}
	y := e.new()
	e.sqrt(y, y2)
	if y.sign() != u.sign() {
		e.neg(y, y)
	}
	return x, y
}

var swuParamsForG1 = struct {
	z           *fe
	zInv        *fe
	a           *fe
	b           *fe
	minusBOverA *fe
}{
	a:           &fe{0x2f65aa0e9af5aa51, 0x86464c2d1e8416c3, 0xb85ce591b7bd31e2, 0x27e11c91b5f24e7c, 0x28376eda6bfc1835, 0x155455c3e5071d85},
	b:           &fe{0xfb996971fe22a1e0, 0x9aa93eb35b742d6f, 0x8c476013de99c5c4, 0x873e27c3a221e571, 0xca72b5e45a52d888, 0x06824061418a386b},
	z:           &fe{0x886c00000023ffdc, 0x0f70008d3090001d, 0x77672417ed5828c3, 0x9dac23e943dc1740, 0x50553f1b9c131521, 0x078c712fbe0ab6e8},
	zInv:        &fe{0x0e8a2e8ba2e83e10, 0x5b28ba2ca4d745d1, 0x678cd5473847377a, 0x4c506dd8a8076116, 0x9bcb227d79284139, 0x0e8d3154b0ba099a},
	minusBOverA: &fe{0x052583c93555a7fe, 0x3b40d72430f93c82, 0x1b75faa0105ec983, 0x2527e7dc63851767, 0x99fffd1f34fc181d, 0x097cab54770ca0d3},
}

var swuParamsForG2 = struct {
	z           *fe2
	zInv        *fe2
	a           *fe2
	b           *fe2
	minusBOverA *fe2
}{
	a: &fe2{
		fe{0, 0, 0, 0, 0, 0},
		fe{0xe53a000003135242, 0x01080c0fdef80285, 0xe7889edbe340f6bd, 0x0b51375126310601, 0x02d6985717c744ab, 0x1220b4e979ea5467},
	},
	b: &fe2{
		fe{0x22ea00000cf89db2, 0x6ec832df71380aa4, 0x6e1b94403db5a66e, 0x75bf3c53a79473ba, 0x3dd3a569412c0a34, 0x125cdb5e74dc4fd1},
		fe{0x22ea00000cf89db2, 0x6ec832df71380aa4, 0x6e1b94403db5a66e, 0x75bf3c53a79473ba, 0x3dd3a569412c0a34, 0x125cdb5e74dc4fd1},
	},
	z: &fe2{
		fe{0x87ebfffffff9555c, 0x656fffe5da8ffffa, 0x0fd0749345d33ad2, 0xd951e663066576f4, 0xde291a3d41e980d3, 0x0815664c7dfe040d},
		fe{0x43f5fffffffcaaae, 0x32b7fff2ed47fffd, 0x07e83a49a2e99d69, 0xeca8f3318332bb7a, 0xef148d1ea0f4c069, 0x040ab3263eff0206},
	},
	zInv: &fe2{
		fe{0xacd0000000011110, 0x9dd9999dc88ccccd, 0xb5ca2ac9b76352bf, 0xf1b574bcf4bc90ce, 0x42dab41f28a77081, 0x132fc6ac14cd1e12},
		fe{0xe396ffffffff2223, 0x4fbf332fcd0d9998, 0x0c4bbd3c1aff4cc4, 0x6b9c91267926ca58, 0x29ae4da6aef7f496, 0x10692e942f195791},
	},
	minusBOverA: &fe2{
		fe{0x903c555555474fb3, 0x5f98cc95ce451105, 0x9f8e582eefe0fade, 0xc68946b6aebbd062, 0x467a4ad10ee6de53, 0x0e7146f483e23a05},
		fe{0x29c2aaaaaab85af8, 0xbf133368e30eeefa, 0xc7a27a7206cffb45, 0x9dee04ce44c9425c, 0x04a15ce53464ce83, 0x0b8fcaf5b59dac95},
	},
}
