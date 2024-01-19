// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

import (
	"math/bits"
)

// Reverse returns the bit array with its bits in reversed order.
func (ba *BitArray) Reverse() *BitArray {
	switch {
	case ba.IsZero():
		return zeroBitArray
	case ba.Len() == 1, ba.b == nil:
		return ba
	}
	buf := make([]byte, len(ba.b))
	for i, o := range ba.b {
		buf[len(ba.b)-1-i] = bits.Reverse8(o)
	}

	return NewFromBytes(buf, ba.NumPadding(), ba.nBits)
}

// ShiftLeft returns the bit array of shifted left by k bits.
// To shift to right, call ShiftLeft(-k).
func (ba *BitArray) ShiftLeft(k int) *BitArray {
	switch {
	case ba.IsZero():
		return zeroBitArray
	case ba.b == nil:
		return ba
	case ba.nBits <= k, ba.nBits <= -k:
		return &BitArray{nBits: ba.nBits}
	case 0 < k:
		return ba.shiftLeft(k)
	case k < 0:
		return ba.shiftRight(-k)
	}

	return ba
}

func (ba *BitArray) shiftLeft(k int) *BitArray {
	if k&7 == 0 {
		buf := allocByteSlice(len(ba.b))
		copy(buf, ba.b[k>>3:])
		return &BitArray{b: buf, nBits: ba.nBits}
	}

	return ba.Slice(k, ba.nBits).append1(&BitArray{nBits: k})
}

func (ba *BitArray) shiftRight(k int) *BitArray {
	if k&7 == 0 {
		buf := allocByteSlice(len(ba.b))
		copy(buf[k>>3:], ba.b)
		if npad := ba.NumPadding(); npad != 0 {
			mask := byte(0xff) << npad
			buf[len(buf)-1] &= mask
		}
		return &BitArray{b: buf, nBits: ba.nBits}
	}

	return (&BitArray{nBits: k}).append1(ba.Slice(0, ba.nBits-k))
}

// RotateLeft returns the bit array of rotated left by k bits.
// To rotate to right, call RotateLeft(-k).
func (ba *BitArray) RotateLeft(k int) *BitArray {
	switch {
	case ba.IsZero():
		return zeroBitArray
	case ba.b == nil:
		return ba
	case 0 < k:
		return ba.rotateLeft(k)
	case k < 0:
		return ba.rotateRight(-k)
	}

	return ba
}

func (ba *BitArray) rotateLeft(k int) *BitArray {
	k %= ba.nBits
	switch {
	case k == 0:
		return ba
	case k&7 == 0 && ba.nBits&7 == 0:
		buf := allocByteSlice(len(ba.b))
		nbs := k >> 3
		copy(buf, ba.b[nbs:])
		copy(buf[len(buf)-nbs:], ba.b)
		return &BitArray{b: buf, nBits: ba.nBits}
	}

	return ba.Slice(k, ba.nBits).append1(ba.Slice(0, k))
}

func (ba *BitArray) rotateRight(k int) *BitArray {
	k %= ba.nBits
	switch {
	case k == 0:
		return ba
	case k&7 == 0 && ba.nBits&7 == 0:
		buf := allocByteSlice(len(ba.b))
		nbs := k >> 3
		copy(buf[nbs:], ba.b)
		copy(buf, ba.b[len(ba.b)-nbs:])
		return &BitArray{b: buf, nBits: ba.nBits}
	}

	return ba.Slice(ba.nBits-k, ba.nBits).append1(ba.Slice(0, ba.nBits-k))
}
