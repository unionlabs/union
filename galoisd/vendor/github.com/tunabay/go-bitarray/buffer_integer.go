// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

import (
	"encoding/binary"
)

// Uint8 returns up to 8 bits from the beginning of the buffer as a uint8 value.
// If buf.Len() is greater than 8, only the first 8 bits are used. If buf.Len()
// is less than 8, it is treated as an integer with that number of bits. For
// example, if buf.Len() == 5, it returns a 5-bit integer, 0 to 31(=0b11111), as
// a uint8 value.
func (buf *Buffer) Uint8() uint8 {
	if buf.nBits == 0 {
		return 0
	}
	n, off := 8, 0
	if buf.nBits < 8 {
		n, off = buf.nBits, 8-buf.nBits
	}
	b := make([]byte, 1)
	copyBits(b, buf.b, off, buf.off, n)

	return b[0]
}

// PutUint8 sets a uint8 value v within up to 8 bits from the beginning of the
// buffer. If buf.Len() is greater than 8, only the first 8 bits are updated. If
// buf.Len() is less than 8, only the LSBs of v are written.
func (buf *Buffer) PutUint8(v uint8) {
	if buf.nBits == 0 {
		return
	}
	n, off := 8, 0
	if buf.nBits < 8 {
		n, off = buf.nBits, 8-buf.nBits
	}
	b := []byte{v}
	copyBits(buf.b, b, buf.off, off, n)
}

// Uint16 returns up to 16 bits from the beginning of the buffer as a uint16
// value. If buf.Len() is greater than 16, only the first 16 bits are used. If
// buf.Len() is less than 16, it is treated as an integer with that number of
// bits. For example, if buf.Len() == 5, it returns a 5-bit integer, 0 to
// 31(=0b11111), as a uint16 value.
func (buf *Buffer) Uint16() uint16 {
	if buf.nBits == 0 {
		return 0
	}
	n, off := 16, 0
	if buf.nBits < 16 {
		n, off = buf.nBits, 16-buf.nBits
	}
	b := make([]byte, 2)
	copyBits(b, buf.b, off, buf.off, n)

	return binary.BigEndian.Uint16(b)
}

// PutUint16 sets a uint16 value v within up to 16 bits from the beginning of
// the buffer. If buf.Len() is greater than 16, only the first 16 bits are
// updated. If buf.Len() is less than 16, only the LSBs of v are written.
func (buf *Buffer) PutUint16(v uint16) {
	if buf.nBits == 0 {
		return
	}
	n, off := 16, 0
	if buf.nBits < 16 {
		n, off = buf.nBits, 16-buf.nBits
	}
	b := make([]byte, 2)
	binary.BigEndian.PutUint16(b, v)
	copyBits(buf.b, b, buf.off, off, n)
}

// Uint32 returns up to 32 bits from the beginning of the buffer as a uint32
// value. If buf.Len() is greater than 32, only the first 32 bits are used. If
// buf.Len() is less than 32, it is treated as an integer with that number of
// bits. For example, if buf.Len() == 5, it returns a 5-bit integer, 0 to
// 31(=0b11111), as a uint32 value.
func (buf *Buffer) Uint32() uint32 {
	if buf.nBits == 0 {
		return 0
	}
	n, off := 32, 0
	if buf.nBits < 32 {
		n, off = buf.nBits, 32-buf.nBits
	}
	b := make([]byte, 4)
	copyBits(b, buf.b, off, buf.off, n)

	return binary.BigEndian.Uint32(b)
}

// PutUint32 sets a uint32 value v within up to 32 bits from the beginning of
// the buffer. If buf.Len() is greater than 32, only the first 32 bits are
// updated. If buf.Len() is less than 32, only the LSBs of v are written.
func (buf *Buffer) PutUint32(v uint32) {
	if buf.nBits == 0 {
		return
	}
	n, off := 32, 0
	if buf.nBits < 32 {
		n, off = buf.nBits, 32-buf.nBits
	}
	b := make([]byte, 4)
	binary.BigEndian.PutUint32(b, v)
	copyBits(buf.b, b, buf.off, off, n)
}

// Uint64 returns up to 64 bits from the beginning of the buffer as a uint64
// value. If buf.Len() is greater than 64, only the first 64 bits are used. If
// buf.Len() is less than 64, it is treated as an integer with that number of
// bits. For example, if buf.Len() == 5, it returns a 5-bit integer, 0 to
// 31(=0b11111), as a uint64 value.
func (buf *Buffer) Uint64() uint64 {
	if buf.nBits == 0 {
		return 0
	}
	n, off := 64, 0
	if buf.nBits < 64 {
		n, off = buf.nBits, 64-buf.nBits
	}
	b := make([]byte, 8)
	copyBits(b, buf.b, off, buf.off, n)

	return binary.BigEndian.Uint64(b)
}

// PutUint64 sets a uint64 value v within up to 64 bits from the beginning of
// the buffer. If buf.Len() is greater than 64, only the first 64 bits are
// updated. If buf.Len() is less than 64, only the LSBs of v are written.
func (buf *Buffer) PutUint64(v uint64) {
	if buf.nBits == 0 {
		return
	}
	n, off := 64, 0
	if buf.nBits < 64 {
		n, off = buf.nBits, 64-buf.nBits
	}
	b := make([]byte, 8)
	binary.BigEndian.PutUint64(b, v)
	copyBits(buf.b, b, buf.off, off, n)
}
