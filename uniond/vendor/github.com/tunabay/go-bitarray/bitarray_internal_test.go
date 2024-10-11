// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

import (
	"fmt"
)

// D returns the string representing of its internal state.
func (ba *BitArray) D() string {
	if ba == nil {
		return "<nil>"
	}
	if ba.b == nil {
		return fmt.Sprintf("BA{nbit=%d, b=<nil>}", ba.nBits)
	}
	return fmt.Sprintf("BA{nbit=%d, b=%08b}", ba.nBits, ba.b)
}

// V validate the internal data representation. It panics on failure.
func (ba *BitArray) V() {
	switch {
	case ba == nil:
		return

	case ba.nBits < 0:
		panicf("V: negative nBits %d.", ba.nBits)

	case ba.b != nil && len(ba.b) == 0:
		panicf("V: ba.b is an empty slice, must be nil: %08b", ba.b)

	case ba.b == nil:
		return

	case len(ba.b) != (ba.nBits+7)>>3:
		panicf("V: wrong len: len=%d, nBits=%d: %08b", len(ba.b), ba.nBits, ba.b)

	case cap(ba.b)&7 != 0:
		panicf("V: wrong cap: cap=%d, len=%d, nBits=%d.", cap(ba.b), len(ba.b), ba.nBits)
	}
	if fb := ba.nBits & 7; fb != 0 {
		mask := byte(0xff) >> fb
		if lb := ba.b[len(ba.b)-1] & mask; lb != 0 {
			panicf("V: non-zero padding bits: nfrac=%d, lastbyte=%08b.", fb, lb)
		}
	}
}

// ZExpand expands the zero-filled BitArray with nil pointer to a normal
// data representation.
func (ba *BitArray) ZExpand() *BitArray {
	if ba.IsZero() || ba.b != nil {
		return ba
	}
	return &BitArray{
		b:     allocByteSlice((ba.nBits + 7) >> 3),
		nBits: ba.nBits,
	}
}

// ZOptimize converts to optimized data representation if all bits are 0.
func (ba *BitArray) ZOptimize() *BitArray {
	if ba.IsZero() || ba.b == nil || !allBytesZero(ba.b) {
		return ba
	}
	return &BitArray{nBits: ba.nBits}
}
