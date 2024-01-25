// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

// BitAt returns a single bit at the specified offset as 0 or 1. It panics if
// the off is out of range.
func (buf *Buffer) BitAt(off int) byte {
	switch {
	case off < 0:
		panicf("BitAt: negative off %d.", off)
	case buf.nBits <= off:
		panicf("BitAt: out of range: off=%d >= len=%d.", off, buf.nBits)
	}

	off += buf.off
	return buf.b[off>>3] >> (7 - off&7) & 1
}

// PutBitAt writes a single bit at the position specified by off in the buffer.
// The bit should be 0 or 1, otherwise its LSB is silently used.
func (buf *Buffer) PutBitAt(off int, bit byte) {
	switch {
	case off < 0:
		panicf("PutBitAt: negative off %d.", off)
	case buf.nBits <= off:
		panicf("PutBitAt: out of range: off=%d >= len=%d.", off, buf.nBits)
	}

	off += buf.off
	buf.b[off>>3] = buf.b[off>>3] & ^(byte(0x80)>>(off&7)) | ((bit & 1) << (7 - off&7))
}

// BitArrayAt returns bits within the specified range as a BitArray.
func (buf *Buffer) BitArrayAt(off, nBits int) *BitArray {
	switch {
	case off < 0:
		panicf("BitArrayAt: negative off %d.", off)
	case nBits < 0:
		panicf("BitArrayAt: negative nBits %d.", nBits)
	case buf.nBits < off+nBits:
		panicf("BitArrayAt: out of range: off=%d + nBits=%d > len=%d.", off, nBits, buf.nBits)
	case nBits == 0:
		return zeroBitArray
	}

	return NewFromBytes(buf.b, buf.off+off, nBits)
}

// PutBitArrayAt writes bits from a BitArray onto the specified offset off.
func (buf *Buffer) PutBitArrayAt(off int, ba BitArrayer) {
	switch {
	case off < 0:
		panicf("PutBitArrayAt: negative off %d.", off)
	case ba == nil:
		return
	}
	bab := ba.BitArray()
	switch {
	case buf.nBits < off+bab.nBits:
		panicf("PutBitArrayAt: out of range: off=%d + ba.len=%d > len=%d.", off, bab.nBits, buf.nBits)
	case bab.IsZero():
		return
	case bab.b == nil:
		clearBits(buf.b, buf.off+off, bab.nBits)
		return
	}
	_ = copyBits(buf.b, bab.b, buf.off+off, 0, bab.nBits)
}

// ByteAt reads 8 bits starting at the offset off and returns them as a single
// byte. Note that off is in bits, not bytes. If the off is not a multiple of 8,
// 8 bits across a byte boundary are returned.
func (buf *Buffer) ByteAt(off int) byte {
	switch {
	case off < 0:
		panicf("ByteAt: negative off %d.", off)
	case buf.nBits < off+8:
		panicf("ByteAt: out of range: off=%d + 8 > len=%d.", off, buf.nBits)
	}
	off += buf.off
	i, f := off>>3, off&7
	if f == 0 {
		return buf.b[i]
	}
	return buf.b[i]<<f | buf.b[i+1]>>(8-f)
}

// PutByteAt writes 8 bits of b to the position specified by off in the buffer.
// Note that off is in bits, not bytes. If the off is not a multiple of 8, it
// writes 8 bits across a byte boundary.
func (buf *Buffer) PutByteAt(off int, b byte) {
	switch {
	case off < 0:
		panicf("PutByteAt: negative off %d.", off)
	case buf.nBits < off+8:
		panicf("PutByteAt: out of range: off=%d + 8 > len=%d.", off, buf.nBits)
	}
	copyBits(buf.b, []byte{b}, buf.off+off, 0, 8)
}

// RawBytes returns all the bits of the buffer as a byte slice. The caller must
// not change the contents of the returned byte slice. The slice returned may or
// may not reference to the internal buffer itself of buf, depending on whether
// bit-shifting is needed of not. Also, if buf.Len() is not a multiple of 8, the
// bits after the last bit in the slice returned are undefined. The main purpose
// of RawBytes is to efficiently pass bit data to other byte-oriented APIs. In
// general, it is recommended to use the safer Bytes() instead.
func (buf *Buffer) RawBytes() []byte {
	if buf.off&7 == 0 {
		return buf.b[buf.off>>3 : (buf.off+buf.nBits+7)>>3]
	}
	return buf.Bytes()
}

// Bytes returns all the bits of the buffer as a byte slice. If buf.Len() is not
// a multiple of 8, it will be padded with 0.
func (buf *Buffer) Bytes() []byte {
	b := make([]byte, (buf.nBits+7)>>3)
	copyBits(b, buf.b, 0, buf.off, buf.nBits)
	return b
}

// BytesAt reads 8 * nBytes bits starting at the offset off and returns them as
// a byte slice. Note that off is in bits, not bytes. If the off is not a
// multiple of 8, it returns a properly shifted byte slice.
func (buf *Buffer) BytesAt(off, nBytes int) []byte {
	nBits := nBytes << 3
	switch {
	case off < 0:
		panicf("ByteAt: negative off %d.", off)
	case nBytes < 0:
		panicf("ByteAt: negative nBytes %d.", nBytes)
	case buf.nBits < off+nBits:
		panicf("BytesAt: out of range: off=%d + 8 * nBytes=%d > len=%d.", off, nBytes, buf.nBits)
	case nBytes == 0:
		return []byte{}
	}
	ret := make([]byte, nBytes)
	copyBits(ret, buf.b, 0, buf.off+off, nBits)

	return ret
}

// PutBytesAt writes 8 * len(b) bits of b to the position specified by off in
// the buffer. Note that off is in bits, not bytes. If the off is not a multiple
// of 8, it writes bytes across byte boundaries of the buffer.
func (buf *Buffer) PutBytesAt(off int, b []byte) {
	nBits := len(b) << 3
	switch {
	case off < 0:
		panicf("PutByteAt: negative off %d.", off)
	case buf.nBits < off+nBits:
		panicf("PutByteAt: out of range: off=%d + 8 * b.len=%d > len=%d.", off, len(b), buf.nBits)
	case len(b) == 0:
		return
	}
	copyBits(buf.b, b, buf.off+off, 0, nBits)
}
