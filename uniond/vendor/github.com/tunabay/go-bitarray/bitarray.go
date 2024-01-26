// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

import (
	"hash"
	"math/bits"
)

// BitArray represents an immutable bit array, or a sequence of bits, of
// arbitrary length. Unlike the builtin []byte, BitArray can properly hold and
// handle fractional bits less than 8 bits. The zero value for BitArray
// represents an empty bit array of zero length. Since it is immutable, it can
// be shared, copied and is safe for concurrent use by multiple goroutines.
type BitArray struct {
	b     []byte // nil for zero filled bit array including zero length
	nBits int    // number of bits contained
}

// zeroBitArray is a shared instance representing an empty bit array.
var zeroBitArray = &BitArray{}

// New creates and returns a new BitArray instance from the bits passed as
// parameters. Each parameter should be 0 or 1, but if any other value is
// passed, no error is reported and only the LSB of each is silently used. In
// most cases it is more convenient to use Parse, NewFromBytes or other
// functions instead of New.
func New(bits ...byte) *BitArray {
	if len(bits) == 0 {
		return zeroBitArray
	}
	var zfb byte
	buf := allocByteSlice((len(bits) + 7) >> 3)
	for i, bit := range bits {
		bit &= 1
		zfb |= bit
		buf[i>>3] |= bit << (7 - i&7)
	}
	if zfb == 0 {
		return &BitArray{nBits: len(bits)}
	}

	return &BitArray{b: buf, nBits: len(bits)}
}

// NewFromBytes reads bits from a byte slice b, creates a new BitArray instance
// and returns it. It skips off bits from the beginning of p and reads nBits
// bits from the next bit.
func NewFromBytes(p []byte, off, nBits int) *BitArray {
	switch {
	case nBits < 0:
		panicf("NewFromBytes: negative nBits %d.", nBits)
	case off < 0:
		panicf("NewFromBytes: negative off %d.", off)
	case len(p)<<3 < off+nBits:
		panicf("NewFromBytes: out of range off=%d + nBits=%d > p.len=%d.", off, nBits, len(p)<<3)
	case nBits == 0:
		return zeroBitArray
	}
	buf := allocByteSlice((nBits + 7) >> 3)
	if copyBits(buf, p, 0, off, nBits) {
		return &BitArray{nBits: nBits}
	}

	return &BitArray{b: buf, nBits: nBits}
}

// NewFromByteBits creates a new BitArray from a []byte in which each element
// represents 1 bit as 0 or 1. If an element is neighter 0 nor 1, only its LSB
// is silently used.
func NewFromByteBits(bits []byte) *BitArray {
	if len(bits) == 0 {
		return zeroBitArray
	}
	var zfb byte
	buf := allocByteSlice((len(bits) + 7) >> 3)
	for i, bit := range bits {
		bit &= 1
		zfb |= bit
		buf[i>>3] |= bit << (7 - i&7)
	}
	if zfb == 0 {
		return &BitArray{nBits: len(bits)}
	}

	return &BitArray{b: buf, nBits: len(bits)}
}

// NewZeroFilled creates a BitArray with all digits filled with 0. An all zero
// filled bit array does not allocate memory for 0 bits. If all bits in a bit
// array are guaranteed to be 0, using this function saves memory usage and
// optimizes some bitwise operations.
func NewZeroFilled(nBits int) *BitArray {
	switch {
	case nBits < 0:
		panicf("NewZeroFilled: negative nBits %d.", nBits)
	case nBits == 0:
		return zeroBitArray
	}

	return &BitArray{nBits: nBits}
}

// NewOneFilled creates a BitArray with all digits filled with 1.
func NewOneFilled(nBits int) *BitArray {
	switch {
	case nBits < 0:
		panicf("NewOneFilled: negative nBits %d.", nBits)
	case nBits == 0:
		return zeroBitArray
	}
	buf := allocByteSlice((nBits + 7) >> 3)
	fillFF(buf)
	if f := nBits & 7; f != 0 {
		buf[len(buf)-1] &= byte(0xff) << (8 - f)
	}

	return &BitArray{b: buf, nBits: nBits}
}

// NewByRunLength creates a BitArray with the argument that represents the
// number of consecutive 0 and 1 bits. The (2n+1)th arguments including the
// first specifies the length of 0s, and the (2n)th arguments including the
// second specifies the length of 1s. Passing 0 as the first argument allows to
// create a bit array starting with 1. It is suitable for making simple bit
// masks.
func NewByRunLength(lengths ...int) *BitArray {
	max1Len, sumLen := 0, 0
	for i := 0; i < len(lengths); i++ {
		sumLen += lengths[i]
		if i&1 != 0 && max1Len < lengths[i] {
			max1Len = lengths[i]
		}
	}
	switch {
	case sumLen == 0:
		return zeroBitArray
	case max1Len == 0:
		return &BitArray{nBits: sumLen}
	}

	buf1 := make([]byte, (max1Len+7)>>3)
	fillFF(buf1)

	bb := NewBuilder()
	for i, length := range lengths {
		switch {
		case length == 0:
		case i&1 == 0:
			bb.append(nil, 0, length, true)
		default:
			bb.append(buf1, 0, length, false)
		}
	}

	return bb.BitArray()
}

// IsZero returns whether the BitArray is empty, zero length.
func (ba *BitArray) IsZero() bool {
	return ba == nil || ba.nBits == 0
}

// Len returns the number of bits contained in the BitArray.
func (ba *BitArray) Len() int {
	if ba == nil {
		return 0
	}

	return ba.nBits
}

// NumPadding returns the number of LSBs padded when expressing the bit array as
// []byte type, that is, the number of bits to be added to make it a multiple of
// 8 bits.
func (ba *BitArray) NumPadding() int {
	if ba == nil {
		return 0
	}

	return (8 - ba.nBits&7) & 7
}

// String returns the string representation of the BitArray.
func (ba BitArray) String() string {
	if ba.IsZero() {
		return ""
	}
	sb := make([]byte, ba.nBits)
	if ba.b == nil {
		fill30(sb)
	} else {
		for i := 0; i < ba.nBits; i++ {
			sb[i] = '0' + ba.b[i>>3]>>(7-i&7)&1
		}
	}

	return string(sb)
}

// Bytes returns the byte slice containing the bit array. It also returns the
// number of the padded LSBs.
func (ba *BitArray) Bytes() ([]byte, int) {
	n := ba.Len() //nolint:ifshort // false positive
	if n == 0 {
		return []byte{}, 0
	}
	b := make([]byte, (n+7)>>3)
	copy(b, ba.b) // works with ba.b == nil

	return b, (8 - ba.nBits&7) & 7
}

// BitArray implements the BitArrayer interface returning itself.
func (ba *BitArray) BitArray() *BitArray {
	return ba
}

// BitAt returns a single bit at the specified offset as 0 or 1. It panics if
// the off is negative or greater than ba.Len()-1.
func (ba *BitArray) BitAt(off int) byte {
	switch {
	case off < 0:
		panicf("BitAt: negative off %d.", off)
	case ba.Len() <= off:
		panicf("BitAt: out of range: off=%d >= len=%d.", off, ba.Len())
	case ba.b == nil:
		return 0
	}

	return ba.b[off>>3] >> (7 - off&7) & 1
}

// Hash calculates the hash of the bit array using the hash function h. The
// hash.Hash is designed to accept input in bytes instead of bits. This causes
// problems with bit arrays that have padding LSBs at the end. For example, the
// two bit arrays "1111" and "1111000" would both be represented as the same
// single byte 0xf0. In order to prevent these from being mapped to the same
// hash value, the hash is calculated after appending a 3 bits marker
// indicating the number of padding LSBs at the end of the original bit array.
//
// Deprecated: Most hash functions can handle bit-oriented messages as-is by
// design, and it is not appropriate to use the byte-oriented standard hash.Hash
// with padding bits. The result does not comply with the specifications. Not
// all hash functions are available, but for SHA-1 and SHA-2, which can handle
// bit-oriented messages correctly, dedicated methods such as SHA512, SHA256,
// and SHA1 are now available. It is better to use them instead.
func (ba *BitArray) Hash(h hash.Hash) []byte {
	b, _ := ba.MarshalBinary()
	h.Write(b)

	return h.Sum(nil)
}

// MapKey returns a string that can be used as a key for the Go built-in map.
// Only the same bit array returns the same string. The String method can also
// be used for the same purpose, but MapKey is faster. Note that it can be used
// as a map key, but it may contain non-printable characters.
func (ba *BitArray) MapKey() string {
	if ba.IsZero() {
		return ""
	}
	nBytes := (ba.nBits + 7) >> 3
	sb := make([]byte, nBytes+1)
	sb[0] = byte(ba.nBits & 7)
	copy(sb[1:], ba.b) // works with ba.b == nil

	return string(sb)
}

// ToPadded8 returns a new BitArray with a length that is a multiple of 8 bits
// by apending 0 to 7 padding bits at the end. For the returned bit array,
// NumPadding() returns 0.
func (ba *BitArray) ToPadded8() *BitArray {
	switch {
	case ba.IsZero():
		return zeroBitArray
	case ba.nBits&7 == 0:
		return ba
	}
	nBits := (ba.nBits + 7) & ^7
	if ba.b == nil {
		return &BitArray{nBits: nBits}
	}

	return &BitArray{b: ba.b, nBits: nBits}
}

// ToPadded64 returns a new BitArray with a length that is a multiple of 64 bits
// by apending 0 to 63 padding bits at the end. For the returned bit array,
// NumPadding() returns 0, and Len() returns a multiple of 8.
func (ba *BitArray) ToPadded64() *BitArray {
	switch {
	case ba.IsZero():
		return zeroBitArray
	case ba.nBits&63 == 0:
		return ba
	}
	nBits := (ba.nBits + 63) & ^63
	if ba.b == nil {
		return &BitArray{nBits: nBits}
	}

	return &BitArray{b: ba.b[:nBits>>3], nBits: nBits}
}

// ToByteBits returns a byte slice that represents the bit array with 1 byte
// per bit. Each byte element of the returned slice represents a single bit with
// 0 or 1. It is a memory-wasting data type, but for the purpose of repeating
// searches and matching using the same bit array, converting to this format
// allows the standard bytes package to be used.
func (ba *BitArray) ToByteBits() []byte {
	if ba.IsZero() {
		return []byte{}
	}
	return ba.bits8()
}

// ParityBit calculates the odd parity bit of the bit array.
func (ba *BitArray) ParityBit() int {
	if ba.IsZero() || ba.b == nil {
		return 1
	}

	// TODO: use an optimized algorithm
	var sum uint64
	for _, b := range asUint64Slice(ba.b) {
		sum ^= b
	}

	return (bits.OnesCount64(sum) + 1) & 1
}

// RepeatEach returns a new BitArray in which each bit is repeated the specified
// number of times. It is an operation like "scaling" a bit pattern.
func (ba *BitArray) RepeatEach(count int) *BitArray {
	switch {
	case count < 0:
		panicf("RepeatEach: negative count %d.", count)
	case ba.IsZero(), count == 0:
		return zeroBitArray
	case count == 1:
		return ba
	case ba.b == nil:
		return &BitArray{nBits: ba.nBits * count}
	}

	buf1 := make([]byte, (count+7)>>3)
	fillFF(buf1)

	bb := NewBuilder()
	for i := 0; i < ba.nBits; i++ {
		if ba.b[i>>3]>>(7-i&7)&1 == 0 {
			bb.append(nil, 0, count, true)
		} else {
			bb.append(buf1, 0, count, false)
		}
	}

	return bb.BitArray()
}
