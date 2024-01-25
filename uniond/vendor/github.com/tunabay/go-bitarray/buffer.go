// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

import (
	"fmt"
)

// Buffer is a bit array buffer whose contents can be updated by partial reading
// and writing with an offset. It is not safe for concurrent use by multiple
// goroutines. The zero value for Buffer represents a zero length buffer that
// can be resized and used.
type Buffer struct {
	b     []byte // nil only for zero length
	nBits int
	off   int
}

// NewBuffer creates a Buffer with the specified bit length.
func NewBuffer(nBits int) *Buffer {
	switch {
	case nBits < 0:
		panicf("NewBuffer: negative nBits %d.", nBits)
	case nBits == 0:
		return &Buffer{}
	}

	return &Buffer{
		b:     allocByteSlice((nBits + 7) >> 3),
		nBits: nBits,
	}
}

// NewBufferFromBitArray creates a new Buffer with the same bit length and
// initial content as the specified BitArray.
func NewBufferFromBitArray(ba BitArrayer) *Buffer {
	if ba == nil {
		return &Buffer{}
	}
	bab := ba.BitArray()
	buf := NewBuffer(bab.Len())
	if 0 < buf.nBits {
		copy(buf.b, bab.b)
	}
	return buf
}

// NewBufferFromByteSlice creates a new Buffer that references an existing byte
// slice b. The created Buffer references b without copying it, therefore,
// changes to the buffer affect b and vice versa.  The length of the buffer
// created will be len(b) * 8. NewBufferFromByteSlice is useful when reading or
// writing a subpart of a byte slice as a bit array without copying or
// bit-shifting.
func NewBufferFromByteSlice(b []byte) *Buffer {
	return NewBufferFromByteSlicePartial(b, 0, len(b)<<3)
}

// NewBufferFromByteSlicePartial is identical to NewBufferFromByteSlice except
// that it creates a buffer with the first bit specified by off, and the length
// specified by nBits.
func NewBufferFromByteSlicePartial(b []byte, off, nBits int) *Buffer {
	switch {
	case off < 0:
		panicf("NewBufferFromByteSlice: negative off %d.", nBits)
	case nBits < 0:
		panicf("NewBufferFromByteSlice: negative nBits %d.", nBits)
	case len(b)<<3 < off+nBits:
		panicf("NewBufferFromByteSlice: out of range: off=%d, nBits=%d > len=%d.", off, nBits, len(b))
	case nBits == 0:
		return &Buffer{}
	}
	return &Buffer{b: b[off>>3:], nBits: nBits, off: off & 7}
}

// IsZero returns whether the Buffer is zero length.
func (buf *Buffer) IsZero() bool {
	return buf.Len() == 0
}

// Len returns the number of bits contained in the buffer.
func (buf *Buffer) Len() int {
	if buf == nil {
		return 0
	}
	return buf.nBits
}

// Clone clones the Buffer with its content.
func (buf *Buffer) Clone() *Buffer {
	if buf.Len() == 0 {
		return &Buffer{}
	}
	b := make([]byte, len(buf.b))
	copy(b, buf.b)

	return &Buffer{b: b, nBits: buf.nBits, off: buf.off}
}

// BitArray creates an imuurable BitArray from the current content.
func (buf *Buffer) BitArray() *BitArray {
	return NewFromBytes(buf.b, buf.off, buf.nBits)
}

// String returns the string representation of the current content.
func (buf Buffer) String() string {
	sb := make([]byte, buf.nBits)
	for i := 0; i < buf.nBits; i++ {
		sb[i] = '0' + buf.b[(buf.off+i)>>3]>>(7-(buf.off+i)&7)&1
	}
	return string(sb)
}

// Resize resizes the Buffer to the size specified by nBits. When expanding, all
// bits in the new range to be extended are initialized with 0. When shrinking,
// the extra bits are truncated. In either case, the align specifies whether to
// fix the MSBs or the LSBs.
//
// Resize always reallocates internal memory. That is, the buffers created by
// Slice method or NewBufferFromByteSlice break their relationship with the
// parent buffer or slice by calling this Resize, even if nBits is equivalent to
// or less than its current size.
func (buf *Buffer) Resize(nBits int, align Alignment) {
	switch {
	case nBits < 0:
		panicf("Resize: negative nBits %d.", nBits)
	case nBits == 0:
		buf.b = nil
		buf.nBits = 0
		buf.off = 0
		return
	}

	b := allocByteSlice((nBits + 7) >> 3)
	if buf.nBits == 0 {
		buf.b = b
		buf.nBits = nBits
		buf.off = 0
		return
	}
	if align == AlignLeft {
		if nBits < buf.nBits { // shrink
			copyBits(b, buf.b, 0, buf.off, nBits)
		} else { // extend
			copyBits(b, buf.b, 0, buf.off, buf.nBits)
		}
	} else {
		if nBits < buf.nBits { // shrink
			copyBits(b, buf.b, 0, buf.off+buf.nBits-nBits, nBits)
		} else { // extend
			copyBits(b, buf.b, nBits-buf.nBits, buf.off, buf.nBits)
		}
	}

	buf.b = b
	buf.nBits = nBits
	buf.off = 0
}

// FillBits sets all the bits in the buffer to the value bit, 0 or 1.
func (buf *Buffer) FillBits(bit byte) {
	buf.FillBitsAt(0, buf.nBits, bit)
}

// FillBitsAt sets the nBits bits starting at off to the value bit.
func (buf *Buffer) FillBitsAt(off, nBits int, bit byte) {
	switch {
	case off < 0:
		panicf("FillBitsAt: negative off %d.", off)
	case nBits < 0:
		panicf("FillBitsAt: negative nBits %d.", nBits)
	case buf.nBits < off+nBits:
		panicf("FillBitsAt: out of range: off=%d + nBits=%d > len=%d.", off, nBits, buf.nBits)
	case bit&1 == 0:
		clearBits(buf.b, buf.off+off, nBits)
	default:
		setBits(buf.b, buf.off+off, nBits)
	}
}

// Format implements the fmt.Formatter interface to format Buffer value using
// the standard fmt.Printf family functions.
func (buf Buffer) Format(s fmt.State, verb rune) { buf.BitArray().Format(s, verb) }
