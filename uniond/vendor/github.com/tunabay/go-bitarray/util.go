// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

import (
	"fmt"
	"unsafe"
)

// dstOff < 8, srcOff < 8, nBits < 8, dstOff + nBits <= 8
func copyBits7(dst *byte, src []byte, dstOff, srcOff, nBits int) bool {
	nBits0, srcBits0 := nBits, 8-srcOff
	if srcBits0 < nBits {
		nBits0 = srcBits0
	}
	mask := byte(0xff) << (8 - nBits0) >> dstOff
	*dst &= ^mask
	*dst |= mask & (src[0] << srcOff >> dstOff)

	zf := src[0] == 0
	if nBits0 < nBits {
		zf2 := copyBits7(dst, src[1:], dstOff+nBits0, 0, nBits-nBits0)
		zf = zf && zf2
	}

	return zf
}

func copyBits(dst, src []byte, dstOff, srcOff, nBits int) bool {
	dst, src = dst[dstOff>>3:], src[srcOff>>3:]
	dstOff, srcOff = dstOff&7, srcOff&7

	zf := true
	if dstOff != 0 {
		nBits0 := 8 - dstOff
		if nBits <= nBits0 {
			zf2 := copyBits7(&dst[0], src, dstOff, srcOff, nBits)
			return zf2
		}
		zf2 := copyBits7(&dst[0], src, dstOff, srcOff, nBits0)
		zf = zf && zf2

		nBits -= nBits0
		dst = dst[1:]
		srcOff += nBits0
		src = src[srcOff>>3:]
		srcOff &= 7
	}

	if nBytes1 := nBits >> 3; 0 < nBytes1 {
		if srcOff == 0 {
			copy(dst, src[:nBytes1])
			zf = false
		} else {
			rsh := 8 - srcOff
			for i := 0; i < nBytes1; i++ {
				dst[i] = src[i]<<srcOff | src[i+1]>>rsh
				zf = zf && dst[i] == 0
			}
		}
		nBits &= 7
		if nBits == 0 {
			return zf
		}
		dst = dst[nBytes1:]
		src = src[nBytes1:]
	}

	zf2 := copyBits7(&dst[0], src, 0, srcOff, nBits)

	return zf && zf2
}

func clearBits(b []byte, off, nBits int) {
	b = b[off>>3:]
	off &= 7

	if off != 0 {
		nBits0 := 8 - off
		if nBits <= nBits0 {
			b[0] &^= byte(0xff) << (8 - nBits) >> off
			return
		}
		b[0] &= byte(0xff) << nBits0
		nBits -= nBits0
		b = b[1:]
	}

	if nBytes1 := nBits >> 3; 0 < nBytes1 {
		for i := 0; i < nBits>>3; i++ {
			b[i] = 0
		}
		nBits &= 7
		b = b[nBytes1:]
	}

	if nBits != 0 {
		b[0] &= byte(0xff) >> nBits
	}
}

func setBits(b []byte, off, nBits int) {
	b = b[off>>3:]
	off &= 7

	if off != 0 {
		nBits0 := 8 - off
		if nBits <= nBits0 {
			b[0] |= byte(0xff) << (8 - nBits) >> off
			return
		}
		b[0] |= byte(0xff) >> off
		nBits -= nBits0
		b = b[1:]
	}

	if nBytes1 := nBits >> 3; 0 < nBytes1 {
		for i := 0; i < nBits>>3; i++ {
			b[i] = 0xff
		}
		nBits &= 7
		b = b[nBytes1:]
	}

	if nBits != 0 {
		b[0] |= byte(0xff) << (8 - nBits)
	}
}

func toggleBits(b []byte, off, nBits int) {
	b = b[off>>3:]
	off &= 7

	if off != 0 {
		nBits0 := 8 - off
		if nBits <= nBits0 {
			b[0] ^= byte(0xff) << (8 - nBits) >> off
			return
		}
		b[0] ^= byte(0xff) >> off
		nBits -= nBits0
		b = b[1:]
	}

	if nBytes1 := nBits >> 3; 0 < nBytes1 {
		for i := 0; i < nBits>>3; i++ {
			b[i] ^= 0xff
		}
		nBits &= 7
		b = b[nBytes1:]
	}

	if nBits != 0 {
		b[0] ^= byte(0xff) << (8 - nBits)
	}
}

// dstOff < 8, srcOff < 8, nBits < 8, dstOff + nBits <= 8
func andBits7(dst *byte, src []byte, dstOff, srcOff, nBits int) {
	nBits0, srcBits0 := nBits, 8-srcOff
	if srcBits0 < nBits {
		nBits0 = srcBits0
	}
	mask := byte(0xff) << (8 - nBits0) >> dstOff
	*dst &= ^mask | (src[0] << srcOff >> dstOff)

	if nBits0 < nBits {
		andBits7(dst, src[1:], dstOff+nBits0, 0, nBits-nBits0)
	}
}

func andBits(dst, src []byte, dstOff, srcOff, nBits int) {
	dst, src = dst[dstOff>>3:], src[srcOff>>3:]
	dstOff, srcOff = dstOff&7, srcOff&7

	if dstOff != 0 {
		nBits0 := 8 - dstOff
		if nBits <= nBits0 {
			andBits7(&dst[0], src, dstOff, srcOff, nBits)
			return
		}
		andBits7(&dst[0], src, dstOff, srcOff, nBits0)

		nBits -= nBits0
		dst = dst[1:]
		srcOff += nBits0
		src = src[srcOff>>3:]
		srcOff &= 7
	}

	// TODO: maybe can be optimized using asUint64Slice()
	if nBytes1 := nBits >> 3; 0 < nBytes1 {
		if srcOff == 0 {
			for i := 0; i < nBytes1; i++ {
				dst[i] &= src[i]
			}
		} else {
			rsh := 8 - srcOff
			for i := 0; i < nBytes1; i++ {
				dst[i] &= src[i]<<srcOff | src[i+1]>>rsh
			}
		}
		nBits &= 7
		if nBits == 0 {
			return
		}
		dst = dst[nBytes1:]
		src = src[nBytes1:]
	}

	andBits7(&dst[0], src, 0, srcOff, nBits)
}

// dstOff < 8, srcOff < 8, nBits < 8, dstOff + nBits <= 8
func orBits7(dst *byte, src []byte, dstOff, srcOff, nBits int) {
	nBits0, srcBits0 := nBits, 8-srcOff
	if srcBits0 < nBits {
		nBits0 = srcBits0
	}
	mask := byte(0xff) << (8 - nBits0) >> dstOff
	*dst |= mask & (src[0] << srcOff >> dstOff)

	if nBits0 < nBits {
		orBits7(dst, src[1:], dstOff+nBits0, 0, nBits-nBits0)
	}
}

func orBits(dst, src []byte, dstOff, srcOff, nBits int) {
	dst, src = dst[dstOff>>3:], src[srcOff>>3:]
	dstOff, srcOff = dstOff&7, srcOff&7

	if dstOff != 0 {
		nBits0 := 8 - dstOff
		if nBits <= nBits0 {
			orBits7(&dst[0], src, dstOff, srcOff, nBits)
			return
		}
		orBits7(&dst[0], src, dstOff, srcOff, nBits0)

		nBits -= nBits0
		dst = dst[1:]
		srcOff += nBits0
		src = src[srcOff>>3:]
		srcOff &= 7
	}

	// TODO: maybe can be optimized using asUint64Slice()
	if nBytes1 := nBits >> 3; 0 < nBytes1 {
		if srcOff == 0 {
			for i := 0; i < nBytes1; i++ {
				dst[i] |= src[i]
			}
		} else {
			rsh := 8 - srcOff
			for i := 0; i < nBytes1; i++ {
				dst[i] |= src[i]<<srcOff | src[i+1]>>rsh
			}
		}
		nBits &= 7
		if nBits == 0 {
			return
		}
		dst = dst[nBytes1:]
		src = src[nBytes1:]
	}

	orBits7(&dst[0], src, 0, srcOff, nBits)
}

// dstOff < 8, srcOff < 8, nBits < 8, dstOff + nBits <= 8
func xorBits7(dst *byte, src []byte, dstOff, srcOff, nBits int) {
	nBits0, srcBits0 := nBits, 8-srcOff
	if srcBits0 < nBits {
		nBits0 = srcBits0
	}
	mask := byte(0xff) << (8 - nBits0) >> dstOff
	*dst ^= mask & (src[0] << srcOff >> dstOff)

	if nBits0 < nBits {
		xorBits7(dst, src[1:], dstOff+nBits0, 0, nBits-nBits0)
	}
}

func xorBits(dst, src []byte, dstOff, srcOff, nBits int) {
	dst, src = dst[dstOff>>3:], src[srcOff>>3:]
	dstOff, srcOff = dstOff&7, srcOff&7

	if dstOff != 0 {
		nBits0 := 8 - dstOff
		if nBits <= nBits0 {
			xorBits7(&dst[0], src, dstOff, srcOff, nBits)
			return
		}
		xorBits7(&dst[0], src, dstOff, srcOff, nBits0)

		nBits -= nBits0
		dst = dst[1:]
		srcOff += nBits0
		src = src[srcOff>>3:]
		srcOff &= 7
	}

	// TODO: maybe can be optimized using asUint64Slice()
	if nBytes1 := nBits >> 3; 0 < nBytes1 {
		if srcOff == 0 {
			for i := 0; i < nBytes1; i++ {
				dst[i] ^= src[i]
			}
		} else {
			rsh := 8 - srcOff
			for i := 0; i < nBytes1; i++ {
				dst[i] ^= src[i]<<srcOff | src[i+1]>>rsh
			}
		}
		nBits &= 7
		if nBits == 0 {
			return
		}
		dst = dst[nBytes1:]
		src = src[nBytes1:]
	}

	xorBits7(&dst[0], src, 0, srcOff, nBits)
}

func allBytesZero(b []byte) bool {
	for _, u64 := range asUint64Slice(b) {
		if u64 != 0 {
			return false
		}
	}
	return true
}

func panicf(format string, v ...interface{}) {
	panic("bitarray: " + fmt.Sprintf(format, v...))
}

func allocByteSlice(nBytes int) []byte {
	return make([]byte, nBytes, (nBytes+7) & ^7)
}

func asUint64Slice(b []byte) []uint64 {
	return unsafe.Slice((*uint64)(unsafe.Pointer(&b[0])), (len(b)+7)>>3)
}

func fill00(b []byte) {
	// This seems to get optimized into a memclr.
	for i := range b {
		b[i] = 0
	}
	// copy(b, []byte{0, 0, 0, 0, 0, 0, 0, 0})
	// for n := 8; n < len(b); n <<= 1 {
	// 	copy(b[n:], b[:n])
	// }
}

func fillFF(b []byte) {
	copy(b, []byte{0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff})
	for n := 8; n < len(b); n <<= 1 {
		copy(b[n:], b[:n])
	}
}

func fill30(b []byte) {
	copy(b, []byte{0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30})
	for n := 8; n < len(b); n <<= 1 {
		copy(b[n:], b[:n])
	}
}

// ba != nil
func (ba *BitArray) bits8() []byte {
	b8 := make([]byte, ba.nBits)
	if ba.b == nil {
		return b8
	}
	for i := 0; i < ba.nBits; i++ {
		b8[i] = ba.b[i>>3] >> (7 - i&7) & 1
	}
	return b8
}
