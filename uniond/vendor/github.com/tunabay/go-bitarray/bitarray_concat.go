// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

// Join concatenates the elements of its first parameter to create a single
// bit array. The separator sep is placed between elements in the result.
func Join(elems []*BitArray, sep BitArrayer) *BitArray {
	var basep *BitArray
	if sep != nil {
		basep = sep.BitArray()
	}
	switch len(elems) {
	case 0:
		return zeroBitArray
	case 1:
		return elems[0]
	}
	bb := NewBuilder()
	for i, elem := range elems {
		if i != 0 && basep != nil {
			_, _ = bb.WriteBitArray(basep)
		}
		_, _ = bb.WriteBitArray(elem)
	}

	return bb.BitArray()
}

// JoinBitArrayer is identical to Join except that it accepts elems in
// []BitArrayer type instead of []*BitArray type.
func JoinBitArrayer(elems []BitArrayer, sep BitArrayer) *BitArray {
	var basep *BitArray
	if sep != nil {
		basep = sep.BitArray()
	}
	switch len(elems) {
	case 0:
		return zeroBitArray
	case 1:
		if elems[0] == nil {
			return zeroBitArray
		}
		return elems[0].BitArray()
	}
	bb := NewBuilder()
	for i, elem := range elems {
		if i != 0 && basep != nil {
			_, _ = bb.WriteBitArray(basep)
		}
		if elem != nil {
			_, _ = bb.WriteBitArray(elem)
		}
	}

	return bb.BitArray()
}

// Append returns the new BitArray resulting from appending the passed elements
// to the current bit array.
func (ba *BitArray) Append(bas ...BitArrayer) *BitArray {
	switch len(bas) {
	case 0:
		if ba.IsZero() {
			return zeroBitArray
		}
		return ba
	case 1:
		if bas[0] == nil {
			return ba
		}
		return ba.append1(bas[0])
	}

	bb := NewBuilder(ba)
	for _, bai := range bas {
		_, _ = bb.WriteBitArray(bai)
	}

	return bb.BitArray()
}

func (ba *BitArray) append1(x BitArrayer) *BitArray {
	var bax *BitArray
	if x != nil {
		bax = x.BitArray()
	}
	switch {
	case ba.IsZero():
		if bax.IsZero() {
			return zeroBitArray
		}
		return bax
	case bax.IsZero():
		return ba
	}
	if bax.b == nil {
		nBits := ba.nBits + bax.nBits
		if ba.b == nil {
			return &BitArray{nBits: nBits}
		}
		nBytes := (nBits + 7) >> 3
		if nBytes <= cap(ba.b) {
			return &BitArray{b: ba.b[:nBytes], nBits: nBits}
		}
		buf := allocByteSlice(nBytes)
		copy(buf, ba.b)
		return &BitArray{b: buf, nBits: nBits}
	}
	nBits := ba.nBits + bax.nBits
	buf := allocByteSlice((nBits + 7) >> 3)
	copy(buf, ba.b)
	if copyBits(buf, bax.b, ba.nBits, 0, bax.nBits) && ba.b == nil {
		return &BitArray{nBits: nBits}
	}
	return &BitArray{b: buf, nBits: nBits}
}

// Repeat returns a bit array consisting of count copies of the bit array ba.
func (ba *BitArray) Repeat(count int) *BitArray {
	switch {
	case count < 0:
		panicf("invalid count: %d < 0", count)
	case ba.IsZero(), count == 0:
		return zeroBitArray
	case count == 1:
		return ba
	case ba.b == nil:
		return &BitArray{nBits: ba.nBits * count}
	}
	bb := NewBuilder()
	for i := 0; i < count; i++ {
		_, _ = bb.WriteBitArray(ba)
	}

	return bb.BitArray()
}
