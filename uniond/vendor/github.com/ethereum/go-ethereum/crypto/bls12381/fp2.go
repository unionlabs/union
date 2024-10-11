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

import (
	"errors"
	"math/big"
)

type fp2Temp struct {
	t [4]*fe
}

type fp2 struct {
	fp2Temp
}

func newFp2Temp() fp2Temp {
	t := [4]*fe{}
	for i := 0; i < len(t); i++ {
		t[i] = &fe{}
	}
	return fp2Temp{t}
}

func newFp2() *fp2 {
	t := newFp2Temp()
	return &fp2{t}
}

func (e *fp2) fromBytes(in []byte) (*fe2, error) {
	if len(in) != 96 {
		return nil, errors.New("length of input string should be 96 bytes")
	}
	c1, err := fromBytes(in[:48])
	if err != nil {
		return nil, err
	}
	c0, err := fromBytes(in[48:])
	if err != nil {
		return nil, err
	}
	return &fe2{*c0, *c1}, nil
}

func (e *fp2) toBytes(a *fe2) []byte {
	out := make([]byte, 96)
	copy(out[:48], toBytes(&a[1]))
	copy(out[48:], toBytes(&a[0]))
	return out
}

func (e *fp2) new() *fe2 {
	return new(fe2).zero()
}

func (e *fp2) zero() *fe2 {
	return new(fe2).zero()
}

func (e *fp2) one() *fe2 {
	return new(fe2).one()
}

func (e *fp2) add(c, a, b *fe2) {
	add(&c[0], &a[0], &b[0])
	add(&c[1], &a[1], &b[1])
}

func (e *fp2) addAssign(a, b *fe2) {
	addAssign(&a[0], &b[0])
	addAssign(&a[1], &b[1])
}

func (e *fp2) ladd(c, a, b *fe2) {
	ladd(&c[0], &a[0], &b[0])
	ladd(&c[1], &a[1], &b[1])
}

func (e *fp2) double(c, a *fe2) {
	double(&c[0], &a[0])
	double(&c[1], &a[1])
}

func (e *fp2) doubleAssign(a *fe2) {
	doubleAssign(&a[0])
	doubleAssign(&a[1])
}

func (e *fp2) ldouble(c, a *fe2) {
	ldouble(&c[0], &a[0])
	ldouble(&c[1], &a[1])
}

func (e *fp2) sub(c, a, b *fe2) {
	sub(&c[0], &a[0], &b[0])
	sub(&c[1], &a[1], &b[1])
}

func (e *fp2) subAssign(c, a *fe2) {
	subAssign(&c[0], &a[0])
	subAssign(&c[1], &a[1])
}

func (e *fp2) neg(c, a *fe2) {
	neg(&c[0], &a[0])
	neg(&c[1], &a[1])
}

func (e *fp2) mul(c, a, b *fe2) {
	t := e.t
	mul(t[1], &a[0], &b[0])
	mul(t[2], &a[1], &b[1])
	add(t[0], &a[0], &a[1])
	add(t[3], &b[0], &b[1])
	sub(&c[0], t[1], t[2])
	addAssign(t[1], t[2])
	mul(t[0], t[0], t[3])
	sub(&c[1], t[0], t[1])
}

func (e *fp2) mulAssign(a, b *fe2) {
	t := e.t
	mul(t[1], &a[0], &b[0])
	mul(t[2], &a[1], &b[1])
	add(t[0], &a[0], &a[1])
	add(t[3], &b[0], &b[1])
	sub(&a[0], t[1], t[2])
	addAssign(t[1], t[2])
	mul(t[0], t[0], t[3])
	sub(&a[1], t[0], t[1])
}

func (e *fp2) square(c, a *fe2) {
	t := e.t
	ladd(t[0], &a[0], &a[1])
	sub(t[1], &a[0], &a[1])
	ldouble(t[2], &a[0])
	mul(&c[0], t[0], t[1])
	mul(&c[1], t[2], &a[1])
}

func (e *fp2) squareAssign(a *fe2) {
	t := e.t
	ladd(t[0], &a[0], &a[1])
	sub(t[1], &a[0], &a[1])
	ldouble(t[2], &a[0])
	mul(&a[0], t[0], t[1])
	mul(&a[1], t[2], &a[1])
}

func (e *fp2) mulByNonResidue(c, a *fe2) {
	t := e.t
	sub(t[0], &a[0], &a[1])
	add(&c[1], &a[0], &a[1])
	c[0].set(t[0])
}

func (e *fp2) mulByB(c, a *fe2) {
	t := e.t
	double(t[0], &a[0])
	double(t[1], &a[1])
	doubleAssign(t[0])
	doubleAssign(t[1])
	sub(&c[0], t[0], t[1])
	add(&c[1], t[0], t[1])
}

func (e *fp2) inverse(c, a *fe2) {
	t := e.t
	square(t[0], &a[0])
	square(t[1], &a[1])
	addAssign(t[0], t[1])
	inverse(t[0], t[0])
	mul(&c[0], &a[0], t[0])
	mul(t[0], t[0], &a[1])
	neg(&c[1], t[0])
}

func (e *fp2) mulByFq(c, a *fe2, b *fe) {
	mul(&c[0], &a[0], b)
	mul(&c[1], &a[1], b)
}

func (e *fp2) exp(c, a *fe2, s *big.Int) {
	z := e.one()
	for i := s.BitLen() - 1; i >= 0; i-- {
		e.square(z, z)
		if s.Bit(i) == 1 {
			e.mul(z, z, a)
		}
	}
	c.set(z)
}

func (e *fp2) frobeniusMap(c, a *fe2, power uint) {
	c[0].set(&a[0])
	if power%2 == 1 {
		neg(&c[1], &a[1])
		return
	}
	c[1].set(&a[1])
}

func (e *fp2) frobeniusMapAssign(a *fe2, power uint) {
	if power%2 == 1 {
		neg(&a[1], &a[1])
		return
	}
}

func (e *fp2) sqrt(c, a *fe2) bool {
	u, x0, a1, alpha := &fe2{}, &fe2{}, &fe2{}, &fe2{}
	u.set(a)
	e.exp(a1, a, pMinus3Over4)
	e.square(alpha, a1)
	e.mul(alpha, alpha, a)
	e.mul(x0, a1, a)
	if alpha.equal(negativeOne2) {
		neg(&c[0], &x0[1])
		c[1].set(&x0[0])
		return true
	}
	e.add(alpha, alpha, e.one())
	e.exp(alpha, alpha, pMinus1Over2)
	e.mul(c, alpha, x0)
	e.square(alpha, c)
	return alpha.equal(u)
}

func (e *fp2) isQuadraticNonResidue(a *fe2) bool {
	// https://github.com/leovt/constructible/wiki/Taking-Square-Roots-in-quadratic-extension-Fields
	c0, c1 := new(fe), new(fe)
	square(c0, &a[0])
	square(c1, &a[1])
	add(c1, c1, c0)
	return isQuadraticNonResidue(c1)
}
