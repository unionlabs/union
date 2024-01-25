// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

// CopyBitsFromBytes reads nBits bits from b at the offset bOff, and write them
// into the buffer at the offset off.
func (buf *Buffer) CopyBitsFromBytes(off int, b []byte, bOff, nBits int) {
	switch {
	case off < 0:
		panicf("CopyBitsFromBytes: negative off %d.", off)
	case buf.nBits < off+nBits:
		panicf("CopyBitsFromBytes: out of range: off=%d + nBits=%d > len=%d.", off, nBits, buf.nBits)
	case nBits == 0:
		return
	}
	copyBits(buf.b, b, buf.off+off, bOff, nBits)
}

// CopyBitsToBytes reads nBits bits of the buffer starting at the offset off,
// and write them into the byte slice b at the offset bOff.
func (buf *Buffer) CopyBitsToBytes(off int, b []byte, bOff, nBits int) {
	switch {
	case off < 0:
		panicf("CopyBitsToBytes: negative off %d.", off)
	case buf.nBits < off+nBits:
		panicf("CopyBitsToBytes: out of range: off=%d + nBits=%d > len=%d.", off, nBits, buf.nBits)
	case nBits == 0:
		return
	}
	copyBits(b, buf.b, bOff, buf.off+off, nBits)
}

// CopyBits copies bits from src into dst. CopyBits returns the number of bits
// copied, which will be the minimum of src.Len() and dst.Len().
func CopyBits(dst, src *Buffer) int {
	nBits := dst.Len()
	if sLen := src.Len(); sLen < nBits {
		nBits = sLen
	}
	if nBits != 0 {
		copyBits(dst.b, src.b, dst.off, src.off, nBits)
	}

	return nBits
}

// CopyBitsN is identical to CopyBits except that it copies up to nBits bits.
func CopyBitsN(dst, src *Buffer, nBits int) int {
	if dLen := dst.Len(); dLen < nBits {
		nBits = dLen
	}
	if sLen := src.Len(); sLen < nBits {
		nBits = sLen
	}
	if nBits != 0 {
		copyBits(dst.b, src.b, dst.off, src.off, nBits)
	}

	return nBits
}

// CopyBitsPartial is identical to CopyBitsN except that it reads and writes
// bits starting at specified offsets rather than the first bits.
func CopyBitsPartial(dst, src *Buffer, dstOff, srcOff, nBits int) int {
	if dLen := dst.Len() - dstOff; dLen < nBits {
		nBits = dLen
	}
	if sLen := src.Len() - srcOff; sLen < nBits {
		nBits = sLen
	}
	if nBits != 0 {
		copyBits(dst.b, src.b, dst.off+dstOff, src.off+srcOff, nBits)
	}

	return nBits
}
