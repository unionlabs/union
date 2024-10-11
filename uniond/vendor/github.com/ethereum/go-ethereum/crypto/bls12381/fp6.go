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

type fp6Temp struct {
	t [6]*fe2
}

type fp6 struct {
	fp2 *fp2
	fp6Temp
}

func newFp6Temp() fp6Temp {
	t := [6]*fe2{}
	for i := 0; i < len(t); i++ {
		t[i] = &fe2{}
	}
	return fp6Temp{t}
}

func newFp6(f *fp2) *fp6 {
	t := newFp6Temp()
	if f == nil {
		return &fp6{newFp2(), t}
	}
	return &fp6{f, t}
}

func (e *fp6) fromBytes(b []byte) (*fe6, error) {
	if len(b) < 288 {
		return nil, errors.New("input string should be larger than 288 bytes")
	}
	fp2 := e.fp2
	u2, err := fp2.fromBytes(b[:96])
	if err != nil {
		return nil, err
	}
	u1, err := fp2.fromBytes(b[96:192])
	if err != nil {
		return nil, err
	}
	u0, err := fp2.fromBytes(b[192:])
	if err != nil {
		return nil, err
	}
	return &fe6{*u0, *u1, *u2}, nil
}

func (e *fp6) toBytes(a *fe6) []byte {
	fp2 := e.fp2
	out := make([]byte, 288)
	copy(out[:96], fp2.toBytes(&a[2]))
	copy(out[96:192], fp2.toBytes(&a[1]))
	copy(out[192:], fp2.toBytes(&a[0]))
	return out
}

func (e *fp6) new() *fe6 {
	return new(fe6)
}

func (e *fp6) zero() *fe6 {
	return new(fe6)
}

func (e *fp6) one() *fe6 {
	return new(fe6).one()
}

func (e *fp6) add(c, a, b *fe6) {
	fp2 := e.fp2
	fp2.add(&c[0], &a[0], &b[0])
	fp2.add(&c[1], &a[1], &b[1])
	fp2.add(&c[2], &a[2], &b[2])
}

func (e *fp6) addAssign(a, b *fe6) {
	fp2 := e.fp2
	fp2.addAssign(&a[0], &b[0])
	fp2.addAssign(&a[1], &b[1])
	fp2.addAssign(&a[2], &b[2])
}

func (e *fp6) double(c, a *fe6) {
	fp2 := e.fp2
	fp2.double(&c[0], &a[0])
	fp2.double(&c[1], &a[1])
	fp2.double(&c[2], &a[2])
}

func (e *fp6) doubleAssign(a *fe6) {
	fp2 := e.fp2
	fp2.doubleAssign(&a[0])
	fp2.doubleAssign(&a[1])
	fp2.doubleAssign(&a[2])
}

func (e *fp6) sub(c, a, b *fe6) {
	fp2 := e.fp2
	fp2.sub(&c[0], &a[0], &b[0])
	fp2.sub(&c[1], &a[1], &b[1])
	fp2.sub(&c[2], &a[2], &b[2])
}

func (e *fp6) subAssign(a, b *fe6) {
	fp2 := e.fp2
	fp2.subAssign(&a[0], &b[0])
	fp2.subAssign(&a[1], &b[1])
	fp2.subAssign(&a[2], &b[2])
}

func (e *fp6) neg(c, a *fe6) {
	fp2 := e.fp2
	fp2.neg(&c[0], &a[0])
	fp2.neg(&c[1], &a[1])
	fp2.neg(&c[2], &a[2])
}

func (e *fp6) mul(c, a, b *fe6) {
	fp2, t := e.fp2, e.t
	fp2.mul(t[0], &a[0], &b[0])
	fp2.mul(t[1], &a[1], &b[1])
	fp2.mul(t[2], &a[2], &b[2])
	fp2.add(t[3], &a[1], &a[2])
	fp2.add(t[4], &b[1], &b[2])
	fp2.mulAssign(t[3], t[4])
	fp2.add(t[4], t[1], t[2])
	fp2.subAssign(t[3], t[4])
	fp2.mulByNonResidue(t[3], t[3])
	fp2.add(t[5], t[0], t[3])
	fp2.add(t[3], &a[0], &a[1])
	fp2.add(t[4], &b[0], &b[1])
	fp2.mulAssign(t[3], t[4])
	fp2.add(t[4], t[0], t[1])
	fp2.subAssign(t[3], t[4])
	fp2.mulByNonResidue(t[4], t[2])
	fp2.add(&c[1], t[3], t[4])
	fp2.add(t[3], &a[0], &a[2])
	fp2.add(t[4], &b[0], &b[2])
	fp2.mulAssign(t[3], t[4])
	fp2.add(t[4], t[0], t[2])
	fp2.subAssign(t[3], t[4])
	fp2.add(&c[2], t[1], t[3])
	c[0].set(t[5])
}

func (e *fp6) mulAssign(a, b *fe6) {
	fp2, t := e.fp2, e.t
	fp2.mul(t[0], &a[0], &b[0])
	fp2.mul(t[1], &a[1], &b[1])
	fp2.mul(t[2], &a[2], &b[2])
	fp2.add(t[3], &a[1], &a[2])
	fp2.add(t[4], &b[1], &b[2])
	fp2.mulAssign(t[3], t[4])
	fp2.add(t[4], t[1], t[2])
	fp2.subAssign(t[3], t[4])
	fp2.mulByNonResidue(t[3], t[3])
	fp2.add(t[5], t[0], t[3])
	fp2.add(t[3], &a[0], &a[1])
	fp2.add(t[4], &b[0], &b[1])
	fp2.mulAssign(t[3], t[4])
	fp2.add(t[4], t[0], t[1])
	fp2.subAssign(t[3], t[4])
	fp2.mulByNonResidue(t[4], t[2])
	fp2.add(&a[1], t[3], t[4])
	fp2.add(t[3], &a[0], &a[2])
	fp2.add(t[4], &b[0], &b[2])
	fp2.mulAssign(t[3], t[4])
	fp2.add(t[4], t[0], t[2])
	fp2.subAssign(t[3], t[4])
	fp2.add(&a[2], t[1], t[3])
	a[0].set(t[5])
}

func (e *fp6) square(c, a *fe6) {
	fp2, t := e.fp2, e.t
	fp2.square(t[0], &a[0])
	fp2.mul(t[1], &a[0], &a[1])
	fp2.doubleAssign(t[1])
	fp2.sub(t[2], &a[0], &a[1])
	fp2.addAssign(t[2], &a[2])
	fp2.squareAssign(t[2])
	fp2.mul(t[3], &a[1], &a[2])
	fp2.doubleAssign(t[3])
	fp2.square(t[4], &a[2])
	fp2.mulByNonResidue(t[5], t[3])
	fp2.add(&c[0], t[0], t[5])
	fp2.mulByNonResidue(t[5], t[4])
	fp2.add(&c[1], t[1], t[5])
	fp2.addAssign(t[1], t[2])
	fp2.addAssign(t[1], t[3])
	fp2.addAssign(t[0], t[4])
	fp2.sub(&c[2], t[1], t[0])
}

func (e *fp6) mulBy01Assign(a *fe6, b0, b1 *fe2) {
	fp2, t := e.fp2, e.t
	fp2.mul(t[0], &a[0], b0)
	fp2.mul(t[1], &a[1], b1)
	fp2.add(t[5], &a[1], &a[2])
	fp2.mul(t[2], b1, t[5])
	fp2.subAssign(t[2], t[1])
	fp2.mulByNonResidue(t[2], t[2])
	fp2.add(t[5], &a[0], &a[2])
	fp2.mul(t[3], b0, t[5])
	fp2.subAssign(t[3], t[0])
	fp2.add(&a[2], t[3], t[1])
	fp2.add(t[4], b0, b1)
	fp2.add(t[5], &a[0], &a[1])
	fp2.mulAssign(t[4], t[5])
	fp2.subAssign(t[4], t[0])
	fp2.sub(&a[1], t[4], t[1])
	fp2.add(&a[0], t[2], t[0])
}

func (e *fp6) mulBy01(c, a *fe6, b0, b1 *fe2) {
	fp2, t := e.fp2, e.t
	fp2.mul(t[0], &a[0], b0)
	fp2.mul(t[1], &a[1], b1)
	fp2.add(t[2], &a[1], &a[2])
	fp2.mulAssign(t[2], b1)
	fp2.subAssign(t[2], t[1])
	fp2.mulByNonResidue(t[2], t[2])
	fp2.add(t[3], &a[0], &a[2])
	fp2.mulAssign(t[3], b0)
	fp2.subAssign(t[3], t[0])
	fp2.add(&c[2], t[3], t[1])
	fp2.add(t[4], b0, b1)
	fp2.add(t[3], &a[0], &a[1])
	fp2.mulAssign(t[4], t[3])
	fp2.subAssign(t[4], t[0])
	fp2.sub(&c[1], t[4], t[1])
	fp2.add(&c[0], t[2], t[0])
}

func (e *fp6) mulBy1(c, a *fe6, b1 *fe2) {
	fp2, t := e.fp2, e.t
	fp2.mul(t[0], &a[2], b1)
	fp2.mul(&c[2], &a[1], b1)
	fp2.mul(&c[1], &a[0], b1)
	fp2.mulByNonResidue(&c[0], t[0])
}

func (e *fp6) mulByNonResidue(c, a *fe6) {
	fp2, t := e.fp2, e.t
	t[0].set(&a[0])
	fp2.mulByNonResidue(&c[0], &a[2])
	c[2].set(&a[1])
	c[1].set(t[0])
}

func (e *fp6) mulByBaseField(c, a *fe6, b *fe2) {
	fp2 := e.fp2
	fp2.mul(&c[0], &a[0], b)
	fp2.mul(&c[1], &a[1], b)
	fp2.mul(&c[2], &a[2], b)
}

func (e *fp6) exp(c, a *fe6, s *big.Int) {
	z := e.one()
	for i := s.BitLen() - 1; i >= 0; i-- {
		e.square(z, z)
		if s.Bit(i) == 1 {
			e.mul(z, z, a)
		}
	}
	c.set(z)
}

func (e *fp6) inverse(c, a *fe6) {
	fp2, t := e.fp2, e.t
	fp2.square(t[0], &a[0])
	fp2.mul(t[1], &a[1], &a[2])
	fp2.mulByNonResidue(t[1], t[1])
	fp2.subAssign(t[0], t[1])
	fp2.square(t[1], &a[1])
	fp2.mul(t[2], &a[0], &a[2])
	fp2.subAssign(t[1], t[2])
	fp2.square(t[2], &a[2])
	fp2.mulByNonResidue(t[2], t[2])
	fp2.mul(t[3], &a[0], &a[1])
	fp2.subAssign(t[2], t[3])
	fp2.mul(t[3], &a[2], t[2])
	fp2.mul(t[4], &a[1], t[1])
	fp2.addAssign(t[3], t[4])
	fp2.mulByNonResidue(t[3], t[3])
	fp2.mul(t[4], &a[0], t[0])
	fp2.addAssign(t[3], t[4])
	fp2.inverse(t[3], t[3])
	fp2.mul(&c[0], t[0], t[3])
	fp2.mul(&c[1], t[2], t[3])
	fp2.mul(&c[2], t[1], t[3])
}

func (e *fp6) frobeniusMap(c, a *fe6, power uint) {
	fp2 := e.fp2
	fp2.frobeniusMap(&c[0], &a[0], power)
	fp2.frobeniusMap(&c[1], &a[1], power)
	fp2.frobeniusMap(&c[2], &a[2], power)
	switch power % 6 {
	case 0:
		return
	case 3:
		neg(&c[0][0], &a[1][1])
		c[1][1].set(&a[1][0])
		fp2.neg(&a[2], &a[2])
	default:
		fp2.mul(&c[1], &c[1], &frobeniusCoeffs61[power%6])
		fp2.mul(&c[2], &c[2], &frobeniusCoeffs62[power%6])
	}
}

func (e *fp6) frobeniusMapAssign(a *fe6, power uint) {
	fp2 := e.fp2
	fp2.frobeniusMapAssign(&a[0], power)
	fp2.frobeniusMapAssign(&a[1], power)
	fp2.frobeniusMapAssign(&a[2], power)
	t := e.t
	switch power % 6 {
	case 0:
		return
	case 3:
		neg(&t[0][0], &a[1][1])
		a[1][1].set(&a[1][0])
		a[1][0].set(&t[0][0])
		fp2.neg(&a[2], &a[2])
	default:
		fp2.mulAssign(&a[1], &frobeniusCoeffs61[power%6])
		fp2.mulAssign(&a[2], &frobeniusCoeffs62[power%6])
	}
}
