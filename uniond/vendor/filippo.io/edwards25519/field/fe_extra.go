// Copyright (c) 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package field

import "errors"

// This file contains additional functionality that is not included in the
// upstream crypto/ed25519/edwards25519/field package.

// SetWideBytes sets v to x, where x is a 64-byte little-endian encoding, which
// is reduced modulo the field order. If x is not of the right length,
// SetWideBytes returns nil and an error, and the receiver is unchanged.
//
// SetWideBytes is not necessary to select a uniformly distributed value, and is
// only provided for compatibility: SetBytes can be used instead as the chance
// of bias is less than 2⁻²⁵⁰.
func (v *Element) SetWideBytes(x []byte) (*Element, error) {
	if len(x) != 64 {
		return nil, errors.New("edwards25519: invalid SetWideBytes input size")
	}

	// Split the 64 bytes into two elements, and extract the most significant
	// bit of each, which is ignored by SetBytes.
	lo, _ := new(Element).SetBytes(x[:32])
	loMSB := uint64(x[31] >> 7)
	hi, _ := new(Element).SetBytes(x[32:])
	hiMSB := uint64(x[63] >> 7)

	// The output we want is
	//
	//   v = lo + loMSB * 2²⁵⁵ + hi * 2²⁵⁶ + hiMSB * 2⁵¹¹
	//
	// which applying the reduction identity comes out to
	//
	//   v = lo + loMSB * 19 + hi * 2 * 19 + hiMSB * 2 * 19²
	//
	// l0 will be the sum of a 52 bits value (lo.l0), plus a 5 bits value
	// (loMSB * 19), a 6 bits value (hi.l0 * 2 * 19), and a 10 bits value
	// (hiMSB * 2 * 19²), so it fits in a uint64.

	v.l0 = lo.l0 + loMSB*19 + hi.l0*2*19 + hiMSB*2*19*19
	v.l1 = lo.l1 + hi.l1*2*19
	v.l2 = lo.l2 + hi.l2*2*19
	v.l3 = lo.l3 + hi.l3*2*19
	v.l4 = lo.l4 + hi.l4*2*19

	return v.carryPropagate(), nil
}
