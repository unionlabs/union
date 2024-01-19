// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

// ToggleBitAt flips a single bit at the position specified by off in the
// buffer.
func (buf *Buffer) ToggleBitAt(off int) {
	switch {
	case off < 0:
		panicf("ToggleBitAt: negative off %d.", off)
	case buf.nBits <= off:
		panicf("ToggleBitAt: out of range: off=%d >= len=%d.", off, buf.nBits)
	}
	off += buf.off
	buf.b[off>>3] ^= byte(0x80) >> (off & 7)
}

// ToggleBitsAt inverts the nBits bits starting at off.
func (buf *Buffer) ToggleBitsAt(off, nBits int) {
	switch {
	case off < 0:
		panicf("ToggleBitsAt: negative off %d.", off)
	case nBits < 0:
		panicf("ToggleBitsAt: negative nBits %d.", nBits)
	case buf.nBits < off+nBits:
		panicf("ToggleBitsAt: out of range: off=%d + nBits=%d > len=%d.", off, nBits, buf.nBits)
	case nBits == 0:
		// no-op
	default:
		toggleBits(buf.b, buf.off+off, nBits)
	}
}

// AndAt applies a bitwise AND operation with x at the offset off. AND is
// applied only to the range from off to off+x.Len(), and other bits are
// preserved.
func (buf *Buffer) AndAt(off int, x BitArrayer) {
	var bax *BitArray
	if x != nil {
		bax = x.BitArray()
	}
	switch {
	case off < 0:
		panicf("AndAt: negative off %d.", off)
	case buf.nBits < off+bax.Len():
		panicf("AndAt: out of range: off=%d + x.len=%d > len=%d.", off, bax.Len(), buf.nBits)
	case bax.IsZero():
		// no-op
	case bax.b == nil:
		clearBits(buf.b, buf.off+off, bax.nBits)
	default:
		andBits(buf.b, bax.b, buf.off+off, 0, bax.nBits)
	}
}

// OrAt applies a bitwise OR operation with x at the offset off. OR is applied
// only to the range from off to off+x.Len(), and other bits are preserved.
func (buf *Buffer) OrAt(off int, x BitArrayer) {
	var bax *BitArray
	if x != nil {
		bax = x.BitArray()
	}
	switch {
	case off < 0:
		panicf("OrAt: negative off %d.", off)
	case buf.nBits < off+bax.Len():
		panicf("OrAt: out of range: off=%d + x.len=%d > len=%d.", off, bax.Len(), buf.nBits)
	case bax.IsZero(), bax.b == nil:
		// no-op
	default:
		orBits(buf.b, bax.b, buf.off+off, 0, bax.nBits)
	}
}

// XorAt applies a bitwise XOR operation with x at the offset off. XOR is
// applied only to the range from off to off+x.Len(), and other bits are
// preserved.
func (buf *Buffer) XorAt(off int, x BitArrayer) {
	var bax *BitArray
	if x != nil {
		bax = x.BitArray()
	}
	switch {
	case off < 0:
		panicf("XorAt: negative off %d.", off)
	case buf.nBits < off+bax.Len():
		panicf("XorAt: out of range: off=%d + x.len=%d > len=%d.", off, bax.Len(), buf.nBits)
	case bax.IsZero(), bax.b == nil:
		// no-op
	default:
		xorBits(buf.b, bax.b, buf.off+off, 0, bax.nBits)
	}
}

// LeadingZeros returns the number of leading zero bits in the Buffer.
func (buf *Buffer) LeadingZeros() int { return buf.BitArray().LeadingZeros() }

// TrailingZeros returns the number of trailing zero bits in the Buffer.
func (buf *Buffer) TrailingZeros() int { return buf.BitArray().TrailingZeros() }

// OnesCount returns the number of one bits, population count, in the Buffer.
func (buf *Buffer) OnesCount() int { return buf.BitArray().OnesCount() }
