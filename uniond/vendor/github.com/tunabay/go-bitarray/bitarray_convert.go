// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

import (
	"encoding/binary"
	"math/big"
)

// NewFromInt creates a BitArray from big.Int. The returned bit array represents
// the absolute value of v in big-endian byte order. For big.Int(0), it returns
// a "0" instead of an empty bit array.
func NewFromInt(v *big.Int) *BitArray {
	nBits := v.BitLen()
	if nBits == 0 {
		return &BitArray{nBits: 1}
	}

	return NewFromBytes(v.Bytes(), (8-nBits&7)&7, nBits)
}

// ToInt parses the bit array as a big-endian representation, and convert it to
// an unsigned integer value. The leading 0s have no effect on the result.
func (ba *BitArray) ToInt() *big.Int {
	v := big.NewInt(0)
	if ba.IsZero() || ba.b == nil {
		return v
	}
	v.SetBytes(ba.b)

	return v.Rsh(v, uint(ba.NumPadding()))
}

// ToUint64 is the same as ToInt except that it returns uint64 instead of
// *big.Int. If the bit array length exceeds 64 bits, only the last 64 bits
// will be used.
func (ba *BitArray) ToUint64() uint64 {
	if ba.IsZero() || ba.b == nil {
		return 0
	}
	switch {
	case ba.nBits < 64:
		buf := make([]byte, 8)
		copy(buf[8-len(ba.b):], ba.b)
		return binary.BigEndian.Uint64(buf) >> ba.NumPadding()
	case 64 < ba.nBits:
		ba = ba.Slice(ba.nBits-64, ba.nBits)
	}

	return binary.BigEndian.Uint64(ba.b)
}
