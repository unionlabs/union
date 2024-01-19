// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

import (
	"math/bits"
)

// LeadingZeros returns the number of leading zero bits in the BitArray.
func (ba *BitArray) LeadingZeros() int {
	switch {
	case ba.IsZero():
		return 0
	case ba.b == nil:
		return ba.nBits
	}
	n := 0
	nb := ba.nBits >> 3
	for i := 0; i < nb; i++ {
		z := bits.LeadingZeros8(ba.b[i])
		n += z
		if z != 8 {
			return n
		}
	}

	fb := ba.nBits & 7
	if fb == 0 {
		return n
	}
	z := bits.LeadingZeros8(ba.b[nb])
	if fb < z {
		z = fb
	}

	return n + z
}

// TrailingZeros returns the number of trailing zero bits in the BitArray.
func (ba *BitArray) TrailingZeros() int {
	switch {
	case ba.IsZero():
		return 0
	case ba.b == nil:
		return ba.nBits
	}
	n := 0
	for i := len(ba.b) - 1; 0 <= i; i-- {
		z := bits.TrailingZeros8(ba.b[i])
		n += z
		if z != 8 {
			return n - ba.NumPadding()
		}
	}

	return n - ba.NumPadding()
}

func (ba *BitArray) hasTrailingZeros(n int) bool {
	if n == 0 {
		return true
	}
	n += ba.NumPadding()

	for i := len(ba.b) - 1; 0 <= i; i-- {
		z := bits.TrailingZeros8(ba.b[i])
		n -= z
		switch {
		case n <= 0:
			return true
		case z != 8:
			return false
		}
	}
	return false
}

// OnesCount returns the number of one bits, population count, in the BitArray.
func (ba *BitArray) OnesCount() int {
	if ba.IsZero() || ba.b == nil {
		return 0
	}
	n := 0
	for _, u64 := range asUint64Slice(ba.b) {
		n += bits.OnesCount64(u64)
	}

	return n
}

// And returns a new BitArray as a result of a bitwise AND with x. The ba and x
// must be the same length, otherwise And will panic. Use AndAt instead to apply
// a partial AND with a short bit array.
func (ba *BitArray) And(x BitArrayer) *BitArray {
	var bax *BitArray
	if x != nil {
		bax = x.BitArray()
	}
	baLen, xLen := ba.Len(), bax.Len()
	switch {
	case baLen != xLen:
		panicf("And: length is not the same: %d != %d.", baLen, xLen)
	case baLen == 0:
		return zeroBitArray
	case ba.b == nil:
		return ba
	case bax.b == nil:
		return bax
	}
	zv := uint64(0)
	buf := allocByteSlice(len(ba.b))
	buf64 := asUint64Slice(buf)
	x64 := asUint64Slice(bax.b)
	for i, u64 := range asUint64Slice(ba.b) {
		buf64[i] = u64 & x64[i]
		zv |= buf64[i]
	}
	if zv == 0 {
		return &BitArray{nBits: ba.nBits}
	}
	return &BitArray{b: buf, nBits: ba.nBits}
}

// Or returns a new BitArray as a result of a bitwise OR with x. The ba and x
// must be the same length, otherwise Or will panic. Use OrAt instead to apply a
// partial OR with a short bit array.
func (ba *BitArray) Or(x BitArrayer) *BitArray {
	var bax *BitArray
	if x != nil {
		bax = x.BitArray()
	}
	baLen, xLen := ba.Len(), bax.Len()
	switch {
	case baLen != xLen:
		panicf("Or: length is not the same: %d != %d.", baLen, xLen)
	case baLen == 0:
		return zeroBitArray
	case ba.b == nil:
		return bax
	case bax.b == nil:
		return ba
	}
	zv := uint64(0)
	buf := allocByteSlice(len(ba.b))
	buf64 := asUint64Slice(buf)
	x64 := asUint64Slice(bax.b)
	for i, u64 := range asUint64Slice(ba.b) {
		buf64[i] = u64 | x64[i]
		zv |= buf64[i]
	}
	if zv == 0 {
		return &BitArray{nBits: ba.nBits}
	}
	return &BitArray{b: buf, nBits: ba.nBits}
}

// Xor returns a new BitArray as a result of a bitwise XOR with x. The ba and x
// must be the same length, otherwise Xor will panic. Use XorAt instead to apply
// a partial XOR with a short bit array.
func (ba *BitArray) Xor(x BitArrayer) *BitArray {
	var bax *BitArray
	if x != nil {
		bax = x.BitArray()
	}
	baLen, xLen := ba.Len(), bax.Len()
	switch {
	case baLen != xLen:
		panicf("Xor: length is not the same: %d != %d.", baLen, xLen)
	case baLen == 0:
		return zeroBitArray
	case ba.b == nil:
		return bax
	case bax.b == nil:
		return ba
	}
	zv := uint64(0)
	buf := allocByteSlice(len(ba.b))
	buf64 := asUint64Slice(buf)
	x64 := asUint64Slice(bax.b)
	for i, u64 := range asUint64Slice(ba.b) {
		buf64[i] = u64 ^ x64[i]
		zv |= buf64[i]
	}
	if zv == 0 {
		return &BitArray{nBits: ba.nBits}
	}
	return &BitArray{b: buf, nBits: ba.nBits}
}

// Not returns a new BitArray that is the result of inverting all the bits.
func (ba *BitArray) Not() *BitArray {
	switch {
	case ba.IsZero():
		return zeroBitArray
	case ba.b == nil:
		return NewOneFilled(ba.nBits)
	}
	// TODO: use asUint64Slice()
	zv := byte(0)
	buf := allocByteSlice(len(ba.b))
	for i := 0; i < len(buf)-1; i++ {
		buf[i] = ^ba.b[i]
		zv |= buf[i]
	}
	buf[len(buf)-1] = ^ba.b[len(ba.b)-1] & (byte(0xff) << ba.NumPadding())
	zv |= buf[len(buf)-1]
	if zv == 0 {
		return &BitArray{nBits: ba.nBits}
	}
	return &BitArray{b: buf, nBits: ba.nBits}
}

// AndAt returns a new BitArray resulting from applying a bitwise AND operation
// with x at the offset off. AND is applied only to the range from off to
// off+x.Len(), and other bits are preserved.
func (ba *BitArray) AndAt(off int, x BitArrayer) *BitArray {
	var bax *BitArray
	if x != nil {
		bax = x.BitArray()
	}
	baLen, xLen := ba.Len(), bax.Len()
	switch {
	case off < 0:
		panicf("AndAt: negative off %d.", off)
	case baLen < off+xLen:
		panicf("AndAt: out of range: off=%d + x.len=%d > len=%d.", off, xLen, baLen)
	case baLen == 0:
		return zeroBitArray
	case ba.b == nil:
		return ba
	case bax.b == nil:
		buf := allocByteSlice(len(ba.b))
		copy(buf, ba.b)
		clearBits(buf, off, xLen)
		return &BitArray{b: buf, nBits: baLen}
	}
	buf := allocByteSlice(len(ba.b))
	copy(buf, ba.b)
	andBits(buf, bax.b, off, 0, xLen)
	return &BitArray{b: buf, nBits: baLen}
}

// OrAt returns a new BitArray resulting from applying a bitwise OR operation
// with x at the offset off. OR is applied only to the range from off to
// off+x.Len(), and other bits are preserved.
func (ba *BitArray) OrAt(off int, x BitArrayer) *BitArray {
	var bax *BitArray
	if x != nil {
		bax = x.BitArray()
	}
	baLen, xLen := ba.Len(), bax.Len()
	switch {
	case off < 0:
		panicf("OrAt: negative off %d.", off)
	case baLen < off+xLen:
		panicf("OrAt: out of range: off=%d + x.len=%d > len=%d.", off, xLen, baLen)
	case baLen == 0:
		return zeroBitArray
	case bax.b == nil:
		return ba
	case ba.b == nil:
		buf := allocByteSlice((baLen + 7) >> 3)
		_ = copyBits(buf, bax.b, off, 0, xLen)
		return &BitArray{b: buf, nBits: baLen}
	}
	buf := allocByteSlice(len(ba.b))
	copy(buf, ba.b)
	orBits(buf, bax.b, off, 0, xLen)
	return &BitArray{b: buf, nBits: baLen}
}

// XorAt returns a new BitArray resulting from applying a bitwise XOR operation
// with x at the offset off. XOR is applied only to the range from off to
// off+x.Len(), and other bits are preserved.
func (ba *BitArray) XorAt(off int, x BitArrayer) *BitArray {
	var bax *BitArray
	if x != nil {
		bax = x.BitArray()
	}
	baLen, xLen := ba.Len(), bax.Len()
	switch {
	case off < 0:
		panicf("XorAt: negative off %d.", off)
	case baLen < off+xLen:
		panicf("XorAt: out of range: off=%d + x.len=%d > len=%d", off, xLen, baLen)
	case baLen == 0:
		return zeroBitArray
	case bax.b == nil:
		return ba
	case ba.b == nil:
		buf := allocByteSlice((baLen + 7) >> 3)
		_ = copyBits(buf, bax.b, off, 0, xLen)
		return &BitArray{b: buf, nBits: baLen}
	}
	buf := allocByteSlice(len(ba.b))
	copy(buf, ba.b)
	xorBits(buf, bax.b, off, 0, xLen)
	return &BitArray{b: buf, nBits: baLen}
}
