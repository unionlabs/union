// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

import (
	"io"
)

// Reader implements io.Reader and io.ByteReader interfaces by reading from a
// BitArray.
//
// The standard io.Reader family interfaces define byte-oriented reading
// methods, rather than bit-oriented. And they always read in bytes and cannot
// read fractional bits less than 8 bits. When the number of remaining available
// bits is not a multiple of 8, the io.EOF may be reached with fractional bits
// remaining. In this case, the padding bits are not added automatically and an
// ErrFractionalBitsBeforeEOF is returned. To handle these trailing fractional
// bits, use bit-oriented reads together, or use ToPadded8 etc in advance to
// make sure that the number of bits in the source is a multiple of 8 bits.
type Reader struct {
	ba *BitArray
	i  int
}

// NewReader returns a new Reader reading from the bit array ba. For mutable
// types of ba, the value at the time NewReader is called is copied and
// subsequent changes will not be reflected in future readings.
func NewReader(ba BitArrayer) *Reader {
	if ba == nil {
		return &Reader{ba: zeroBitArray}
	}

	return &Reader{ba: ba.BitArray()}
}

// Reset resets the Reader to be reading from ba.
func (r *Reader) Reset(ba BitArrayer) {
	if ba == nil {
		r.ba = zeroBitArray
	} else {
		r.ba = ba.BitArray()
	}
	r.i = 0
}

// Read implements the io.Reader interface. It reads up to 8 * len(p) bits from
// the underlying BitArray, writes them to p as bytes, and returns the number of
// bytes read. It always reads in bytes, and if only fractional bits less than 8
// bits are available, it returns an ErrFractionalBitsBeforeEOF.
func (r *Reader) Read(p []byte) (int, error) {
	switch {
	case r.ba.IsZero(), r.ba.nBits <= r.i:
		return 0, io.EOF
	case r.ba.nBits-8 < r.i:
		return 0, ErrFractionalBitsBeforeEOF
	}
	nBytes := len(p)
	if n := (r.ba.Len() - r.i) >> 3; n < nBytes {
		nBytes = n
	}
	switch {
	case nBytes == 0:
		return 0, nil
	case r.ba.b == nil:
		fill00(p[:nBytes])
		r.i += nBytes << 3
		return nBytes, nil
	}
	_ = copyBits(p[:nBytes], r.ba.b, 0, r.i, nBytes<<3)
	r.i += nBytes << 3
	return nBytes, nil
}

// ReadByte implements the io.ByteReader interface. It reads 8 bits from the
// underlying BitArray and returns them as a byte. If only fractional bits less
// than 8 bits are available, it returns an ErrFractionalBitsBeforeEOF.
func (r *Reader) ReadByte() (byte, error) {
	switch {
	case r.ba.IsZero(), r.ba.nBits <= r.i:
		return 0, io.EOF
	case r.ba.nBits-8 < r.i:
		return 0, ErrFractionalBitsBeforeEOF
	}
	if r.ba.b == nil {
		r.i += 8
		return 0, nil
	}
	b := make([]byte, 1)
	_ = copyBits(b, r.ba.b, 0, r.i, 8)
	r.i += 8
	return b[0], nil
}

// ReadBitArray reads up to nBits bits as a BitArray. The returned BitArray may
// be shorter than nBits. The caller should check the length of the returned
// bit array and handle it.
func (r *Reader) ReadBitArray(nBits int) (*BitArray, error) {
	switch {
	case nBits < 0:
		panicf("ReadBitArray: negative nBits %d.", nBits)
	case nBits == 0:
		return zeroBitArray, nil
	case r.ba.IsZero(), r.ba.nBits <= r.i:
		return zeroBitArray, io.EOF
	}
	if n := r.ba.nBits - r.i; n < nBits {
		nBits = n
	}
	ba := r.ba.Slice(r.i, r.i+nBits)
	r.i += nBits

	return ba, nil
}

// ReadBit reads a single bit as 0 or 1.
func (r *Reader) ReadBit() (byte, error) {
	if r.ba.IsZero() || r.ba.nBits <= r.i {
		return 0, io.EOF
	}
	if r.ba.b == nil {
		r.i++
		return 0, nil
	}
	b := r.ba.b[r.i>>3] >> (7 - r.i&7) & 1
	r.i++
	return b, nil
}

// ReadBits reads up to p.Len() bits into p. It returns the number of bits read
// (0 <= n <= p.Len()) and any error encountered. The data in the buffer beyond
// the returned length is undefined.
func (r *Reader) ReadBits(p *Buffer) (int, error) {
	nBits := p.Len()
	switch {
	case nBits == 0:
		return 0, nil
	case r.ba.IsZero(), r.ba.nBits <= r.i:
		return 0, io.EOF
	}
	if n := r.ba.nBits - r.i; n < nBits {
		nBits = n
	}
	if r.ba.b == nil {
		clearBits(p.b, 0, nBits)
	} else {
		_ = copyBits(p.b, r.ba.b, 0, r.i, nBits)
	}
	r.i += nBits

	return nBits, nil
}

/*
// TODO:
// WriteTo implements the io.WriterTo interface.
func (r *Reader) WriteTo(w io.Writer) (int64, error) {}
*/
