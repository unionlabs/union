// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

import (
	"bytes"
	"fmt"
	"io"
)

// Builder is used to efficiently build a BitArray using Write methods. It
// minimizes memory copying and bit shifting.
//
// The zero value for Builder is already to use. Do not copy a non-zero Builder.
// It is not safe for concurrent use by multiple goroutines.
type Builder struct {
	head, tail *builderChunk
	nBits      int
}

// NewBuilder creates a Builder with bit arrays as the initial contents.
func NewBuilder(bas ...BitArrayer) *Builder {
	b := &Builder{}
	for _, bai := range bas {
		if bai == nil {
			continue
		}
		ba := bai.BitArray()
		if !ba.IsZero() {
			b.append(ba.b, 0, ba.nBits, ba.b == nil)
		}
	}

	return b
}

type builderChunk struct {
	b     []byte // can be nil when zf == true
	nBits int    // 0 < nBits
	off   int    // off must be canonicalized to 0..7
	zf    bool   // zero-filled and immutable
	next  *builderChunk
}

// BitArray builds an immutable BitArray from accumulated bits. References to
// the accumulated byte slices are copied when this BitArray method is called.
func (b *Builder) BitArray() *BitArray {
	if b.nBits == 0 {
		return zeroBitArray
	}
	buf := allocByteSlice((b.nBits + 7) >> 3)
	idx, zf := 0, true
	for c := b.head; c != nil; c = c.next {
		if c.zf {
			idx += c.nBits
			continue
		}
		zf2 := copyBits(buf, c.b, idx, c.off, c.nBits)
		zf = zf && zf2
		idx += c.nBits
	}
	if zf {
		return &BitArray{nBits: b.nBits}
	}

	return &BitArray{b: buf, nBits: b.nBits}
}

// String returns the string representation of the bit array being built. The
// result is the same as b.BitArray().String().
func (b *Builder) String() string {
	sb := make([]byte, b.nBits)
	idx := 0
	for c := b.head; c != nil; c = c.next {
		if c.zf {
			s := idx
			idx += c.nBits
			fill30(sb[s:idx])
			continue
		}
		for i := c.off; i < c.off+c.nBits; i++ {
			sb[idx] = '0' + c.b[i>>3]>>(7-i&7)&1
			idx++
		}
	}
	return string(sb)
}

func (b *Builder) append(buf []byte, off, nBits int, zf bool) {
	if nBits == 0 {
		return
	}
	c := &builderChunk{b: buf, nBits: nBits, off: off, zf: zf}
	if b.head == nil {
		b.head, b.tail = c, c
	} else {
		b.tail.next = c
		b.tail = c
	}
	b.nBits += nBits
}

// Reset resets the builder state to empty. All the bits accumulated by writing
// methods are discarded.
func (b *Builder) Reset() {
	b.head, b.tail, b.nBits = nil, nil, 0
}

// Len returns the current number of bits accumurated.
func (b *Builder) Len() int {
	return b.nBits
}

// WriteBitsFromBytes adds the number of bits specified by nBits from the byte
// slice p to the Builder. It skips the off bits from the beginning of p and
// reads up to 8 bits from each byte from the MSB to the LSB.
//
// WriteBitsFromBytes only references the slice and offset of p, and does not
// copy the contents of p. Therefore, any changes to the contents of p before
// calling the BitArray() or String() methods are affected. Be especially
// careful when using a same buffer for iterations.
func (b *Builder) WriteBitsFromBytes(p []byte, off, nBits int) {
	if len(p) == 0 || nBits == 0 {
		return
	}
	p = p[off>>3:]
	off &= 7
	b.append(p[:(off+nBits+7)>>3], off, nBits, false)
}

// Write implements io.Writer by writing 8 * len(p) bits read from a byte slice
// p. Write copies p once because the io.Writer prohibits the implementation to
// retain p. Use WriteBitsFromBytes to avoid copying. Write always returns
// len(p), nil.
func (b *Builder) Write(p []byte) (int, error) {
	buf := make([]byte, len(p))
	copy(buf, p)
	b.append(buf, 0, len(p)<<3, false)

	return len(p), nil
}

// WriteByte implements io.ByteWriter by writing 8 bits read from a byte c.
// WriteByte always returns a nil error.
func (b *Builder) WriteByte(c byte) error {
	b.append([]byte{c}, 0, 8, c == 0)

	return nil
}

// ReadFrom implements io.ReaderFrom. It reads bytes from r until io.EOF or an
// error, and appends the bits read to the builder. Even if an error occurs, the
// bits that could be read before the error are appended to the builder.
func (b *Builder) ReadFrom(r io.Reader) (int64, error) {
	var buf bytes.Buffer
	n, err := buf.ReadFrom(r)
	if 0 < n {
		b.append(buf.Bytes(), 0, int(n)<<3, false)
	}
	if err != nil {
		return n, fmt.Errorf("read failure: %w", err)
	}

	return n, nil
}

// WriteBit writes a single bit to the builder. The bit should be 0 or 1,
// otherwise its LSB is silently used. It always returns a nil error.
func (b *Builder) WriteBit(bit byte) error {
	b.append([]byte{bit}, 7, 1, bit&1 == 0)

	return nil
}

// WriteBitArray writes a bit array to the builder. It always returns the length
// of the bit array and a nil error.
func (b *Builder) WriteBitArray(x BitArrayer) (int, error) {
	if x == nil {
		return 0, nil
	}
	bax := x.BitArray()
	if bax.IsZero() {
		return 0, nil
	}

	b.append(bax.b, 0, bax.nBits, bax.b == nil)

	return bax.Len(), nil
}

// WriteByteBits adds to the Builder the bits read from a byte slice where each
// element contains individual bits. Each element of bits should be 0 or 1,
// otherwise only its LSB is used silently. WriteByteBits copies the bits from
// bits, so any future changes to bits will not affect the contents of Builder.
func (b *Builder) WriteByteBits(bits []byte) {
	if len(bits) == 0 {
		return
	}
	var zfb byte
	buf := make([]byte, (len(bits)+7)>>3)
	for i, bit := range bits {
		bit &= 1
		zfb |= bit
		buf[i>>3] |= bit << (7 - i&7)
	}
	b.append(buf, 0, len(bits), zfb == 0)
}

// WriteBits write the bits read from a Buffer p. WriteBits copies p and is
// unaffected by changes to p after the call. It always returns p.Len() and nil
// error.
func (b *Builder) WriteBits(p *Buffer) (int, error) {
	if p.IsZero() {
		return 0, nil
	}
	p = p.Clone()
	b.append(p.b, p.off, p.nBits, false)

	return p.nBits, nil
}

/*
// Unwrite discards the last nBits bits of the bits already written. It is
// useful for removing trailing padding bits after writing using a
// byte-oriented method.
func (b *Builder) Unwrite(nBits int) {}
*/
