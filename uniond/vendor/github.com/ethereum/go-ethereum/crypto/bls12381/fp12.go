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

type fp12 struct {
	fp12temp
	fp6 *fp6
}

type fp12temp struct {
	t2  [9]*fe2
	t6  [5]*fe6
	t12 *fe12
}

func newFp12Temp() fp12temp {
	t2 := [9]*fe2{}
	t6 := [5]*fe6{}
	for i := 0; i < len(t2); i++ {
		t2[i] = &fe2{}
	}
	for i := 0; i < len(t6); i++ {
		t6[i] = &fe6{}
	}
	return fp12temp{t2, t6, &fe12{}}
}

func newFp12(fp6 *fp6) *fp12 {
	t := newFp12Temp()
	if fp6 == nil {
		return &fp12{t, newFp6(nil)}
	}
	return &fp12{t, fp6}
}

func (e *fp12) fp2() *fp2 {
	return e.fp6.fp2
}

func (e *fp12) fromBytes(in []byte) (*fe12, error) {
	if len(in) != 576 {
		return nil, errors.New("input string should be larger than 96 bytes")
	}
	fp6 := e.fp6
	c1, err := fp6.fromBytes(in[:288])
	if err != nil {
		return nil, err
	}
	c0, err := fp6.fromBytes(in[288:])
	if err != nil {
		return nil, err
	}
	return &fe12{*c0, *c1}, nil
}

func (e *fp12) toBytes(a *fe12) []byte {
	fp6 := e.fp6
	out := make([]byte, 576)
	copy(out[:288], fp6.toBytes(&a[1]))
	copy(out[288:], fp6.toBytes(&a[0]))
	return out
}

func (e *fp12) new() *fe12 {
	return new(fe12)
}

func (e *fp12) zero() *fe12 {
	return new(fe12)
}

func (e *fp12) one() *fe12 {
	return new(fe12).one()
}

func (e *fp12) add(c, a, b *fe12) {
	fp6 := e.fp6
	fp6.add(&c[0], &a[0], &b[0])
	fp6.add(&c[1], &a[1], &b[1])
}

func (e *fp12) double(c, a *fe12) {
	fp6 := e.fp6
	fp6.double(&c[0], &a[0])
	fp6.double(&c[1], &a[1])
}

func (e *fp12) sub(c, a, b *fe12) {
	fp6 := e.fp6
	fp6.sub(&c[0], &a[0], &b[0])
	fp6.sub(&c[1], &a[1], &b[1])
}

func (e *fp12) neg(c, a *fe12) {
	fp6 := e.fp6
	fp6.neg(&c[0], &a[0])
	fp6.neg(&c[1], &a[1])
}

func (e *fp12) conjugate(c, a *fe12) {
	fp6 := e.fp6
	c[0].set(&a[0])
	fp6.neg(&c[1], &a[1])
}

func (e *fp12) square(c, a *fe12) {
	fp6, t := e.fp6, e.t6
	fp6.add(t[0], &a[0], &a[1])
	fp6.mul(t[2], &a[0], &a[1])
	fp6.mulByNonResidue(t[1], &a[1])
	fp6.addAssign(t[1], &a[0])
	fp6.mulByNonResidue(t[3], t[2])
	fp6.mulAssign(t[0], t[1])
	fp6.subAssign(t[0], t[2])
	fp6.sub(&c[0], t[0], t[3])
	fp6.double(&c[1], t[2])
}

func (e *fp12) cyclotomicSquare(c, a *fe12) {
	t, fp2 := e.t2, e.fp2()
	e.fp4Square(t[3], t[4], &a[0][0], &a[1][1])
	fp2.sub(t[2], t[3], &a[0][0])
	fp2.doubleAssign(t[2])
	fp2.add(&c[0][0], t[2], t[3])
	fp2.add(t[2], t[4], &a[1][1])
	fp2.doubleAssign(t[2])
	fp2.add(&c[1][1], t[2], t[4])
	e.fp4Square(t[3], t[4], &a[1][0], &a[0][2])
	e.fp4Square(t[5], t[6], &a[0][1], &a[1][2])
	fp2.sub(t[2], t[3], &a[0][1])
	fp2.doubleAssign(t[2])
	fp2.add(&c[0][1], t[2], t[3])
	fp2.add(t[2], t[4], &a[1][2])
	fp2.doubleAssign(t[2])
	fp2.add(&c[1][2], t[2], t[4])
	fp2.mulByNonResidue(t[3], t[6])
	fp2.add(t[2], t[3], &a[1][0])
	fp2.doubleAssign(t[2])
	fp2.add(&c[1][0], t[2], t[3])
	fp2.sub(t[2], t[5], &a[0][2])
	fp2.doubleAssign(t[2])
	fp2.add(&c[0][2], t[2], t[5])
}

func (e *fp12) mul(c, a, b *fe12) {
	t, fp6 := e.t6, e.fp6
	fp6.mul(t[1], &a[0], &b[0])
	fp6.mul(t[2], &a[1], &b[1])
	fp6.add(t[0], t[1], t[2])
	fp6.mulByNonResidue(t[2], t[2])
	fp6.add(t[3], t[1], t[2])
	fp6.add(t[1], &a[0], &a[1])
	fp6.add(t[2], &b[0], &b[1])
	fp6.mulAssign(t[1], t[2])
	c[0].set(t[3])
	fp6.sub(&c[1], t[1], t[0])
}

func (e *fp12) mulAssign(a, b *fe12) {
	t, fp6 := e.t6, e.fp6
	fp6.mul(t[1], &a[0], &b[0])
	fp6.mul(t[2], &a[1], &b[1])
	fp6.add(t[0], t[1], t[2])
	fp6.mulByNonResidue(t[2], t[2])
	fp6.add(t[3], t[1], t[2])
	fp6.add(t[1], &a[0], &a[1])
	fp6.add(t[2], &b[0], &b[1])
	fp6.mulAssign(t[1], t[2])
	a[0].set(t[3])
	fp6.sub(&a[1], t[1], t[0])
}

func (e *fp12) fp4Square(c0, c1, a0, a1 *fe2) {
	t, fp2 := e.t2, e.fp2()
	fp2.square(t[0], a0)
	fp2.square(t[1], a1)
	fp2.mulByNonResidue(t[2], t[1])
	fp2.add(c0, t[2], t[0])
	fp2.add(t[2], a0, a1)
	fp2.squareAssign(t[2])
	fp2.subAssign(t[2], t[0])
	fp2.sub(c1, t[2], t[1])
}

func (e *fp12) inverse(c, a *fe12) {
	fp6, t := e.fp6, e.t6
	fp6.square(t[0], &a[0])
	fp6.square(t[1], &a[1])
	fp6.mulByNonResidue(t[1], t[1])
	fp6.sub(t[1], t[0], t[1])
	fp6.inverse(t[0], t[1])
	fp6.mul(&c[0], &a[0], t[0])
	fp6.mulAssign(t[0], &a[1])
	fp6.neg(&c[1], t[0])
}

func (e *fp12) mulBy014Assign(a *fe12, c0, c1, c4 *fe2) {
	fp2, fp6, t, t2 := e.fp2(), e.fp6, e.t6, e.t2[0]
	fp6.mulBy01(t[0], &a[0], c0, c1)
	fp6.mulBy1(t[1], &a[1], c4)
	fp2.add(t2, c1, c4)
	fp6.add(t[2], &a[1], &a[0])
	fp6.mulBy01Assign(t[2], c0, t2)
	fp6.subAssign(t[2], t[0])
	fp6.sub(&a[1], t[2], t[1])
	fp6.mulByNonResidue(t[1], t[1])
	fp6.add(&a[0], t[1], t[0])
}

func (e *fp12) exp(c, a *fe12, s *big.Int) {
	z := e.one()
	for i := s.BitLen() - 1; i >= 0; i-- {
		e.square(z, z)
		if s.Bit(i) == 1 {
			e.mul(z, z, a)
		}
	}
	c.set(z)
}

func (e *fp12) cyclotomicExp(c, a *fe12, s *big.Int) {
	z := e.one()
	for i := s.BitLen() - 1; i >= 0; i-- {
		e.cyclotomicSquare(z, z)
		if s.Bit(i) == 1 {
			e.mul(z, z, a)
		}
	}
	c.set(z)
}

func (e *fp12) frobeniusMap(c, a *fe12, power uint) {
	fp6 := e.fp6
	fp6.frobeniusMap(&c[0], &a[0], power)
	fp6.frobeniusMap(&c[1], &a[1], power)
	switch power {
	case 0:
		return
	case 6:
		fp6.neg(&c[1], &c[1])
	default:
		fp6.mulByBaseField(&c[1], &c[1], &frobeniusCoeffs12[power])
	}
}

func (e *fp12) frobeniusMapAssign(a *fe12, power uint) {
	fp6 := e.fp6
	fp6.frobeniusMapAssign(&a[0], power)
	fp6.frobeniusMapAssign(&a[1], power)
	switch power {
	case 0:
		return
	case 6:
		fp6.neg(&a[1], &a[1])
	default:
		fp6.mulByBaseField(&a[1], &a[1], &frobeniusCoeffs12[power])
	}
}
