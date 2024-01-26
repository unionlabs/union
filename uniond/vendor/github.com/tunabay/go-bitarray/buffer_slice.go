// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

// Slice extracts a subpart from the buffer and returns it as a new Buffer in
// the same manner as Go's native slices. Note that like Go's native slices, the
// sliced buffers share memory with the original buffer. Changes to the original
// buffer affect slices and vice versa. Slice does not perform bit-shifting,
// even when creating slices that are not aligned to byte boundaries. It just
// records the offset and length for reference.
//
// The two arguments start and end specify the indexes of the bits to select. 0
// points to the first bit and buf.Len()-1 points to the last bit. The start and
// end select a half-open range which includes the start, but excludes the end.
// If the index is outside the range of the buffer, Slice will panic.
func (buf *Buffer) Slice(start, end int) *Buffer {
	switch {
	case start < 0, buf.Len() < start:
		panicf("Slice: start %d out of range: 0..%d.", start, buf.Len())
	case end < 0, buf.Len() < end:
		panicf("Slice: end %d out of range: 0..%d.", end, buf.Len())
	case end < start:
		panicf("Slice: invalid range start=%d > end=%d.", start, end)
	case start == end:
		return &Buffer{}
	}
	off := buf.off + start
	return &Buffer{
		b:     buf.b[off>>3 : (buf.off+end+7)>>3],
		nBits: end - start,
		off:   off & 7,
	}
}

// SliceToEnd is shorthand for Slice(start, buf.Len()) and returns the subpart
// from the position specified start to the last bit.
func (buf *Buffer) SliceToEnd(start int) *Buffer { return buf.Slice(start, buf.Len()) }
