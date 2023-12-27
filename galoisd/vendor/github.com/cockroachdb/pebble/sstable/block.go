// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package sstable

import (
	"encoding/binary"
	"unsafe"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/invariants"
	"github.com/cockroachdb/pebble/internal/keyspan"
	"github.com/cockroachdb/pebble/internal/manual"
	"github.com/cockroachdb/pebble/internal/rangedel"
	"github.com/cockroachdb/pebble/internal/rangekey"
)

func uvarintLen(v uint32) int {
	i := 0
	for v >= 0x80 {
		v >>= 7
		i++
	}
	return i + 1
}

type blockWriter struct {
	restartInterval int
	nEntries        int
	nextRestart     int
	buf             []byte
	// For datablocks in TableFormatPebblev3, we steal the most significant bit
	// in restarts for encoding setHasSameKeyPrefixSinceLastRestart. This leaves
	// us with 31 bits, which is more than enough (no one needs > 2GB blocks).
	// Typically, restarts occur every 16 keys, and by storing this bit with the
	// restart, we can optimize for the case where a user wants to skip to the
	// next prefix which happens to be in the same data block, but is > 16 keys
	// away. We have seen production situations with 100+ versions per MVCC key
	// (which share the same prefix). Additionally, for such writers, the prefix
	// compression of the key, that shares the key with the preceding key, is
	// limited to the prefix part of the preceding key -- this ensures that when
	// doing NPrefix (see blockIter) we don't need to assemble the full key
	// for each step since by limiting the length of the shared key we are
	// ensuring that any of the keys with the same prefix can be used to
	// assemble the full key when the prefix does change.
	restarts []uint32
	// Do not read curKey directly from outside blockWriter since it can have
	// the InternalKeyKindSSTableInternalObsoleteBit set. Use getCurKey() or
	// getCurUserKey() instead.
	curKey []byte
	// curValue excludes the optional prefix provided to
	// storeWithOptionalValuePrefix.
	curValue []byte
	prevKey  []byte
	tmp      [4]byte
	// We don't know the state of the sets that were at the end of the previous
	// block, so this is initially 0. It may be true for the second and later
	// restarts in a block. Not having inter-block information is fine since we
	// will optimize by stepping through restarts only within the same block.
	// Note that the first restart is the first key in the block.
	setHasSameKeyPrefixSinceLastRestart bool
}

func (w *blockWriter) clear() {
	*w = blockWriter{
		buf:      w.buf[:0],
		restarts: w.restarts[:0],
		curKey:   w.curKey[:0],
		curValue: w.curValue[:0],
		prevKey:  w.prevKey[:0],
	}
}

// MaximumBlockSize is an extremely generous maximum block size of 256MiB. We
// explicitly place this limit to reserve a few bits in the restart for
// internal use.
const MaximumBlockSize = 1 << 28
const setHasSameKeyPrefixRestartMask uint32 = 1 << 31
const restartMaskLittleEndianHighByteWithoutSetHasSamePrefix byte = 0b0111_1111
const restartMaskLittleEndianHighByteOnlySetHasSamePrefix byte = 0b1000_0000

func (w *blockWriter) getCurKey() InternalKey {
	k := base.DecodeInternalKey(w.curKey)
	k.Trailer = k.Trailer & trailerObsoleteMask
	return k
}

func (w *blockWriter) getCurUserKey() []byte {
	n := len(w.curKey) - base.InternalTrailerLen
	if n < 0 {
		panic(errors.AssertionFailedf("corrupt key in blockWriter buffer"))
	}
	return w.curKey[:n:n]
}

// If !addValuePrefix, the valuePrefix is ignored.
func (w *blockWriter) storeWithOptionalValuePrefix(
	keySize int,
	value []byte,
	maxSharedKeyLen int,
	addValuePrefix bool,
	valuePrefix valuePrefix,
	setHasSameKeyPrefix bool,
) {
	shared := 0
	if !setHasSameKeyPrefix {
		w.setHasSameKeyPrefixSinceLastRestart = false
	}
	if w.nEntries == w.nextRestart {
		w.nextRestart = w.nEntries + w.restartInterval
		restart := uint32(len(w.buf))
		if w.setHasSameKeyPrefixSinceLastRestart {
			restart = restart | setHasSameKeyPrefixRestartMask
		}
		w.setHasSameKeyPrefixSinceLastRestart = true
		w.restarts = append(w.restarts, restart)
	} else {
		// TODO(peter): Manually inlined version of base.SharedPrefixLen(). This
		// is 3% faster on BenchmarkWriter on go1.16. Remove if future versions
		// show this to not be a performance win. For now, functions that use of
		// unsafe cannot be inlined.
		n := maxSharedKeyLen
		if n > len(w.prevKey) {
			n = len(w.prevKey)
		}
		asUint64 := func(b []byte, i int) uint64 {
			return binary.LittleEndian.Uint64(b[i:])
		}
		for shared < n-7 && asUint64(w.curKey, shared) == asUint64(w.prevKey, shared) {
			shared += 8
		}
		for shared < n && w.curKey[shared] == w.prevKey[shared] {
			shared++
		}
	}

	lenValuePlusOptionalPrefix := len(value)
	if addValuePrefix {
		lenValuePlusOptionalPrefix++
	}
	needed := 3*binary.MaxVarintLen32 + len(w.curKey[shared:]) + lenValuePlusOptionalPrefix
	n := len(w.buf)
	if cap(w.buf) < n+needed {
		newCap := 2 * cap(w.buf)
		if newCap == 0 {
			newCap = 1024
		}
		for newCap < n+needed {
			newCap *= 2
		}
		newBuf := make([]byte, n, newCap)
		copy(newBuf, w.buf)
		w.buf = newBuf
	}
	w.buf = w.buf[:n+needed]

	// TODO(peter): Manually inlined versions of binary.PutUvarint(). This is 15%
	// faster on BenchmarkWriter on go1.13. Remove if go1.14 or future versions
	// show this to not be a performance win.
	{
		x := uint32(shared)
		for x >= 0x80 {
			w.buf[n] = byte(x) | 0x80
			x >>= 7
			n++
		}
		w.buf[n] = byte(x)
		n++
	}

	{
		x := uint32(keySize - shared)
		for x >= 0x80 {
			w.buf[n] = byte(x) | 0x80
			x >>= 7
			n++
		}
		w.buf[n] = byte(x)
		n++
	}

	{
		x := uint32(lenValuePlusOptionalPrefix)
		for x >= 0x80 {
			w.buf[n] = byte(x) | 0x80
			x >>= 7
			n++
		}
		w.buf[n] = byte(x)
		n++
	}

	n += copy(w.buf[n:], w.curKey[shared:])
	if addValuePrefix {
		w.buf[n : n+1][0] = byte(valuePrefix)
		n++
	}
	n += copy(w.buf[n:], value)
	w.buf = w.buf[:n]

	w.curValue = w.buf[n-len(value):]

	w.nEntries++
}

func (w *blockWriter) add(key InternalKey, value []byte) {
	w.addWithOptionalValuePrefix(
		key, false, value, len(key.UserKey), false, 0, false)
}

// Callers that always set addValuePrefix to false should use add() instead.
//
// isObsolete indicates whether this key-value pair is obsolete in this
// sstable (only applicable when writing data blocks) -- see the comment in
// table.go and the longer one in format.go. addValuePrefix adds a 1 byte
// prefix to the value, specified in valuePrefix -- this is used for data
// blocks in TableFormatPebblev3 onwards for SETs (see the comment in
// format.go, with more details in value_block.go). setHasSameKeyPrefix is
// also used in TableFormatPebblev3 onwards for SETs.
func (w *blockWriter) addWithOptionalValuePrefix(
	key InternalKey,
	isObsolete bool,
	value []byte,
	maxSharedKeyLen int,
	addValuePrefix bool,
	valuePrefix valuePrefix,
	setHasSameKeyPrefix bool,
) {
	w.curKey, w.prevKey = w.prevKey, w.curKey

	size := key.Size()
	if cap(w.curKey) < size {
		w.curKey = make([]byte, 0, size*2)
	}
	w.curKey = w.curKey[:size]
	if isObsolete {
		key.Trailer = key.Trailer | trailerObsoleteBit
	}
	key.Encode(w.curKey)

	w.storeWithOptionalValuePrefix(
		size, value, maxSharedKeyLen, addValuePrefix, valuePrefix, setHasSameKeyPrefix)
}

func (w *blockWriter) finish() []byte {
	// Write the restart points to the buffer.
	if w.nEntries == 0 {
		// Every block must have at least one restart point.
		if cap(w.restarts) > 0 {
			w.restarts = w.restarts[:1]
			w.restarts[0] = 0
		} else {
			w.restarts = append(w.restarts, 0)
		}
	}
	tmp4 := w.tmp[:4]
	for _, x := range w.restarts {
		binary.LittleEndian.PutUint32(tmp4, x)
		w.buf = append(w.buf, tmp4...)
	}
	binary.LittleEndian.PutUint32(tmp4, uint32(len(w.restarts)))
	w.buf = append(w.buf, tmp4...)
	result := w.buf

	// Reset the block state.
	w.nEntries = 0
	w.nextRestart = 0
	w.buf = w.buf[:0]
	w.restarts = w.restarts[:0]
	return result
}

// emptyBlockSize holds the size of an empty block. Every block ends
// in a uint32 trailer encoding the number of restart points within the
// block.
const emptyBlockSize = 4

func (w *blockWriter) estimatedSize() int {
	return len(w.buf) + 4*len(w.restarts) + emptyBlockSize
}

type blockEntry struct {
	offset   int32
	keyStart int32
	keyEnd   int32
	valStart int32
	valSize  int32
}

// blockIter is an iterator over a single block of data.
//
// A blockIter provides an additional guarantee around key stability when a
// block has a restart interval of 1 (i.e. when there is no prefix
// compression). Key stability refers to whether the InternalKey.UserKey bytes
// returned by a positioning call will remain stable after a subsequent
// positioning call. The normal case is that a positioning call will invalidate
// any previously returned InternalKey.UserKey. If a block has a restart
// interval of 1 (no prefix compression), blockIter guarantees that
// InternalKey.UserKey will point to the key as stored in the block itself
// which will remain valid until the blockIter is closed. The key stability
// guarantee is used by the range tombstone and range key code, which knows that
// the respective blocks are always encoded with a restart interval of 1. This
// per-block key stability guarantee is sufficient for range tombstones and
// range deletes as they are always encoded in a single block.
//
// A blockIter also provides a value stability guarantee for range deletions and
// range keys since there is only a single range deletion and range key block
// per sstable and the blockIter will not release the bytes for the block until
// it is closed.
//
// Note on why blockIter knows about lazyValueHandling:
//
// blockIter's positioning functions (that return a LazyValue), are too
// complex to inline even prior to lazyValueHandling. blockIter.Next and
// blockIter.First were by far the cheapest and had costs 195 and 180
// respectively, which exceeds the budget of 80. We initially tried to keep
// the lazyValueHandling logic out of blockIter by wrapping it with a
// lazyValueDataBlockIter. singleLevelIter and twoLevelIter would use this
// wrapped iter. The functions in lazyValueDataBlockIter were simple, in that
// they called the corresponding blockIter func and then decided whether the
// value was in fact in-place (so return immediately) or needed further
// handling. But these also turned out too costly for mid-stack inlining since
// simple calls like the following have a high cost that is barely under the
// budget of 80
//
//	k, v := i.data.SeekGE(key, flags)  // cost 74
//	k, v := i.data.Next()              // cost 72
//
// We have 2 options for minimizing performance regressions:
//   - Include the lazyValueHandling logic in the already non-inlineable
//     blockIter functions: Since most of the time is spent in data block iters,
//     it is acceptable to take the small hit of unnecessary branching (which
//     hopefully branch prediction will predict correctly) for other kinds of
//     blocks.
//   - Duplicate the logic of singleLevelIterator and twoLevelIterator for the
//     v3 sstable and only use the aforementioned lazyValueDataBlockIter for a
//     v3 sstable. We would want to manage these copies via code generation.
//
// We have picked the first option here.
type blockIter struct {
	cmp Compare
	// offset is the byte index that marks where the current key/value is
	// encoded in the block.
	offset int32
	// nextOffset is the byte index where the next key/value is encoded in the
	// block.
	nextOffset int32
	// A "restart point" in a block is a point where the full key is encoded,
	// instead of just having a suffix of the key encoded. See readEntry() for
	// how prefix compression of keys works. Keys in between two restart points
	// only have a suffix encoded in the block. When restart interval is 1, no
	// prefix compression of keys happens. This is the case with range tombstone
	// blocks.
	//
	// All restart offsets are listed in increasing order in
	// i.ptr[i.restarts:len(block)-4], while numRestarts is encoded in the last
	// 4 bytes of the block as a uint32 (i.ptr[len(block)-4:]). i.restarts can
	// therefore be seen as the point where data in the block ends, and a list
	// of offsets of all restart points begins.
	restarts int32
	// Number of restart points in this block. Encoded at the end of the block
	// as a uint32.
	numRestarts  int32
	globalSeqNum uint64
	ptr          unsafe.Pointer
	data         []byte
	// key contains the raw key the iterator is currently pointed at. This may
	// point directly to data stored in the block (for a key which has no prefix
	// compression), to fullKey (for a prefix compressed key), or to a slice of
	// data stored in cachedBuf (during reverse iteration).
	key []byte
	// fullKey is a buffer used for key prefix decompression.
	fullKey []byte
	// val contains the value the iterator is currently pointed at. If non-nil,
	// this points to a slice of the block data.
	val []byte
	// lazyValue is val turned into a LazyValue, whenever a positioning method
	// returns a non-nil key-value pair.
	lazyValue base.LazyValue
	// ikey contains the decoded InternalKey the iterator is currently pointed
	// at. Note that the memory backing ikey.UserKey is either data stored
	// directly in the block, fullKey, or cachedBuf. The key stability guarantee
	// for blocks built with a restart interval of 1 is achieved by having
	// ikey.UserKey always point to data stored directly in the block.
	ikey InternalKey
	// cached and cachedBuf are used during reverse iteration. They are needed
	// because we can't perform prefix decoding in reverse, only in the forward
	// direction. In order to iterate in reverse, we decode and cache the entries
	// between two restart points.
	//
	// Note that cached[len(cached)-1] contains the previous entry to the one the
	// blockIter is currently pointed at. As usual, nextOffset will contain the
	// offset of the next entry. During reverse iteration, nextOffset will be
	// updated to point to offset, and we'll set the blockIter to point at the
	// entry cached[len(cached)-1]. See Prev() for more details.
	//
	// For a block encoded with a restart interval of 1, cached and cachedBuf
	// will not be used as there are no prefix compressed entries between the
	// restart points.
	cached    []blockEntry
	cachedBuf []byte
	handle    bufferHandle
	// for block iteration for already loaded blocks.
	firstUserKey      []byte
	lazyValueHandling struct {
		vbr            *valueBlockReader
		hasValuePrefix bool
	}
	hideObsoletePoints bool
}

// blockIter implements the base.InternalIterator interface.
var _ base.InternalIterator = (*blockIter)(nil)

func newBlockIter(cmp Compare, block block) (*blockIter, error) {
	i := &blockIter{}
	return i, i.init(cmp, block, 0, false)
}

func (i *blockIter) String() string {
	return "block"
}

func (i *blockIter) init(
	cmp Compare, block block, globalSeqNum uint64, hideObsoletePoints bool,
) error {
	numRestarts := int32(binary.LittleEndian.Uint32(block[len(block)-4:]))
	if numRestarts == 0 {
		return base.CorruptionErrorf("pebble/table: invalid table (block has no restart points)")
	}
	i.cmp = cmp
	i.restarts = int32(len(block)) - 4*(1+numRestarts)
	i.numRestarts = numRestarts
	i.globalSeqNum = globalSeqNum
	i.ptr = unsafe.Pointer(&block[0])
	i.data = block
	i.fullKey = i.fullKey[:0]
	i.val = nil
	i.hideObsoletePoints = hideObsoletePoints
	i.clearCache()
	if i.restarts > 0 {
		if err := i.readFirstKey(); err != nil {
			return err
		}
	} else {
		// Block is empty.
		i.firstUserKey = nil
	}
	return nil
}

// NB: two cases of hideObsoletePoints:
//   - Local sstable iteration: globalSeqNum will be set iff the sstable was
//     ingested.
//   - Foreign sstable iteration: globalSeqNum is always set.
func (i *blockIter) initHandle(
	cmp Compare, block bufferHandle, globalSeqNum uint64, hideObsoletePoints bool,
) error {
	i.handle.Release()
	i.handle = block
	return i.init(cmp, block.Get(), globalSeqNum, hideObsoletePoints)
}

func (i *blockIter) invalidate() {
	i.clearCache()
	i.offset = 0
	i.nextOffset = 0
	i.restarts = 0
	i.numRestarts = 0
	i.data = nil
}

// isDataInvalidated returns true when the blockIter has been invalidated
// using an invalidate call. NB: this is different from blockIter.Valid
// which is part of the InternalIterator implementation.
func (i *blockIter) isDataInvalidated() bool {
	return i.data == nil
}

func (i *blockIter) resetForReuse() blockIter {
	return blockIter{
		fullKey:   i.fullKey[:0],
		cached:    i.cached[:0],
		cachedBuf: i.cachedBuf[:0],
		data:      nil,
	}
}

func (i *blockIter) readEntry() {
	ptr := unsafe.Pointer(uintptr(i.ptr) + uintptr(i.offset))

	// This is an ugly performance hack. Reading entries from blocks is one of
	// the inner-most routines and decoding the 3 varints per-entry takes
	// significant time. Neither go1.11 or go1.12 will inline decodeVarint for
	// us, so we do it manually. This provides a 10-15% performance improvement
	// on blockIter benchmarks on both go1.11 and go1.12.
	//
	// TODO(peter): remove this hack if go:inline is ever supported.

	var shared uint32
	if a := *((*uint8)(ptr)); a < 128 {
		shared = uint32(a)
		ptr = unsafe.Pointer(uintptr(ptr) + 1)
	} else if a, b := a&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 1))); b < 128 {
		shared = uint32(b)<<7 | uint32(a)
		ptr = unsafe.Pointer(uintptr(ptr) + 2)
	} else if b, c := b&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 2))); c < 128 {
		shared = uint32(c)<<14 | uint32(b)<<7 | uint32(a)
		ptr = unsafe.Pointer(uintptr(ptr) + 3)
	} else if c, d := c&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 3))); d < 128 {
		shared = uint32(d)<<21 | uint32(c)<<14 | uint32(b)<<7 | uint32(a)
		ptr = unsafe.Pointer(uintptr(ptr) + 4)
	} else {
		d, e := d&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 4)))
		shared = uint32(e)<<28 | uint32(d)<<21 | uint32(c)<<14 | uint32(b)<<7 | uint32(a)
		ptr = unsafe.Pointer(uintptr(ptr) + 5)
	}

	var unshared uint32
	if a := *((*uint8)(ptr)); a < 128 {
		unshared = uint32(a)
		ptr = unsafe.Pointer(uintptr(ptr) + 1)
	} else if a, b := a&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 1))); b < 128 {
		unshared = uint32(b)<<7 | uint32(a)
		ptr = unsafe.Pointer(uintptr(ptr) + 2)
	} else if b, c := b&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 2))); c < 128 {
		unshared = uint32(c)<<14 | uint32(b)<<7 | uint32(a)
		ptr = unsafe.Pointer(uintptr(ptr) + 3)
	} else if c, d := c&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 3))); d < 128 {
		unshared = uint32(d)<<21 | uint32(c)<<14 | uint32(b)<<7 | uint32(a)
		ptr = unsafe.Pointer(uintptr(ptr) + 4)
	} else {
		d, e := d&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 4)))
		unshared = uint32(e)<<28 | uint32(d)<<21 | uint32(c)<<14 | uint32(b)<<7 | uint32(a)
		ptr = unsafe.Pointer(uintptr(ptr) + 5)
	}

	var value uint32
	if a := *((*uint8)(ptr)); a < 128 {
		value = uint32(a)
		ptr = unsafe.Pointer(uintptr(ptr) + 1)
	} else if a, b := a&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 1))); b < 128 {
		value = uint32(b)<<7 | uint32(a)
		ptr = unsafe.Pointer(uintptr(ptr) + 2)
	} else if b, c := b&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 2))); c < 128 {
		value = uint32(c)<<14 | uint32(b)<<7 | uint32(a)
		ptr = unsafe.Pointer(uintptr(ptr) + 3)
	} else if c, d := c&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 3))); d < 128 {
		value = uint32(d)<<21 | uint32(c)<<14 | uint32(b)<<7 | uint32(a)
		ptr = unsafe.Pointer(uintptr(ptr) + 4)
	} else {
		d, e := d&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 4)))
		value = uint32(e)<<28 | uint32(d)<<21 | uint32(c)<<14 | uint32(b)<<7 | uint32(a)
		ptr = unsafe.Pointer(uintptr(ptr) + 5)
	}

	unsharedKey := getBytes(ptr, int(unshared))
	// TODO(sumeer): move this into the else block below.
	i.fullKey = append(i.fullKey[:shared], unsharedKey...)
	if shared == 0 {
		// Provide stability for the key across positioning calls if the key
		// doesn't share a prefix with the previous key. This removes requiring the
		// key to be copied if the caller knows the block has a restart interval of
		// 1. An important example of this is range-del blocks.
		i.key = unsharedKey
	} else {
		i.key = i.fullKey
	}
	ptr = unsafe.Pointer(uintptr(ptr) + uintptr(unshared))
	i.val = getBytes(ptr, int(value))
	i.nextOffset = int32(uintptr(ptr)-uintptr(i.ptr)) + int32(value)
}

func (i *blockIter) readFirstKey() error {
	ptr := i.ptr

	// This is an ugly performance hack. Reading entries from blocks is one of
	// the inner-most routines and decoding the 3 varints per-entry takes
	// significant time. Neither go1.11 or go1.12 will inline decodeVarint for
	// us, so we do it manually. This provides a 10-15% performance improvement
	// on blockIter benchmarks on both go1.11 and go1.12.
	//
	// TODO(peter): remove this hack if go:inline is ever supported.

	if shared := *((*uint8)(ptr)); shared == 0 {
		ptr = unsafe.Pointer(uintptr(ptr) + 1)
	} else {
		// The shared length is != 0, which is invalid.
		panic("first key in block must have zero shared length")
	}

	var unshared uint32
	if a := *((*uint8)(ptr)); a < 128 {
		unshared = uint32(a)
		ptr = unsafe.Pointer(uintptr(ptr) + 1)
	} else if a, b := a&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 1))); b < 128 {
		unshared = uint32(b)<<7 | uint32(a)
		ptr = unsafe.Pointer(uintptr(ptr) + 2)
	} else if b, c := b&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 2))); c < 128 {
		unshared = uint32(c)<<14 | uint32(b)<<7 | uint32(a)
		ptr = unsafe.Pointer(uintptr(ptr) + 3)
	} else if c, d := c&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 3))); d < 128 {
		unshared = uint32(d)<<21 | uint32(c)<<14 | uint32(b)<<7 | uint32(a)
		ptr = unsafe.Pointer(uintptr(ptr) + 4)
	} else {
		d, e := d&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 4)))
		unshared = uint32(e)<<28 | uint32(d)<<21 | uint32(c)<<14 | uint32(b)<<7 | uint32(a)
		ptr = unsafe.Pointer(uintptr(ptr) + 5)
	}

	// Skip the value length.
	if a := *((*uint8)(ptr)); a < 128 {
		ptr = unsafe.Pointer(uintptr(ptr) + 1)
	} else if a := *((*uint8)(unsafe.Pointer(uintptr(ptr) + 1))); a < 128 {
		ptr = unsafe.Pointer(uintptr(ptr) + 2)
	} else if a := *((*uint8)(unsafe.Pointer(uintptr(ptr) + 2))); a < 128 {
		ptr = unsafe.Pointer(uintptr(ptr) + 3)
	} else if a := *((*uint8)(unsafe.Pointer(uintptr(ptr) + 3))); a < 128 {
		ptr = unsafe.Pointer(uintptr(ptr) + 4)
	} else {
		ptr = unsafe.Pointer(uintptr(ptr) + 5)
	}

	firstKey := getBytes(ptr, int(unshared))
	// Manually inlining base.DecodeInternalKey provides a 5-10% speedup on
	// BlockIter benchmarks.
	if n := len(firstKey) - 8; n >= 0 {
		i.firstUserKey = firstKey[:n:n]
	} else {
		i.firstUserKey = nil
		return base.CorruptionErrorf("pebble/table: invalid firstKey in block")
	}
	return nil
}

// The sstable internal obsolete bit is set when writing a block and unset by
// blockIter, so no code outside block writing/reading code ever sees it.
const trailerObsoleteBit = uint64(base.InternalKeyKindSSTableInternalObsoleteBit)
const trailerObsoleteMask = (InternalKeySeqNumMax << 8) | uint64(base.InternalKeyKindSSTableInternalObsoleteMask)

func (i *blockIter) decodeInternalKey(key []byte) (hiddenPoint bool) {
	// Manually inlining base.DecodeInternalKey provides a 5-10% speedup on
	// BlockIter benchmarks.
	if n := len(key) - 8; n >= 0 {
		trailer := binary.LittleEndian.Uint64(key[n:])
		hiddenPoint = i.hideObsoletePoints &&
			(trailer&trailerObsoleteBit != 0)
		i.ikey.Trailer = trailer & trailerObsoleteMask
		i.ikey.UserKey = key[:n:n]
		if i.globalSeqNum != 0 {
			i.ikey.SetSeqNum(i.globalSeqNum)
		}
	} else {
		i.ikey.Trailer = uint64(InternalKeyKindInvalid)
		i.ikey.UserKey = nil
	}
	return hiddenPoint
}

func (i *blockIter) clearCache() {
	i.cached = i.cached[:0]
	i.cachedBuf = i.cachedBuf[:0]
}

func (i *blockIter) cacheEntry() {
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

func (i *blockIter) getFirstUserKey() []byte {
	return i.firstUserKey
}

// SeekGE implements internalIterator.SeekGE, as documented in the pebble
// package.
func (i *blockIter) SeekGE(key []byte, flags base.SeekGEFlags) (*InternalKey, base.LazyValue) {
	if invariants.Enabled && i.isDataInvalidated() {
		panic(errors.AssertionFailedf("invalidated blockIter used"))
	}

	i.clearCache()
	// Find the index of the smallest restart point whose key is > the key
	// sought; index will be numRestarts if there is no such restart point.
	i.offset = 0
	var index int32

	{
		// NB: manually inlined sort.Seach is ~5% faster.
		//
		// Define f(-1) == false and f(n) == true.
		// Invariant: f(index-1) == false, f(upper) == true.
		upper := i.numRestarts
		for index < upper {
			h := int32(uint(index+upper) >> 1) // avoid overflow when computing h
			// index ≤ h < upper
			offset := decodeRestart(i.data[i.restarts+4*h:])
			// For a restart point, there are 0 bytes shared with the previous key.
			// The varint encoding of 0 occupies 1 byte.
			ptr := unsafe.Pointer(uintptr(i.ptr) + uintptr(offset+1))

			// Decode the key at that restart point, and compare it to the key
			// sought. See the comment in readEntry for why we manually inline the
			// varint decoding.
			var v1 uint32
			if a := *((*uint8)(ptr)); a < 128 {
				v1 = uint32(a)
				ptr = unsafe.Pointer(uintptr(ptr) + 1)
			} else if a, b := a&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 1))); b < 128 {
				v1 = uint32(b)<<7 | uint32(a)
				ptr = unsafe.Pointer(uintptr(ptr) + 2)
			} else if b, c := b&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 2))); c < 128 {
				v1 = uint32(c)<<14 | uint32(b)<<7 | uint32(a)
				ptr = unsafe.Pointer(uintptr(ptr) + 3)
			} else if c, d := c&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 3))); d < 128 {
				v1 = uint32(d)<<21 | uint32(c)<<14 | uint32(b)<<7 | uint32(a)
				ptr = unsafe.Pointer(uintptr(ptr) + 4)
			} else {
				d, e := d&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 4)))
				v1 = uint32(e)<<28 | uint32(d)<<21 | uint32(c)<<14 | uint32(b)<<7 | uint32(a)
				ptr = unsafe.Pointer(uintptr(ptr) + 5)
			}

			if *((*uint8)(ptr)) < 128 {
				ptr = unsafe.Pointer(uintptr(ptr) + 1)
			} else if *((*uint8)(unsafe.Pointer(uintptr(ptr) + 1))) < 128 {
				ptr = unsafe.Pointer(uintptr(ptr) + 2)
			} else if *((*uint8)(unsafe.Pointer(uintptr(ptr) + 2))) < 128 {
				ptr = unsafe.Pointer(uintptr(ptr) + 3)
			} else if *((*uint8)(unsafe.Pointer(uintptr(ptr) + 3))) < 128 {
				ptr = unsafe.Pointer(uintptr(ptr) + 4)
			} else {
				ptr = unsafe.Pointer(uintptr(ptr) + 5)
			}

			// Manually inlining part of base.DecodeInternalKey provides a 5-10%
			// speedup on BlockIter benchmarks.
			s := getBytes(ptr, int(v1))
			var k []byte
			if n := len(s) - 8; n >= 0 {
				k = s[:n:n]
			}
			// Else k is invalid, and left as nil

			if i.cmp(key, k) > 0 {
				// The search key is greater than the user key at this restart point.
				// Search beyond this restart point, since we are trying to find the
				// first restart point with a user key >= the search key.
				index = h + 1 // preserves f(i-1) == false
			} else {
				// k >= search key, so prune everything after index (since index
				// satisfies the property we are looking for).
				upper = h // preserves f(j) == true
			}
		}
		// index == upper, f(index-1) == false, and f(upper) (= f(index)) == true
		// => answer is index.
	}

	// index is the first restart point with key >= search key. Define the keys
	// between a restart point and the next restart point as belonging to that
	// restart point.
	//
	// Since keys are strictly increasing, if index > 0 then the restart point
	// at index-1 will be the first one that has some keys belonging to it that
	// could be equal to the search key.  If index == 0, then all keys in this
	// block are larger than the key sought, and offset remains at zero.
	if index > 0 {
		i.offset = decodeRestart(i.data[i.restarts+4*(index-1):])
	}
	i.readEntry()
	hiddenPoint := i.decodeInternalKey(i.key)

	// Iterate from that restart point to somewhere >= the key sought.
	if !i.valid() {
		return nil, base.LazyValue{}
	}
	if !hiddenPoint && i.cmp(i.ikey.UserKey, key) >= 0 {
		// Initialize i.lazyValue
		if !i.lazyValueHandling.hasValuePrefix ||
			base.TrailerKind(i.ikey.Trailer) != InternalKeyKindSet {
			i.lazyValue = base.MakeInPlaceValue(i.val)
		} else if i.lazyValueHandling.vbr == nil || !isValueHandle(valuePrefix(i.val[0])) {
			i.lazyValue = base.MakeInPlaceValue(i.val[1:])
		} else {
			i.lazyValue = i.lazyValueHandling.vbr.getLazyValueForPrefixAndValueHandle(i.val)
		}
		return &i.ikey, i.lazyValue
	}
	for i.Next(); i.valid(); i.Next() {
		if i.cmp(i.ikey.UserKey, key) >= 0 {
			// i.Next() has already initialized i.lazyValue.
			return &i.ikey, i.lazyValue
		}
	}
	return nil, base.LazyValue{}
}

// SeekPrefixGE implements internalIterator.SeekPrefixGE, as documented in the
// pebble package.
func (i *blockIter) SeekPrefixGE(
	prefix, key []byte, flags base.SeekGEFlags,
) (*base.InternalKey, base.LazyValue) {
	// This should never be called as prefix iteration is handled by sstable.Iterator.
	panic("pebble: SeekPrefixGE unimplemented")
}

// SeekLT implements internalIterator.SeekLT, as documented in the pebble
// package.
func (i *blockIter) SeekLT(key []byte, flags base.SeekLTFlags) (*InternalKey, base.LazyValue) {
	if invariants.Enabled && i.isDataInvalidated() {
		panic(errors.AssertionFailedf("invalidated blockIter used"))
	}

	i.clearCache()
	// Find the index of the smallest restart point whose key is >= the key
	// sought; index will be numRestarts if there is no such restart point.
	i.offset = 0
	var index int32

	{
		// NB: manually inlined sort.Search is ~5% faster.
		//
		// Define f(-1) == false and f(n) == true.
		// Invariant: f(index-1) == false, f(upper) == true.
		upper := i.numRestarts
		for index < upper {
			h := int32(uint(index+upper) >> 1) // avoid overflow when computing h
			// index ≤ h < upper
			offset := decodeRestart(i.data[i.restarts+4*h:])
			// For a restart point, there are 0 bytes shared with the previous key.
			// The varint encoding of 0 occupies 1 byte.
			ptr := unsafe.Pointer(uintptr(i.ptr) + uintptr(offset+1))

			// Decode the key at that restart point, and compare it to the key
			// sought. See the comment in readEntry for why we manually inline the
			// varint decoding.
			var v1 uint32
			if a := *((*uint8)(ptr)); a < 128 {
				v1 = uint32(a)
				ptr = unsafe.Pointer(uintptr(ptr) + 1)
			} else if a, b := a&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 1))); b < 128 {
				v1 = uint32(b)<<7 | uint32(a)
				ptr = unsafe.Pointer(uintptr(ptr) + 2)
			} else if b, c := b&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 2))); c < 128 {
				v1 = uint32(c)<<14 | uint32(b)<<7 | uint32(a)
				ptr = unsafe.Pointer(uintptr(ptr) + 3)
			} else if c, d := c&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 3))); d < 128 {
				v1 = uint32(d)<<21 | uint32(c)<<14 | uint32(b)<<7 | uint32(a)
				ptr = unsafe.Pointer(uintptr(ptr) + 4)
			} else {
				d, e := d&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 4)))
				v1 = uint32(e)<<28 | uint32(d)<<21 | uint32(c)<<14 | uint32(b)<<7 | uint32(a)
				ptr = unsafe.Pointer(uintptr(ptr) + 5)
			}

			if *((*uint8)(ptr)) < 128 {
				ptr = unsafe.Pointer(uintptr(ptr) + 1)
			} else if *((*uint8)(unsafe.Pointer(uintptr(ptr) + 1))) < 128 {
				ptr = unsafe.Pointer(uintptr(ptr) + 2)
			} else if *((*uint8)(unsafe.Pointer(uintptr(ptr) + 2))) < 128 {
				ptr = unsafe.Pointer(uintptr(ptr) + 3)
			} else if *((*uint8)(unsafe.Pointer(uintptr(ptr) + 3))) < 128 {
				ptr = unsafe.Pointer(uintptr(ptr) + 4)
			} else {
				ptr = unsafe.Pointer(uintptr(ptr) + 5)
			}

			// Manually inlining part of base.DecodeInternalKey provides a 5-10%
			// speedup on BlockIter benchmarks.
			s := getBytes(ptr, int(v1))
			var k []byte
			if n := len(s) - 8; n >= 0 {
				k = s[:n:n]
			}
			// Else k is invalid, and left as nil

			if i.cmp(key, k) > 0 {
				// The search key is greater than the user key at this restart point.
				// Search beyond this restart point, since we are trying to find the
				// first restart point with a user key >= the search key.
				index = h + 1 // preserves f(i-1) == false
			} else {
				// k >= search key, so prune everything after index (since index
				// satisfies the property we are looking for).
				upper = h // preserves f(j) == true
			}
		}
		// index == upper, f(index-1) == false, and f(upper) (= f(index)) == true
		// => answer is index.
	}

	// index is the first restart point with key >= search key. Define the keys
	// between a restart point and the next restart point as belonging to that
	// restart point. Note that index could be equal to i.numRestarts, i.e., we
	// are past the last restart.
	//
	// Since keys are strictly increasing, if index > 0 then the restart point
	// at index-1 will be the first one that has some keys belonging to it that
	// are less than the search key.  If index == 0, then all keys in this block
	// are larger than the search key, so there is no match.
	targetOffset := i.restarts
	if index > 0 {
		i.offset = decodeRestart(i.data[i.restarts+4*(index-1):])
		if index < i.numRestarts {
			targetOffset = decodeRestart(i.data[i.restarts+4*(index):])
		}
	} else if index == 0 {
		// If index == 0 then all keys in this block are larger than the key
		// sought.
		i.offset = -1
		i.nextOffset = 0
		return nil, base.LazyValue{}
	}

	// Iterate from that restart point to somewhere >= the key sought, then back
	// up to the previous entry. The expectation is that we'll be performing
	// reverse iteration, so we cache the entries as we advance forward.
	i.nextOffset = i.offset

	for {
		i.offset = i.nextOffset
		i.readEntry()
		// When hidden keys are common, there is additional optimization possible
		// by not caching entries that are hidden (note that some calls to
		// cacheEntry don't decode the internal key before caching, but checking
		// whether a key is hidden does not require full decoding). However, we do
		// need to use the blockEntry.offset in the cache for the first entry at
		// the reset point to do the binary search when the cache is empty -- so
		// we would need to cache that first entry (though not the key) even if
		// was hidden. Our current assumption is that if there are large numbers
		// of hidden keys we will be able to skip whole blocks (using block
		// property filters) so we don't bother optimizing.
		hiddenPoint := i.decodeInternalKey(i.key)

		// NB: we don't use the hiddenPoint return value of decodeInternalKey
		// since we want to stop as soon as we reach a key >= ikey.UserKey, so
		// that we can reverse.
		if i.cmp(i.ikey.UserKey, key) >= 0 {
			// The current key is greater than or equal to our search key. Back up to
			// the previous key which was less than our search key. Note that this for
			// loop will execute at least once with this if-block not being true, so
			// the key we are backing up to is the last one this loop cached.
			return i.Prev()
		}

		if i.nextOffset >= targetOffset {
			// We've reached the end of the current restart block. Return the
			// current key if not hidden, else call Prev().
			//
			// When the restart interval is 1, the first iteration of the for loop
			// will bring us here. In that case ikey is backed by the block so we
			// get the desired key stability guarantee for the lifetime of the
			// blockIter. That is, we never cache anything and therefore never
			// return a key backed by cachedBuf.
			if hiddenPoint {
				return i.Prev()
			}
			break
		}

		i.cacheEntry()
	}

	if !i.valid() {
		return nil, base.LazyValue{}
	}
	if !i.lazyValueHandling.hasValuePrefix ||
		base.TrailerKind(i.ikey.Trailer) != InternalKeyKindSet {
		i.lazyValue = base.MakeInPlaceValue(i.val)
	} else if i.lazyValueHandling.vbr == nil || !isValueHandle(valuePrefix(i.val[0])) {
		i.lazyValue = base.MakeInPlaceValue(i.val[1:])
	} else {
		i.lazyValue = i.lazyValueHandling.vbr.getLazyValueForPrefixAndValueHandle(i.val)
	}
	return &i.ikey, i.lazyValue
}

// First implements internalIterator.First, as documented in the pebble
// package.
func (i *blockIter) First() (*InternalKey, base.LazyValue) {
	if invariants.Enabled && i.isDataInvalidated() {
		panic(errors.AssertionFailedf("invalidated blockIter used"))
	}

	i.offset = 0
	if !i.valid() {
		return nil, base.LazyValue{}
	}
	i.clearCache()
	i.readEntry()
	hiddenPoint := i.decodeInternalKey(i.key)
	if hiddenPoint {
		return i.Next()
	}
	if !i.lazyValueHandling.hasValuePrefix ||
		base.TrailerKind(i.ikey.Trailer) != InternalKeyKindSet {
		i.lazyValue = base.MakeInPlaceValue(i.val)
	} else if i.lazyValueHandling.vbr == nil || !isValueHandle(valuePrefix(i.val[0])) {
		i.lazyValue = base.MakeInPlaceValue(i.val[1:])
	} else {
		i.lazyValue = i.lazyValueHandling.vbr.getLazyValueForPrefixAndValueHandle(i.val)
	}
	return &i.ikey, i.lazyValue
}

func decodeRestart(b []byte) int32 {
	_ = b[3] // bounds check hint to compiler; see golang.org/issue/14808
	return int32(uint32(b[0]) | uint32(b[1])<<8 | uint32(b[2])<<16 |
		uint32(b[3]&restartMaskLittleEndianHighByteWithoutSetHasSamePrefix)<<24)
}

// Last implements internalIterator.Last, as documented in the pebble package.
func (i *blockIter) Last() (*InternalKey, base.LazyValue) {
	if invariants.Enabled && i.isDataInvalidated() {
		panic(errors.AssertionFailedf("invalidated blockIter used"))
	}

	// Seek forward from the last restart point.
	i.offset = decodeRestart(i.data[i.restarts+4*(i.numRestarts-1):])
	if !i.valid() {
		return nil, base.LazyValue{}
	}

	i.readEntry()
	i.clearCache()

	for i.nextOffset < i.restarts {
		i.cacheEntry()
		i.offset = i.nextOffset
		i.readEntry()
	}

	hiddenPoint := i.decodeInternalKey(i.key)
	if hiddenPoint {
		return i.Prev()
	}
	if !i.lazyValueHandling.hasValuePrefix ||
		base.TrailerKind(i.ikey.Trailer) != InternalKeyKindSet {
		i.lazyValue = base.MakeInPlaceValue(i.val)
	} else if i.lazyValueHandling.vbr == nil || !isValueHandle(valuePrefix(i.val[0])) {
		i.lazyValue = base.MakeInPlaceValue(i.val[1:])
	} else {
		i.lazyValue = i.lazyValueHandling.vbr.getLazyValueForPrefixAndValueHandle(i.val)
	}
	return &i.ikey, i.lazyValue
}

// Next implements internalIterator.Next, as documented in the pebble
// package.
func (i *blockIter) Next() (*InternalKey, base.LazyValue) {
	if len(i.cachedBuf) > 0 {
		// We're switching from reverse iteration to forward iteration. We need to
		// populate i.fullKey with the current key we're positioned at so that
		// readEntry() can use i.fullKey for key prefix decompression. Note that we
		// don't know whether i.key is backed by i.cachedBuf or i.fullKey (if
		// SeekLT was the previous call, i.key may be backed by i.fullKey), but
		// copying into i.fullKey works for both cases.
		//
		// TODO(peter): Rather than clearing the cache, we could instead use the
		// cache until it is exhausted. This would likely be faster than falling
		// through to the normal forward iteration code below.
		i.fullKey = append(i.fullKey[:0], i.key...)
		i.clearCache()
	}

start:
	i.offset = i.nextOffset
	if !i.valid() {
		return nil, base.LazyValue{}
	}
	i.readEntry()
	// Manually inlined version of i.decodeInternalKey(i.key).
	if n := len(i.key) - 8; n >= 0 {
		trailer := binary.LittleEndian.Uint64(i.key[n:])
		hiddenPoint := i.hideObsoletePoints &&
			(trailer&trailerObsoleteBit != 0)
		i.ikey.Trailer = trailer & trailerObsoleteMask
		i.ikey.UserKey = i.key[:n:n]
		if i.globalSeqNum != 0 {
			i.ikey.SetSeqNum(i.globalSeqNum)
		}
		if hiddenPoint {
			goto start
		}
	} else {
		i.ikey.Trailer = uint64(InternalKeyKindInvalid)
		i.ikey.UserKey = nil
	}
	if !i.lazyValueHandling.hasValuePrefix ||
		base.TrailerKind(i.ikey.Trailer) != InternalKeyKindSet {
		i.lazyValue = base.MakeInPlaceValue(i.val)
	} else if i.lazyValueHandling.vbr == nil || !isValueHandle(valuePrefix(i.val[0])) {
		i.lazyValue = base.MakeInPlaceValue(i.val[1:])
	} else {
		i.lazyValue = i.lazyValueHandling.vbr.getLazyValueForPrefixAndValueHandle(i.val)
	}
	return &i.ikey, i.lazyValue
}

// NextPrefix implements (base.InternalIterator).NextPrefix.
func (i *blockIter) NextPrefix(succKey []byte) (*InternalKey, base.LazyValue) {
	if i.lazyValueHandling.hasValuePrefix {
		return i.nextPrefixV3(succKey)
	}
	const nextsBeforeSeek = 3
	k, v := i.Next()
	for j := 1; k != nil && i.cmp(k.UserKey, succKey) < 0; j++ {
		if j >= nextsBeforeSeek {
			return i.SeekGE(succKey, base.SeekGEFlagsNone)
		}
		k, v = i.Next()
	}
	return k, v
}

func (i *blockIter) nextPrefixV3(succKey []byte) (*InternalKey, base.LazyValue) {
	// Doing nexts that involve a key comparison can be expensive (and the cost
	// depends on the key length), so we use the same threshold of 3 that we use
	// for TableFormatPebblev2 in blockIter.nextPrefix above. The next fast path
	// that looks at setHasSamePrefix takes ~5ns per key, which is ~150x faster
	// than doing a SeekGE within the block, so we do this 16 times
	// (~5ns*16=80ns), and then switch to looking at restarts. Doing the binary
	// search for the restart consumes > 100ns. If the number of versions is >
	// 17, we will increment nextFastCount to 17, then do a binary search, and
	// on average need to find a key between two restarts, so another 8 steps
	// corresponding to nextFastCount, for a mean total of 17 + 8 = 25 such
	// steps.
	//
	// TODO(sumeer): use the configured restartInterval for the sstable when it
	// was written (which we don't currently store) instead of the default value
	// of 16.
	const nextCmpThresholdBeforeSeek = 3
	const nextFastThresholdBeforeRestarts = 16
	nextCmpCount := 0
	nextFastCount := 0
	usedRestarts := false
	// INVARIANT: blockIter is valid.
	if invariants.Enabled && !i.valid() {
		panic(errors.AssertionFailedf("nextPrefixV3 called on invalid blockIter"))
	}
	prevKeyIsSet := i.ikey.Kind() == InternalKeyKindSet
	for {
		i.offset = i.nextOffset
		if !i.valid() {
			return nil, base.LazyValue{}
		}
		// Need to decode the length integers, so we can compute nextOffset.
		ptr := unsafe.Pointer(uintptr(i.ptr) + uintptr(i.offset))
		// This is an ugly performance hack. Reading entries from blocks is one of
		// the inner-most routines and decoding the 3 varints per-entry takes
		// significant time. Neither go1.11 or go1.12 will inline decodeVarint for
		// us, so we do it manually. This provides a 10-15% performance improvement
		// on blockIter benchmarks on both go1.11 and go1.12.
		//
		// TODO(peter): remove this hack if go:inline is ever supported.

		// Decode the shared key length integer.
		var shared uint32
		if a := *((*uint8)(ptr)); a < 128 {
			shared = uint32(a)
			ptr = unsafe.Pointer(uintptr(ptr) + 1)
		} else if a, b := a&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 1))); b < 128 {
			shared = uint32(b)<<7 | uint32(a)
			ptr = unsafe.Pointer(uintptr(ptr) + 2)
		} else if b, c := b&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 2))); c < 128 {
			shared = uint32(c)<<14 | uint32(b)<<7 | uint32(a)
			ptr = unsafe.Pointer(uintptr(ptr) + 3)
		} else if c, d := c&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 3))); d < 128 {
			shared = uint32(d)<<21 | uint32(c)<<14 | uint32(b)<<7 | uint32(a)
			ptr = unsafe.Pointer(uintptr(ptr) + 4)
		} else {
			d, e := d&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 4)))
			shared = uint32(e)<<28 | uint32(d)<<21 | uint32(c)<<14 | uint32(b)<<7 | uint32(a)
			ptr = unsafe.Pointer(uintptr(ptr) + 5)
		}
		// Decode the unshared key length integer.
		var unshared uint32
		if a := *((*uint8)(ptr)); a < 128 {
			unshared = uint32(a)
			ptr = unsafe.Pointer(uintptr(ptr) + 1)
		} else if a, b := a&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 1))); b < 128 {
			unshared = uint32(b)<<7 | uint32(a)
			ptr = unsafe.Pointer(uintptr(ptr) + 2)
		} else if b, c := b&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 2))); c < 128 {
			unshared = uint32(c)<<14 | uint32(b)<<7 | uint32(a)
			ptr = unsafe.Pointer(uintptr(ptr) + 3)
		} else if c, d := c&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 3))); d < 128 {
			unshared = uint32(d)<<21 | uint32(c)<<14 | uint32(b)<<7 | uint32(a)
			ptr = unsafe.Pointer(uintptr(ptr) + 4)
		} else {
			d, e := d&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 4)))
			unshared = uint32(e)<<28 | uint32(d)<<21 | uint32(c)<<14 | uint32(b)<<7 | uint32(a)
			ptr = unsafe.Pointer(uintptr(ptr) + 5)
		}
		// Decode the value length integer.
		var value uint32
		if a := *((*uint8)(ptr)); a < 128 {
			value = uint32(a)
			ptr = unsafe.Pointer(uintptr(ptr) + 1)
		} else if a, b := a&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 1))); b < 128 {
			value = uint32(b)<<7 | uint32(a)
			ptr = unsafe.Pointer(uintptr(ptr) + 2)
		} else if b, c := b&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 2))); c < 128 {
			value = uint32(c)<<14 | uint32(b)<<7 | uint32(a)
			ptr = unsafe.Pointer(uintptr(ptr) + 3)
		} else if c, d := c&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 3))); d < 128 {
			value = uint32(d)<<21 | uint32(c)<<14 | uint32(b)<<7 | uint32(a)
			ptr = unsafe.Pointer(uintptr(ptr) + 4)
		} else {
			d, e := d&0x7f, *((*uint8)(unsafe.Pointer(uintptr(ptr) + 4)))
			value = uint32(e)<<28 | uint32(d)<<21 | uint32(c)<<14 | uint32(b)<<7 | uint32(a)
			ptr = unsafe.Pointer(uintptr(ptr) + 5)
		}
		// The starting position of the value.
		valuePtr := unsafe.Pointer(uintptr(ptr) + uintptr(unshared))
		i.nextOffset = int32(uintptr(valuePtr)-uintptr(i.ptr)) + int32(value)
		if invariants.Enabled && unshared < 8 {
			// This should not happen since only the key prefix is shared, so even
			// if the prefix length is the same as the user key length, the unshared
			// will include the trailer.
			panic(errors.AssertionFailedf("unshared %d is too small", unshared))
		}
		// The trailer is written in little endian, so the key kind is the first
		// byte in the trailer that is encoded in the slice [unshared-8:unshared].
		keyKind := InternalKeyKind((*[manual.MaxArrayLen]byte)(ptr)[unshared-8])
		keyKind = keyKind & base.InternalKeyKindSSTableInternalObsoleteMask
		prefixChanged := false
		if keyKind == InternalKeyKindSet {
			if invariants.Enabled && value == 0 {
				panic(errors.AssertionFailedf("value is of length 0, but we expect a valuePrefix"))
			}
			valPrefix := *((*valuePrefix)(valuePtr))
			if setHasSamePrefix(valPrefix) {
				// Fast-path. No need to assemble i.fullKey, or update i.key. We know
				// that subsequent keys will not have a shared length that is greater
				// than the prefix of the current key, which is also the prefix of
				// i.key. Since we are continuing to iterate, we don't need to
				// initialize i.ikey and i.lazyValue (these are initialized before
				// returning).
				nextFastCount++
				if nextFastCount > nextFastThresholdBeforeRestarts {
					if usedRestarts {
						// Exhausted iteration budget. This will never happen unless
						// someone is using a restart interval > 16. It is just to guard
						// against long restart intervals causing too much iteration.
						break
					}
					// Haven't used restarts yet, so find the first restart at or beyond
					// the current offset.
					targetOffset := i.offset
					var index int32
					{
						// NB: manually inlined sort.Sort is ~5% faster.
						//
						// f defined for a restart point is true iff the offset >=
						// targetOffset.
						// Define f(-1) == false and f(i.numRestarts) == true.
						// Invariant: f(index-1) == false, f(upper) == true.
						upper := i.numRestarts
						for index < upper {
							h := int32(uint(index+upper) >> 1) // avoid overflow when computing h
							// index ≤ h < upper
							offset := decodeRestart(i.data[i.restarts+4*h:])
							if offset < targetOffset {
								index = h + 1 // preserves f(index-1) == false
							} else {
								upper = h // preserves f(upper) == true
							}
						}
						// index == upper, f(index-1) == false, and f(upper) (= f(index)) == true
						// => answer is index.
					}
					usedRestarts = true
					nextFastCount = 0
					if index == i.numRestarts {
						// Already past the last real restart, so iterate a bit more until
						// we are done with the block.
						continue
					}
					// Have some real restarts after index. NB: index is the first
					// restart at or beyond the current offset.
					startingIndex := index
					for index != i.numRestarts &&
						// The restart at index is 4 bytes written in little endian format
						// starting at i.restart+4*index. The 0th byte is the least
						// significant and the 3rd byte is the most significant. Since the
						// most significant bit of the 3rd byte is what we use for
						// encoding the set-has-same-prefix information, the indexing
						// below has +3.
						i.data[i.restarts+4*index+3]&restartMaskLittleEndianHighByteOnlySetHasSamePrefix != 0 {
						// We still have the same prefix, so move to the next restart.
						index++
					}
					// index is the first restart that did not have the same prefix.
					if index != startingIndex {
						// Managed to skip past at least one restart. Resume iteration
						// from index-1. Since nextFastCount has been reset to 0, we
						// should be able to iterate to the next prefix.
						i.offset = decodeRestart(i.data[i.restarts+4*(index-1):])
						i.readEntry()
					}
					// Else, unable to skip past any restart. Resume iteration. Since
					// nextFastCount has been reset to 0, we should be able to iterate
					// to the next prefix.
					continue
				}
				continue
			} else if prevKeyIsSet {
				prefixChanged = true
			}
		} else {
			prevKeyIsSet = false
		}
		// Slow-path cases:
		// - (Likely) The prefix has changed.
		// - (Unlikely) The prefix has not changed.
		// We assemble the key etc. under the assumption that it is the likely
		// case.
		unsharedKey := getBytes(ptr, int(unshared))
		// TODO(sumeer): move this into the else block below. This is a bit tricky
		// since the current logic assumes we have always copied the latest key
		// into fullKey, which is why when we get to the next key we can (a)
		// access i.fullKey[:shared], (b) append only the unsharedKey to
		// i.fullKey. For (a), we can access i.key[:shared] since that memory is
		// valid (even if unshared). For (b), we will need to remember whether
		// i.key refers to i.fullKey or not, and can append the unsharedKey only
		// in the former case and for the latter case need to copy the shared part
		// too. This same comment applies to the other place where we can do this
		// optimization, in readEntry().
		i.fullKey = append(i.fullKey[:shared], unsharedKey...)
		i.val = getBytes(valuePtr, int(value))
		if shared == 0 {
			// Provide stability for the key across positioning calls if the key
			// doesn't share a prefix with the previous key. This removes requiring the
			// key to be copied if the caller knows the block has a restart interval of
			// 1. An important example of this is range-del blocks.
			i.key = unsharedKey
		} else {
			i.key = i.fullKey
		}
		// Manually inlined version of i.decodeInternalKey(i.key).
		hiddenPoint := false
		if n := len(i.key) - 8; n >= 0 {
			trailer := binary.LittleEndian.Uint64(i.key[n:])
			hiddenPoint = i.hideObsoletePoints &&
				(trailer&trailerObsoleteBit != 0)
			i.ikey.Trailer = trailer & trailerObsoleteMask
			i.ikey.UserKey = i.key[:n:n]
			if i.globalSeqNum != 0 {
				i.ikey.SetSeqNum(i.globalSeqNum)
			}
		} else {
			i.ikey.Trailer = uint64(InternalKeyKindInvalid)
			i.ikey.UserKey = nil
		}
		nextCmpCount++
		if invariants.Enabled && prefixChanged && i.cmp(i.ikey.UserKey, succKey) < 0 {
			panic(errors.AssertionFailedf("prefix should have changed but %x < %x",
				i.ikey.UserKey, succKey))
		}
		if prefixChanged || i.cmp(i.ikey.UserKey, succKey) >= 0 {
			// Prefix has changed.
			if hiddenPoint {
				return i.Next()
			}
			if invariants.Enabled && !i.lazyValueHandling.hasValuePrefix {
				panic(errors.AssertionFailedf("nextPrefixV3 being run for non-v3 sstable"))
			}
			if base.TrailerKind(i.ikey.Trailer) != InternalKeyKindSet {
				i.lazyValue = base.MakeInPlaceValue(i.val)
			} else if i.lazyValueHandling.vbr == nil || !isValueHandle(valuePrefix(i.val[0])) {
				i.lazyValue = base.MakeInPlaceValue(i.val[1:])
			} else {
				i.lazyValue = i.lazyValueHandling.vbr.getLazyValueForPrefixAndValueHandle(i.val)
			}
			return &i.ikey, i.lazyValue
		}
		// Else prefix has not changed.

		if nextCmpCount >= nextCmpThresholdBeforeSeek {
			break
		}
	}
	return i.SeekGE(succKey, base.SeekGEFlagsNone)
}

// Prev implements internalIterator.Prev, as documented in the pebble
// package.
func (i *blockIter) Prev() (*InternalKey, base.LazyValue) {
start:
	for n := len(i.cached) - 1; n >= 0; n-- {
		i.nextOffset = i.offset
		e := &i.cached[n]
		i.offset = e.offset
		i.val = getBytes(unsafe.Pointer(uintptr(i.ptr)+uintptr(e.valStart)), int(e.valSize))
		// Manually inlined version of i.decodeInternalKey(i.key).
		i.key = i.cachedBuf[e.keyStart:e.keyEnd]
		if n := len(i.key) - 8; n >= 0 {
			trailer := binary.LittleEndian.Uint64(i.key[n:])
			hiddenPoint := i.hideObsoletePoints &&
				(trailer&trailerObsoleteBit != 0)
			if hiddenPoint {
				continue
			}
			i.ikey.Trailer = trailer & trailerObsoleteMask
			i.ikey.UserKey = i.key[:n:n]
			if i.globalSeqNum != 0 {
				i.ikey.SetSeqNum(i.globalSeqNum)
			}
		} else {
			i.ikey.Trailer = uint64(InternalKeyKindInvalid)
			i.ikey.UserKey = nil
		}
		i.cached = i.cached[:n]
		if !i.lazyValueHandling.hasValuePrefix ||
			base.TrailerKind(i.ikey.Trailer) != InternalKeyKindSet {
			i.lazyValue = base.MakeInPlaceValue(i.val)
		} else if i.lazyValueHandling.vbr == nil || !isValueHandle(valuePrefix(i.val[0])) {
			i.lazyValue = base.MakeInPlaceValue(i.val[1:])
		} else {
			i.lazyValue = i.lazyValueHandling.vbr.getLazyValueForPrefixAndValueHandle(i.val)
		}
		return &i.ikey, i.lazyValue
	}

	i.clearCache()
	if i.offset <= 0 {
		i.offset = -1
		i.nextOffset = 0
		return nil, base.LazyValue{}
	}

	targetOffset := i.offset
	var index int32

	{
		// NB: manually inlined sort.Sort is ~5% faster.
		//
		// Define f(-1) == false and f(n) == true.
		// Invariant: f(index-1) == false, f(upper) == true.
		upper := i.numRestarts
		for index < upper {
			h := int32(uint(index+upper) >> 1) // avoid overflow when computing h
			// index ≤ h < upper
			offset := decodeRestart(i.data[i.restarts+4*h:])
			if offset < targetOffset {
				// Looking for the first restart that has offset >= targetOffset, so
				// ignore h and earlier.
				index = h + 1 // preserves f(i-1) == false
			} else {
				upper = h // preserves f(j) == true
			}
		}
		// index == upper, f(index-1) == false, and f(upper) (= f(index)) == true
		// => answer is index.
	}

	// index is first restart with offset >= targetOffset. Note that
	// targetOffset may not be at a restart point since one can call Prev()
	// after Next() (so the cache was not populated) and targetOffset refers to
	// the current entry. index-1 must have an offset < targetOffset (it can't
	// be equal to targetOffset since the binary search would have selected that
	// as the index).
	i.offset = 0
	if index > 0 {
		i.offset = decodeRestart(i.data[i.restarts+4*(index-1):])
	}
	// TODO(sumeer): why is the else case not an error given targetOffset is a
	// valid offset.

	i.readEntry()

	// We stop when i.nextOffset == targetOffset since the targetOffset is the
	// entry we are stepping back from, and we don't need to cache the entry
	// before it, since it is the candidate to return.
	for i.nextOffset < targetOffset {
		i.cacheEntry()
		i.offset = i.nextOffset
		i.readEntry()
	}

	hiddenPoint := i.decodeInternalKey(i.key)
	if hiddenPoint {
		// Use the cache.
		goto start
	}
	if !i.lazyValueHandling.hasValuePrefix ||
		base.TrailerKind(i.ikey.Trailer) != InternalKeyKindSet {
		i.lazyValue = base.MakeInPlaceValue(i.val)
	} else if i.lazyValueHandling.vbr == nil || !isValueHandle(valuePrefix(i.val[0])) {
		i.lazyValue = base.MakeInPlaceValue(i.val[1:])
	} else {
		i.lazyValue = i.lazyValueHandling.vbr.getLazyValueForPrefixAndValueHandle(i.val)
	}
	return &i.ikey, i.lazyValue
}

// Key implements internalIterator.Key, as documented in the pebble package.
func (i *blockIter) Key() *InternalKey {
	return &i.ikey
}

func (i *blockIter) value() base.LazyValue {
	return i.lazyValue
}

// Error implements internalIterator.Error, as documented in the pebble
// package.
func (i *blockIter) Error() error {
	return nil // infallible
}

// Close implements internalIterator.Close, as documented in the pebble
// package.
func (i *blockIter) Close() error {
	i.handle.Release()
	i.handle = bufferHandle{}
	i.val = nil
	i.lazyValue = base.LazyValue{}
	i.lazyValueHandling.vbr = nil
	return nil
}

func (i *blockIter) SetBounds(lower, upper []byte) {
	// This should never be called as bounds are handled by sstable.Iterator.
	panic("pebble: SetBounds unimplemented")
}

func (i *blockIter) valid() bool {
	return i.offset >= 0 && i.offset < i.restarts
}

// fragmentBlockIter wraps a blockIter, implementing the
// keyspan.FragmentIterator interface. It's used for reading range deletion and
// range key blocks.
//
// Range deletions and range keys are fragmented before they're persisted to the
// block. Overlapping fragments have identical bounds.  The fragmentBlockIter
// gathers all the fragments with identical bounds within a block and returns a
// single keyspan.Span describing all the keys defined over the span.
//
// # Memory lifetime
//
// A Span returned by fragmentBlockIter is only guaranteed to be stable until
// the next fragmentBlockIter iteration positioning method. A Span's Keys slice
// may be reused, so the user must not assume it's stable.
//
// Blocks holding range deletions and range keys are configured to use a restart
// interval of 1. This provides key stability. The caller may treat the various
// byte slices (start, end, suffix, value) as stable for the lifetime of the
// iterator.
type fragmentBlockIter struct {
	blockIter blockIter
	keyBuf    [2]keyspan.Key
	span      keyspan.Span
	err       error
	dir       int8
	closeHook func(i keyspan.FragmentIterator) error

	// elideSameSeqnum, if true, returns only the first-occurring (in forward
	// order) Key for each sequence number.
	elideSameSeqnum bool
}

func (i *fragmentBlockIter) resetForReuse() fragmentBlockIter {
	return fragmentBlockIter{blockIter: i.blockIter.resetForReuse()}
}

func (i *fragmentBlockIter) decodeSpanKeys(k *InternalKey, internalValue []byte) {
	// TODO(jackson): The use of i.span.Keys to accumulate keys across multiple
	// calls to Decode is too confusing and subtle. Refactor to make it
	// explicit.

	// decode the contents of the fragment's value. This always includes at
	// least the end key: RANGEDELs store the end key directly as the value,
	// whereas the various range key kinds store are more complicated.  The
	// details of the range key internal value format are documented within the
	// internal/rangekey package.
	switch k.Kind() {
	case base.InternalKeyKindRangeDelete:
		i.span = rangedel.Decode(*k, internalValue, i.span.Keys)
		i.err = nil
	case base.InternalKeyKindRangeKeySet, base.InternalKeyKindRangeKeyUnset, base.InternalKeyKindRangeKeyDelete:
		i.span, i.err = rangekey.Decode(*k, internalValue, i.span.Keys)
	default:
		i.span = keyspan.Span{}
		i.err = base.CorruptionErrorf("pebble: corrupt keyspan fragment of kind %d", k.Kind())
	}
}

func (i *fragmentBlockIter) elideKeysOfSameSeqNum() {
	if invariants.Enabled {
		if !i.elideSameSeqnum || len(i.span.Keys) == 0 {
			panic("elideKeysOfSameSeqNum called when it should not be")
		}
	}
	lastSeqNum := i.span.Keys[0].SeqNum()
	k := 1
	for j := 1; j < len(i.span.Keys); j++ {
		if lastSeqNum != i.span.Keys[j].SeqNum() {
			lastSeqNum = i.span.Keys[j].SeqNum()
			i.span.Keys[k] = i.span.Keys[j]
			k++
		}
	}
	i.span.Keys = i.span.Keys[:k]
}

// gatherForward gathers internal keys with identical bounds. Keys defined over
// spans of the keyspace are fragmented such that any overlapping key spans have
// identical bounds. When these spans are persisted to a range deletion or range
// key block, they may be persisted as multiple internal keys in order to encode
// multiple sequence numbers or key kinds.
//
// gatherForward iterates forward, re-combining the fragmented internal keys to
// reconstruct a keyspan.Span that holds all the keys defined over the span.
func (i *fragmentBlockIter) gatherForward(k *InternalKey, lazyValue base.LazyValue) *keyspan.Span {
	i.span = keyspan.Span{}
	if k == nil || !i.blockIter.valid() {
		return nil
	}
	i.err = nil
	// Use the i.keyBuf array to back the Keys slice to prevent an allocation
	// when a span contains few keys.
	i.span.Keys = i.keyBuf[:0]

	// Decode the span's end key and individual keys from the value.
	internalValue := lazyValue.InPlaceValue()
	i.decodeSpanKeys(k, internalValue)
	if i.err != nil {
		return nil
	}
	prevEnd := i.span.End

	// There might exist additional internal keys with identical bounds encoded
	// within the block. Iterate forward, accumulating all the keys with
	// identical bounds to s.
	k, lazyValue = i.blockIter.Next()
	internalValue = lazyValue.InPlaceValue()
	for k != nil && i.blockIter.cmp(k.UserKey, i.span.Start) == 0 {
		i.decodeSpanKeys(k, internalValue)
		if i.err != nil {
			return nil
		}

		// Since k indicates an equal start key, the encoded end key must
		// exactly equal the original end key from the first internal key.
		// Overlapping fragments are required to have exactly equal start and
		// end bounds.
		if i.blockIter.cmp(prevEnd, i.span.End) != 0 {
			i.err = base.CorruptionErrorf("pebble: corrupt keyspan fragmentation")
			i.span = keyspan.Span{}
			return nil
		}
		k, lazyValue = i.blockIter.Next()
		internalValue = lazyValue.InPlaceValue()
	}
	if i.elideSameSeqnum && len(i.span.Keys) > 0 {
		i.elideKeysOfSameSeqNum()
	}
	// i.blockIter is positioned over the first internal key for the next span.
	return &i.span
}

// gatherBackward gathers internal keys with identical bounds. Keys defined over
// spans of the keyspace are fragmented such that any overlapping key spans have
// identical bounds. When these spans are persisted to a range deletion or range
// key block, they may be persisted as multiple internal keys in order to encode
// multiple sequence numbers or key kinds.
//
// gatherBackward iterates backwards, re-combining the fragmented internal keys
// to reconstruct a keyspan.Span that holds all the keys defined over the span.
func (i *fragmentBlockIter) gatherBackward(k *InternalKey, lazyValue base.LazyValue) *keyspan.Span {
	i.span = keyspan.Span{}
	if k == nil || !i.blockIter.valid() {
		return nil
	}
	i.err = nil
	// Use the i.keyBuf array to back the Keys slice to prevent an allocation
	// when a span contains few keys.
	i.span.Keys = i.keyBuf[:0]

	// Decode the span's end key and individual keys from the value.
	internalValue := lazyValue.InPlaceValue()
	i.decodeSpanKeys(k, internalValue)
	if i.err != nil {
		return nil
	}
	prevEnd := i.span.End

	// There might exist additional internal keys with identical bounds encoded
	// within the block. Iterate backward, accumulating all the keys with
	// identical bounds to s.
	k, lazyValue = i.blockIter.Prev()
	internalValue = lazyValue.InPlaceValue()
	for k != nil && i.blockIter.cmp(k.UserKey, i.span.Start) == 0 {
		i.decodeSpanKeys(k, internalValue)
		if i.err != nil {
			return nil
		}

		// Since k indicates an equal start key, the encoded end key must
		// exactly equal the original end key from the first internal key.
		// Overlapping fragments are required to have exactly equal start and
		// end bounds.
		if i.blockIter.cmp(prevEnd, i.span.End) != 0 {
			i.err = base.CorruptionErrorf("pebble: corrupt keyspan fragmentation")
			i.span = keyspan.Span{}
			return nil
		}
		k, lazyValue = i.blockIter.Prev()
		internalValue = lazyValue.InPlaceValue()
	}
	// i.blockIter is positioned over the last internal key for the previous
	// span.

	// Backwards iteration encounters internal keys in the wrong order.
	keyspan.SortKeysByTrailer(&i.span.Keys)

	if i.elideSameSeqnum && len(i.span.Keys) > 0 {
		i.elideKeysOfSameSeqNum()
	}
	return &i.span
}

// Error implements (keyspan.FragmentIterator).Error.
func (i *fragmentBlockIter) Error() error {
	return i.err
}

// Close implements (keyspan.FragmentIterator).Close.
func (i *fragmentBlockIter) Close() error {
	var err error
	if i.closeHook != nil {
		err = i.closeHook(i)
	}
	err = firstError(err, i.blockIter.Close())
	return err
}

// First implements (keyspan.FragmentIterator).First
func (i *fragmentBlockIter) First() *keyspan.Span {
	i.dir = +1
	return i.gatherForward(i.blockIter.First())
}

// Last implements (keyspan.FragmentIterator).Last.
func (i *fragmentBlockIter) Last() *keyspan.Span {
	i.dir = -1
	return i.gatherBackward(i.blockIter.Last())
}

// Next implements (keyspan.FragmentIterator).Next.
func (i *fragmentBlockIter) Next() *keyspan.Span {
	switch {
	case i.dir == -1 && !i.span.Valid():
		// Switching directions.
		//
		// i.blockIter is exhausted, before the first key. Move onto the first.
		i.blockIter.First()
		i.dir = +1
	case i.dir == -1 && i.span.Valid():
		// Switching directions.
		//
		// i.blockIter is currently positioned over the last internal key for
		// the previous span. Next it once to move to the first internal key
		// that makes up the current span, and gatherForwaad to land on the
		// first internal key making up the next span.
		//
		// In the diagram below, if the last span returned to the user during
		// reverse iteration was [b,c), i.blockIter is currently positioned at
		// [a,b). The block iter must be positioned over [d,e) to gather the
		// next span's fragments.
		//
		//    ... [a,b) [b,c) [b,c) [b,c) [d,e) ...
		//          ^                       ^
		//     i.blockIter                 want
		if x := i.gatherForward(i.blockIter.Next()); invariants.Enabled && !x.Valid() {
			panic("pebble: invariant violation: next entry unexpectedly invalid")
		}
		i.dir = +1
	}
	// We know that this blockIter has in-place values.
	return i.gatherForward(&i.blockIter.ikey, base.MakeInPlaceValue(i.blockIter.val))
}

// Prev implements (keyspan.FragmentIterator).Prev.
func (i *fragmentBlockIter) Prev() *keyspan.Span {
	switch {
	case i.dir == +1 && !i.span.Valid():
		// Switching directions.
		//
		// i.blockIter is exhausted, after the last key. Move onto the last.
		i.blockIter.Last()
		i.dir = -1
	case i.dir == +1 && i.span.Valid():
		// Switching directions.
		//
		// i.blockIter is currently positioned over the first internal key for
		// the next span. Prev it once to move to the last internal key that
		// makes up the current span, and gatherBackward to land on the last
		// internal key making up the previous span.
		//
		// In the diagram below, if the last span returned to the user during
		// forward iteration was [b,c), i.blockIter is currently positioned at
		// [d,e). The block iter must be positioned over [a,b) to gather the
		// previous span's fragments.
		//
		//    ... [a,b) [b,c) [b,c) [b,c) [d,e) ...
		//          ^                       ^
		//        want                  i.blockIter
		if x := i.gatherBackward(i.blockIter.Prev()); invariants.Enabled && !x.Valid() {
			panic("pebble: invariant violation: previous entry unexpectedly invalid")
		}
		i.dir = -1
	}
	// We know that this blockIter has in-place values.
	return i.gatherBackward(&i.blockIter.ikey, base.MakeInPlaceValue(i.blockIter.val))
}

// SeekGE implements (keyspan.FragmentIterator).SeekGE.
func (i *fragmentBlockIter) SeekGE(k []byte) *keyspan.Span {
	if s := i.SeekLT(k); s != nil && i.blockIter.cmp(k, s.End) < 0 {
		return s
	}
	// TODO(jackson): If the above i.SeekLT(k) discovers a span but the span
	// doesn't meet the k < s.End comparison, then there's no need for the
	// SeekLT to gatherBackward.
	return i.Next()
}

// SeekLT implements (keyspan.FragmentIterator).SeekLT.
func (i *fragmentBlockIter) SeekLT(k []byte) *keyspan.Span {
	i.dir = -1
	return i.gatherBackward(i.blockIter.SeekLT(k, base.SeekLTFlagsNone))
}

// String implements fmt.Stringer.
func (i *fragmentBlockIter) String() string {
	return "fragment-block-iter"
}

// SetCloseHook implements sstable.FragmentIterator.
func (i *fragmentBlockIter) SetCloseHook(fn func(i keyspan.FragmentIterator) error) {
	i.closeHook = fn
}
