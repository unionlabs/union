// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

import (
	"bytes"
)

// Compare returns an integer comparing two bit arrays lexicographically. The
// result will be 0 if x == y, -1 if x < y, and +1 if y < x. A nil argument is
// equivalent to an empty bit array.
func Compare(x, y BitArrayer) int {
	var bax, bay *BitArray
	if x != nil {
		bax = x.BitArray()
	}
	if y != nil {
		bay = y.BitArray()
	}
	xLen, yLen, xlty := bax.Len(), bay.Len(), -1
	if yLen < xLen {
		bax, bay, xLen, yLen, xlty = bay, bax, yLen, xLen, +1
	}

	switch {
	case yLen == 0:
		return 0
	case xLen == 0:
		return xlty
	case bax.b == nil:
		if xLen == yLen && (bay.b == nil || allBytesZero(bay.b)) {
			return 0
		}
		return xlty
	case bay.b == nil:
		if allBytesZero(bax.b) {
			if xLen == yLen {
				return 0
			}
			return xlty
		}
		return -xlty
	}

	ce := bax.nBits >> 3 // end index of common bytes
	cc := bytes.Compare(bax.b[:ce], bay.b[:ce])
	switch {
	case 0 < cc:
		return -xlty
	case cc < 0:
		return xlty
	}

	if bax.nBits&7 == 0 { // no more x bits
		if xLen == yLen {
			return 0
		}
		return xlty
	}

	// compare the fractional bits in the last byte
	cs := 8 - bax.nBits&7
	xl, yl := bax.b[ce]>>cs, bay.b[ce]>>cs
	switch {
	case yl < xl:
		return -xlty
	case xl < yl, xLen != yLen:
		return xlty
	}

	return 0
}

// Equal returns whether the bit array is the same as specified one.
// nil and zero length bit array are considered to be equal.
func (ba *BitArray) Equal(x BitArrayer) bool {
	var bax *BitArray
	if x != nil {
		bax = x.BitArray()
	}
	switch {
	case ba.IsZero():
		return bax.IsZero()
	case bax.IsZero():
		return false
	case ba.nBits != bax.nBits:
		return false
	case ba.b == nil:
		return bax.b == nil || allBytesZero(bax.b)
	case bax.b == nil:
		return allBytesZero(ba.b)
	}

	return bytes.Equal(ba.b, bax.b)
}
