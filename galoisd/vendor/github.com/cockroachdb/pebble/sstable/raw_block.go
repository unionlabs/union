// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package sstable

import (
	"encoding/binary"
	"sort"
	"unsafe"

	"github.com/cockroachdb/pebble/internal/base"
)

type rawBlockWriter struct {
	blockWriter
}

func (w *rawBlockWriter) add(key InternalKey, value []byte) {
	w.curKey, w.prevKey = w.prevKey, w.curKey

	size := len(key.UserKey)
	if cap(w.curKey) < size {
		w.curKey = make([]byte, 0, size*2)
	}
	w.curKey = w.curKey[:size]
	copy(w.curKey, key.UserKey)

	w.storeWithOptionalValuePrefix(
		size, value, len(key.UserKey), false, 0, false)
}

// rawBlockIter is an iterator over a single block of data. Unlike blockIter,
// keys are stored in "raw" format (i.e. not as internal keys). Note that there
// is significant similarity between this code and the code in blockIter. Yet
// reducing duplication is difficult due to the blockIter being performance
// critical. rawBlockIter must only be used for blocks where the value is
// stored together with the key.
type rawBlockIter struct {
	cmp         Compare
	offset      int32
	nextOffset  int32
	restarts    int32
	numRestarts int32
	ptr         unsafe.Pointer
	data        []byte
	key, val    []byte
	ikey        InternalKey
	cached      []blockEntry
	cachedBuf   []byte
}

func newRawBlockIter(cmp Compare, block block) (*rawBlockIter, error) {
	i := &rawBlockIter{}
	return i, i.init(cmp, block)
}

func (i *rawBlockIter) init(cmp Compare, block block) error {
	numRestarts := int32(binary.LittleEndian.Uint32(block[len(block)-4:]))
	if numRestarts == 0 {
		return base.CorruptionErrorf("pebble/table: invalid table (block has no restart points)")
	}
	i.cmp = cmp
	i.restarts = int32(len(block)) - 4*(1+numRestarts)
	i.numRestarts = numRestarts
	i.ptr = unsafe.Pointer(&block[0])
	i.data = block
	if i.key == nil {
		i.key = make([]byte, 0, 256)
	} else {
		i.key = i.key[:0]
	}
	i.val = nil
	i.clearCache()
	return nil
}

func (i *rawBlockIter) readEntry() {
	ptr := unsafe.Pointer(uintptr(i.ptr) + uintptr(i.offset))
	shared, ptr := decodeVarint(ptr)
	unshared, ptr := decodeVarint(ptr)
	value, ptr := decodeVarint(ptr)
	i.key = append(i.key[:shared], getBytes(ptr, int(unshared))...)
	i.key = i.key[:len(i.key):len(i.key)]
	ptr = unsafe.Pointer(uintptr(ptr) + uintptr(unshared))
	i.val = getBytes(ptr, int(value))
	i.nextOffset = int32(uintptr(ptr)-uintptr(i.ptr)) + int32(value)
}

func (i *rawBlockIter) loadEntry() {
	i.readEntry()
	i.ikey.UserKey = i.key
}

func (i *rawBlockIter) clearCache() {
	i.cached = i.cached[:0]
	i.cachedBuf = i.cachedBuf[:0]
}

func (i *rawBlockIter) cacheEntry() {
	var valStart int32
	valSize := int32(len(i.val))
	if valSize > 0 {
		valStart = int32(uintptr(unsafe.Pointer(&i.val[0])) - uintptr(i.ptr))
	}

	i.cached = append(i.cached, blockEntry{
		offset:   i.offset,
		keyStart: int32(len(i.cachedBuf)),
		keyEnd:   int32(len(i.cachedBuf) + len(i.key)),
		valStart: valStart,
		valSize:  valSize,
	})
	i.cachedBuf = append(i.cachedBuf, i.key...)
}

// SeekGE implements internalIterator.SeekGE, as documented in the pebble
// package.
func (i *rawBlockIter) SeekGE(key []byte) bool {
	// Find the index of the smallest restart point whose key is > the key
	// sought; index will be numRestarts if there is no such restart point.
	i.offset = 0
	index := sort.Search(int(i.numRestarts), func(j int) bool {
		offset := int32(binary.LittleEndian.Uint32(i.data[int(i.restarts)+4*j:]))
		// For a restart point, there are 0 bytes shared with the previous key.
		// The varint encoding of 0 occupies 1 byte.
		ptr := unsafe.Pointer(uintptr(i.ptr) + uintptr(offset+1))
		// Decode the key at that restart point, and compare it to the key sought.
		v1, ptr := decodeVarint(ptr)
		_, ptr = decodeVarint(ptr)
		s := getBytes(ptr, int(v1))
		return i.cmp(key, s) < 0
	})

	// Since keys are strictly increasing, if index > 0 then the restart point at
	// index-1 will be the largest whose key is <= the key sought.  If index ==
	// 0, then all keys in this block are larger than the key sought, and offset
	// remains at zero.
	if index > 0 {
		i.offset = int32(binary.LittleEndian.Uint32(i.data[int(i.restarts)+4*(index-1):]))
	}
	i.loadEntry()

	// Iterate from that restart point to somewhere >= the key sought.
	for valid := i.Valid(); valid; valid = i.Next() {
		if i.cmp(key, i.key) <= 0 {
			break
		}
	}
	return i.Valid()
}

// First implements internalIterator.First, as documented in the pebble
// package.
func (i *rawBlockIter) First() bool {
	i.offset = 0
	i.loadEntry()
	return i.Valid()
}

// Last implements internalIterator.Last, as documented in the pebble package.
func (i *rawBlockIter) Last() bool {
	// Seek forward from the last restart point.
	i.offset = int32(binary.LittleEndian.Uint32(i.data[i.restarts+4*(i.numRestarts-1):]))

	i.readEntry()
	i.clearCache()
	i.cacheEntry()

	for i.nextOffset < i.restarts {
		i.offset = i.nextOffset
		i.readEntry()
		i.cacheEntry()
	}

	i.ikey.UserKey = i.key
	return i.Valid()
}

// Next implements internalIterator.Next, as documented in the pebble
// package.
func (i *rawBlockIter) Next() bool {
	i.offset = i.nextOffset
	if !i.Valid() {
		return false
	}
	i.loadEntry()
	return true
}

// Prev implements internalIterator.Prev, as documented in the pebble
// package.
func (i *rawBlockIter) Prev() bool {
	if n := len(i.cached) - 1; n > 0 && i.cached[n].offset == i.offset {
		i.nextOffset = i.offset
		e := &i.cached[n-1]
		i.offset = e.offset
		i.val = getBytes(unsafe.Pointer(uintptr(i.ptr)+uintptr(e.valStart)), int(e.valSize))
		i.ikey.UserKey = i.cachedBuf[e.keyStart:e.keyEnd]
		i.cached = i.cached[:n]
		return true
	}

	if i.offset == 0 {
		i.offset = -1
		i.nextOffset = 0
		return false
	}

	targetOffset := i.offset
	index := sort.Search(int(i.numRestarts), func(j int) bool {
		offset := int32(binary.LittleEndian.Uint32(i.data[int(i.restarts)+4*j:]))
		return offset >= targetOffset
	})
	i.offset = 0
	if index > 0 {
		i.offset = int32(binary.LittleEndian.Uint32(i.data[int(i.restarts)+4*(index-1):]))
	}

	i.readEntry()
	i.clearCache()
	i.cacheEntry()

	for i.nextOffset < targetOffset {
		i.offset = i.nextOffset
		i.readEntry()
		i.cacheEntry()
	}

	i.ikey.UserKey = i.key
	return true
}

// Key implements internalIterator.Key, as documented in the pebble package.
func (i *rawBlockIter) Key() InternalKey {
	return i.ikey
}

// Value implements internalIterator.Value, as documented in the pebble
// package.
func (i *rawBlockIter) Value() []byte {
	return i.val
}

// Valid implements internalIterator.Valid, as documented in the pebble
// package.
func (i *rawBlockIter) Valid() bool {
	return i.offset >= 0 && i.offset < i.restarts
}

// Error implements internalIterator.Error, as documented in the pebble
// package.
func (i *rawBlockIter) Error() error {
	return nil
}

// Close implements internalIterator.Close, as documented in the pebble
// package.
func (i *rawBlockIter) Close() error {
	i.val = nil
	return nil
}
