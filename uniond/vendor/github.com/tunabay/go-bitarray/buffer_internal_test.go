// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

import (
	"fmt"
)

// D returns the string representing of its internal state.
func (buf *Buffer) D() string {
	if buf == nil {
		return "<nil>"
	}
	return fmt.Sprintf("BUF{nbit=%d, off=%d, b=%08b}", buf.nBits, buf.off, buf.b)
}

// V validate the internal data representation. It panics on failure.
func (buf *Buffer) V() {
	switch {
	case buf == nil:
		return

	case buf.nBits < 0:
		panicf("V: negative nBits %d", buf.nBits)

	case buf.b != nil && len(buf.b) == 0:
		panicf("V: buf.b is an empty slice, must be nil: %08b", buf.b)

	case buf.b == nil && buf.nBits != 0:
		panicf("V: buf.b is nil, must be non nil for nbits=%d", buf.nBits)

	case buf.b == nil:
		return

	case buf.off < 0:
		panicf("V: negative off %d", buf.off)

	case len(buf.b) < (buf.off+buf.nBits+7)>>3:
		panicf(
			"V: short buf: off=%d, nBits=%d, reqB=%d, lenB=%d: %08b",
			buf.off, buf.nBits, (buf.off+buf.nBits+7)>>3, len(buf.b),
			buf.b,
		)
	}
}
