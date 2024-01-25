// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

import (
	"bytes"
)

// HasPrefix reports whether the bit array starts with prefix.
func (ba *BitArray) HasPrefix(prefix BitArrayer) bool {
	var baPrefix *BitArray
	if prefix != nil {
		baPrefix = prefix.BitArray()
	}
	switch {
	case baPrefix.IsZero():
		return true
	case ba.IsZero(), ba.nBits < baPrefix.nBits:
		return false
	case ba.b == nil:
		return baPrefix.b == nil || allBytesZero(baPrefix.b)
	}
	if baPrefix.nBits < ba.nBits {
		ba = ba.Slice(0, baPrefix.nBits)
	}
	switch {
	case baPrefix.b == nil:
		return ba.b == nil || allBytesZero(ba.b)
	case ba.b == nil:
		return allBytesZero(baPrefix.b)
	}

	return bytes.Equal(ba.b, baPrefix.b)
}

// HasSuffix reports whether the bit array ends with suffix.
func (ba *BitArray) HasSuffix(suffix BitArrayer) bool {
	var baSuffix *BitArray
	if suffix != nil {
		baSuffix = suffix.BitArray()
	}
	switch {
	case baSuffix.IsZero():
		return true
	case ba.IsZero(), ba.nBits < baSuffix.nBits:
		return false
	case ba.b == nil:
		return baSuffix.b == nil || allBytesZero(baSuffix.b)
	}
	if baSuffix.nBits < ba.nBits {
		ba = ba.Slice(ba.nBits-baSuffix.nBits, ba.nBits)
	}
	switch {
	case baSuffix.b == nil:
		return ba.b == nil || allBytesZero(ba.b)
	case ba.b == nil:
		return allBytesZero(baSuffix.b)
	}

	return bytes.Equal(ba.b, baSuffix.b)
}

// Index returns the index of the first instance of x in the bit array ba, or -1
// if x is not present in ba.
func (ba *BitArray) Index(x BitArrayer) int {
	var bax *BitArray
	if x != nil {
		bax = x.BitArray()
	}
	switch {
	case ba.Len() < bax.Len():
		return -1
	case bax.IsZero():
		return 0
	case ba.b == nil && bax.b == nil:
		return 0
	}
	ba8 := ba.bits8()
	bax8 := bax.bits8()

	return bytes.Index(ba8, bax8)
}

// LastIndex returns the index of the last instance of x in the bit array ba, or
// -1 if x is not present in ba.
func (ba *BitArray) LastIndex(x BitArrayer) int {
	var bax *BitArray
	if x != nil {
		bax = x.BitArray()
	}
	switch {
	case ba.Len() < bax.Len():
		return -1
	case bax.IsZero():
		return ba.Len()
	case ba.b == nil && bax.b == nil:
		return ba.nBits - bax.nBits
	}
	ba8 := ba.bits8()
	bax8 := bax.bits8()

	return bytes.LastIndex(ba8, bax8)
}

// AllIndex returns the indexes of the all instance of x in the bit array ba, or
// empty slice if x is not present in ba.
func (ba *BitArray) AllIndex(x BitArrayer) []int {
	var bax *BitArray
	if x != nil {
		bax = x.BitArray()
	}
	switch {
	case ba.Len() < bax.Len():
		return []int{}
	case bax.IsZero():
		idxs := make([]int, ba.Len()+1)
		for i := range idxs {
			idxs[i] = i
		}
		return idxs
	case ba.b == nil && bax.b == nil:
		idxs := make([]int, ba.nBits-bax.nBits+1)
		for i := range idxs {
			idxs[i] = i
		}
		return idxs
	}
	ba8 := ba.bits8()
	bax8 := bax.bits8()
	var idxs []int
	i := 0
	for i < ba.nBits-bax.nBits+1 {
		idx := bytes.Index(ba8[i:], bax8)
		if idx < 0 {
			break
		}
		idx += i
		idxs = append(idxs, idx)
		i = idx + 1
	}
	return idxs
}
