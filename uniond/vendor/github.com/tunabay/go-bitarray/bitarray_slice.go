// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

// Slice returns a new BitArray extracted as a subpart from the bit array in
// the same manner as Go's native slices. The two arguments start and end
// specify the indexes of the bits to select. 0 points to the first bit and
// ba.Len()-1 points to the last bit. The start and end select a half-open
// range which includes the start, but excludes the end. If the index is
// outside the range of the bit array, Slice will panic.
func (ba *BitArray) Slice(start, end int) *BitArray {
	switch {
	case start < 0, ba.Len() < start:
		panicf("Slice: start %d out of range: 0..%d.", start, ba.Len())
	case end < 0, ba.Len() < end:
		panicf("Slice: end %d out of range: 0..%d.", end, ba.Len())
	case end < start:
		panicf("Slice: invalid range start=%d > end=%d.", start, end)
	}
	nBits := end - start
	switch {
	case nBits == 0:
		return zeroBitArray
	case start == 0 && end == ba.nBits:
		return ba
	case ba.b == nil:
		return &BitArray{nBits: nBits}
	case start == 0 && ba.hasTrailingZeros(ba.nBits-end):
		return &BitArray{b: ba.b[:(end+7)>>3], nBits: nBits}
	}
	buf := allocByteSlice((nBits + 7) >> 3)
	if copyBits(buf, ba.b, 0, start, nBits) {
		return &BitArray{nBits: nBits}
	}

	return &BitArray{b: buf, nBits: nBits}
}

// SliceToEnd is shorthand for Slice(start, ba.Len()) and returns the subpart
// from the position specified start to the last bit.
func (ba *BitArray) SliceToEnd(start int) *BitArray { return ba.Slice(start, ba.Len()) }

// ToWidth returns a new BitArray resized to wid bits with its contents
// preserved. If wid is less than ba.Len(), some bits will be lost. If wid is
// greater than be.Len(), the expanded space will be filled with 0s. In both
// cases, the MSBs or LSBs are fixed according to the specified align.
func (ba *BitArray) ToWidth(wid int, align Alignment) *BitArray {
	switch {
	case wid < 0:
		panicf("ToWidth: negative wid %d.", wid)
	case wid == 0:
		return zeroBitArray
	case ba.IsZero(), ba.b == nil:
		return &BitArray{nBits: wid}
	case wid == ba.nBits:
		return ba
	case wid < ba.nBits:
		if align == AlignLeft {
			return ba.Slice(0, wid)
		}
		return ba.Slice(ba.nBits-wid, ba.nBits)
	}
	add := &BitArray{nBits: wid - ba.nBits}
	if align == AlignLeft {
		return ba.append1(add)
	}
	return add.append1(ba)
}

// TrimPrefix returns a new BitArray with the leading prefix removed. If the bit
// array does not start with prefix, ba itself is returned unchanged.
func (ba *BitArray) TrimPrefix(prefix BitArrayer) *BitArray {
	var baPrefix *BitArray
	if prefix != nil {
		baPrefix = prefix.BitArray()
	}
	switch {
	case ba.IsZero():
		return zeroBitArray
	case !ba.HasPrefix(baPrefix), baPrefix.IsZero():
		return ba
	}

	return ba.Slice(baPrefix.nBits, ba.nBits)
}

// TrimSuffix returns a new BitArray with the trailing suffix removed. If the
// bit array does not end with prefix, ba itself is returned unchanged.
func (ba *BitArray) TrimSuffix(suffix BitArrayer) *BitArray {
	var baSuffix *BitArray
	if suffix != nil {
		baSuffix = suffix.BitArray()
	}
	switch {
	case ba.IsZero():
		return zeroBitArray
	case !ba.HasSuffix(baSuffix), baSuffix.IsZero():
		return ba
	}

	return ba.Slice(0, ba.nBits-baSuffix.nBits)
}

// TrimLeadingZeros returns a new BitArray with the leading zeros removed.
func (ba *BitArray) TrimLeadingZeros() *BitArray {
	if ba.IsZero() || ba.b == nil {
		return zeroBitArray
	}

	return ba.Slice(ba.LeadingZeros(), ba.nBits)
}

// TrimTrailingZeros returns a new BitArray with the trailing zeros removed.
func (ba *BitArray) TrimTrailingZeros() *BitArray {
	if ba.IsZero() || ba.b == nil {
		return zeroBitArray
	}

	return ba.Slice(0, ba.nBits-ba.TrailingZeros())
}
