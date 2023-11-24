/*
 * Copyright 2017 Dgraph Labs, Inc. and Contributors
 * Modifications copyright (C) 2017 Andy Kimball and Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/*
Adapted from RocksDB inline skiplist.

Key differences:
- No optimization for sequential inserts (no "prev").
- No custom comparator.
- Support overwrites. This requires care when we see the same key when inserting.
  For RocksDB or LevelDB, overwrites are implemented as a newer sequence number in the key, so
	there is no need for values. We don't intend to support versioning. In-place updates of values
	would be more efficient.
- We discard all non-concurrent code.
- We do not support Splices. This simplifies the code a lot.
- No AllocateNode or other pointer arithmetic.
- We combine the findLessThan, findGreaterOrEqual, etc into one function.
*/

/*
Further adapted from Badger: https://github.com/dgraph-io/badger.

Key differences:
- Support for previous pointers - doubly linked lists. Note that it's up to higher
  level code to deal with the intermediate state that occurs during insertion,
  where node A is linked to node B, but node B is not yet linked back to node A.
- Iterator includes mutator functions.
*/

package arenaskl // import "github.com/cockroachdb/pebble/internal/arenaskl"

import (
	"math"
	"runtime"
	"sync/atomic"
	"unsafe"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/fastrand"
)

const (
	maxHeight   = 20
	maxNodeSize = int(unsafe.Sizeof(node{}))
	linksSize   = int(unsafe.Sizeof(links{}))
	pValue      = 1 / math.E
)

// ErrRecordExists indicates that an entry with the specified key already
// exists in the skiplist. Duplicate entries are not directly supported and
// instead must be handled by the user by appending a unique version suffix to
// keys.
var ErrRecordExists = errors.New("record with this key already exists")

// Skiplist is a fast, concurrent skiplist implementation that supports forward
// and backward iteration. See batchskl.Skiplist for a non-concurrent
// skiplist. Keys and values are immutable once added to the skiplist and
// deletion is not supported. Instead, higher-level code is expected to add new
// entries that shadow existing entries and perform deletion via tombstones. It
// is up to the user to process these shadow entries and tombstones
// appropriately during retrieval.
type Skiplist struct {
	arena  *Arena
	cmp    base.Compare
	head   *node
	tail   *node
	height atomic.Uint32 // Current height. 1 <= height <= maxHeight. CAS.

	// If set to true by tests, then extra delays are added to make it easier to
	// detect unusual race conditions.
	testing bool
}

// Inserter TODO(peter)
type Inserter struct {
	spl    [maxHeight]splice
	height uint32
}

// Add TODO(peter)
func (ins *Inserter) Add(list *Skiplist, key base.InternalKey, value []byte) error {
	return list.addInternal(key, value, ins)
}

var (
	probabilities [maxHeight]uint32
)

func init() {
	// Precompute the skiplist probabilities so that only a single random number
	// needs to be generated and so that the optimal pvalue can be used (inverse
	// of Euler's number).
	p := float64(1.0)
	for i := 0; i < maxHeight; i++ {
		probabilities[i] = uint32(float64(math.MaxUint32) * p)
		p *= pValue
	}
}

// NewSkiplist constructs and initializes a new, empty skiplist. All nodes, keys,
// and values in the skiplist will be allocated from the given arena.
func NewSkiplist(arena *Arena, cmp base.Compare) *Skiplist {
	skl := &Skiplist{}
	skl.Reset(arena, cmp)
	return skl
}

// Reset the skiplist to empty and re-initialize.
func (s *Skiplist) Reset(arena *Arena, cmp base.Compare) {
	// Allocate head and tail nodes.
	head, err := newRawNode(arena, maxHeight, 0, 0)
	if err != nil {
		panic("arenaSize is not large enough to hold the head node")
	}
	head.keyOffset = 0

	tail, err := newRawNode(arena, maxHeight, 0, 0)
	if err != nil {
		panic("arenaSize is not large enough to hold the tail node")
	}
	tail.keyOffset = 0

	// Link all head/tail levels together.
	headOffset := arena.getPointerOffset(unsafe.Pointer(head))
	tailOffset := arena.getPointerOffset(unsafe.Pointer(tail))
	for i := 0; i < maxHeight; i++ {
		head.tower[i].nextOffset.Store(tailOffset)
		tail.tower[i].prevOffset.Store(headOffset)
	}

	*s = Skiplist{
		arena: arena,
		cmp:   cmp,
		head:  head,
		tail:  tail,
	}
	s.height.Store(1)
}

// Height returns the height of the highest tower within any of the nodes that
// have ever been allocated as part of this skiplist.
func (s *Skiplist) Height() uint32 { return s.height.Load() }

// Arena returns the arena backing this skiplist.
func (s *Skiplist) Arena() *Arena { return s.arena }

// Size returns the number of bytes that have allocated from the arena.
func (s *Skiplist) Size() uint32 { return s.arena.Size() }

// Add adds a new key if it does not yet exist. If the key already exists, then
// Add returns ErrRecordExists. If there isn't enough room in the arena, then
// Add returns ErrArenaFull.
func (s *Skiplist) Add(key base.InternalKey, value []byte) error {
	var ins Inserter
	return s.addInternal(key, value, &ins)
}

func (s *Skiplist) addInternal(key base.InternalKey, value []byte, ins *Inserter) error {
	if s.findSplice(key, ins) {
		// Found a matching node, but handle case where it's been deleted.
		return ErrRecordExists
	}

	if s.testing {
		// Add delay to make it easier to test race between this thread
		// and another thread that sees the intermediate state between
		// finding the splice and using it.
		runtime.Gosched()
	}

	nd, height, err := s.newNode(key, value)
	if err != nil {
		return err
	}

	ndOffset := s.arena.getPointerOffset(unsafe.Pointer(nd))

	// We always insert from the base level and up. After you add a node in base
	// level, we cannot create a node in the level above because it would have
	// discovered the node in the base level.
	var found bool
	var invalidateSplice bool
	for i := 0; i < int(height); i++ {
		prev := ins.spl[i].prev
		next := ins.spl[i].next

		if prev == nil {
			// New node increased the height of the skiplist, so assume that the
			// new level has not yet been populated.
			if next != nil {
				panic("next is expected to be nil, since prev is nil")
			}

			prev = s.head
			next = s.tail
		}

		// +----------------+     +------------+     +----------------+
		// |      prev      |     |     nd     |     |      next      |
		// | prevNextOffset |---->|            |     |                |
		// |                |<----| prevOffset |     |                |
		// |                |     | nextOffset |---->|                |
		// |                |     |            |<----| nextPrevOffset |
		// +----------------+     +------------+     +----------------+
		//
		// 1. Initialize prevOffset and nextOffset to point to prev and next.
		// 2. CAS prevNextOffset to repoint from next to nd.
		// 3. CAS nextPrevOffset to repoint from prev to nd.
		for {
			prevOffset := s.arena.getPointerOffset(unsafe.Pointer(prev))
			nextOffset := s.arena.getPointerOffset(unsafe.Pointer(next))
			nd.tower[i].init(prevOffset, nextOffset)

			// Check whether next has an updated link to prev. If it does not,
			// that can mean one of two things:
			//   1. The thread that added the next node hasn't yet had a chance
			//      to add the prev link (but will shortly).
			//   2. Another thread has added a new node between prev and next.
			nextPrevOffset := next.prevOffset(i)
			if nextPrevOffset != prevOffset {
				// Determine whether #1 or #2 is true by checking whether prev
				// is still pointing to next. As long as the atomic operations
				// have at least acquire/release semantics (no need for
				// sequential consistency), this works, as it is equivalent to
				// the "publication safety" pattern.
				prevNextOffset := prev.nextOffset(i)
				if prevNextOffset == nextOffset {
					// Ok, case #1 is true, so help the other thread along by
					// updating the next node's prev link.
					next.casPrevOffset(i, nextPrevOffset, prevOffset)
				}
			}

			if prev.casNextOffset(i, nextOffset, ndOffset) {
				// Managed to insert nd between prev and next, so update the next
				// node's prev link and go to the next level.
				if s.testing {
					// Add delay to make it easier to test race between this thread
					// and another thread that sees the intermediate state between
					// setting next and setting prev.
					runtime.Gosched()
				}

				next.casPrevOffset(i, prevOffset, ndOffset)
				break
			}

			// CAS failed. We need to recompute prev and next. It is unlikely to
			// be helpful to try to use a different level as we redo the search,
			// because it is unlikely that lots of nodes are inserted between prev
			// and next.
			prev, next, found = s.findSpliceForLevel(key, i, prev)
			if found {
				if i != 0 {
					panic("how can another thread have inserted a node at a non-base level?")
				}

				return ErrRecordExists
			}
			invalidateSplice = true
		}
	}

	// If we had to recompute the splice for a level, invalidate the entire
	// cached splice.
	if invalidateSplice {
		ins.height = 0
	} else {
		// The splice was valid. We inserted a node between spl[i].prev and
		// spl[i].next. Optimistically update spl[i].prev for use in a subsequent
		// call to add.
		for i := uint32(0); i < height; i++ {
			ins.spl[i].prev = nd
		}
	}

	return nil
}

// NewIter returns a new Iterator object. The lower and upper bound parameters
// control the range of keys the iterator will return. Specifying for nil for
// lower or upper bound disables the check for that boundary. Note that lower
// bound is not checked on {SeekGE,First} and upper bound is not check on
// {SeekLT,Last}. The user is expected to perform that check. Note that it is
// safe for an iterator to be copied by value.
func (s *Skiplist) NewIter(lower, upper []byte) *Iterator {
	it := iterPool.Get().(*Iterator)
	*it = Iterator{list: s, nd: s.head, lower: lower, upper: upper}
	return it
}

// NewFlushIter returns a new flushIterator, which is similar to an Iterator
// but also sets the current number of the bytes that have been iterated
// through.
func (s *Skiplist) NewFlushIter(bytesFlushed *uint64) base.InternalIterator {
	return &flushIterator{
		Iterator:      Iterator{list: s, nd: s.head},
		bytesIterated: bytesFlushed,
	}
}

func (s *Skiplist) newNode(
	key base.InternalKey, value []byte,
) (nd *node, height uint32, err error) {
	height = s.randomHeight()
	nd, err = newNode(s.arena, height, key, value)
	if err != nil {
		return
	}

	// Try to increase s.height via CAS.
	listHeight := s.Height()
	for height > listHeight {
		if s.height.CompareAndSwap(listHeight, height) {
			// Successfully increased skiplist.height.
			break
		}

		listHeight = s.Height()
	}

	return
}

func (s *Skiplist) randomHeight() uint32 {
	rnd := fastrand.Uint32()

	h := uint32(1)
	for h < maxHeight && rnd <= probabilities[h] {
		h++
	}

	return h
}

func (s *Skiplist) findSplice(key base.InternalKey, ins *Inserter) (found bool) {
	listHeight := s.Height()
	var level int

	prev := s.head
	if ins.height < listHeight {
		// Our cached height is less than the list height, which means there were
		// inserts that increased the height of the list. Recompute the splice from
		// scratch.
		ins.height = listHeight
		level = int(ins.height)
	} else {
		// Our cached height is equal to the list height.
		for ; level < int(listHeight); level++ {
			spl := &ins.spl[level]
			if s.getNext(spl.prev, level) != spl.next {
				// One or more nodes have been inserted between the splice at this
				// level.
				continue
			}
			if spl.prev != s.head && !s.keyIsAfterNode(spl.prev, key) {
				// Key lies before splice.
				level = int(listHeight)
				break
			}
			if spl.next != s.tail && s.keyIsAfterNode(spl.next, key) {
				// Key lies after splice.
				level = int(listHeight)
				break
			}
			// The splice brackets the key!
			prev = spl.prev
			break
		}
	}

	for level = level - 1; level >= 0; level-- {
		var next *node
		prev, next, found = s.findSpliceForLevel(key, level, prev)
		if next == nil {
			next = s.tail
		}
		ins.spl[level].init(prev, next)
	}

	return
}

func (s *Skiplist) findSpliceForLevel(
	key base.InternalKey, level int, start *node,
) (prev, next *node, found bool) {
	prev = start

	for {
		// Assume prev.key < key.
		next = s.getNext(prev, level)
		if next == s.tail {
			// Tail node, so done.
			break
		}

		offset, size := next.keyOffset, next.keySize
		nextKey := s.arena.buf[offset : offset+size]
		cmp := s.cmp(key.UserKey, nextKey)
		if cmp < 0 {
			// We are done for this level, since prev.key < key < next.key.
			break
		}
		if cmp == 0 {
			// User-key equality.
			if key.Trailer == next.keyTrailer {
				// Internal key equality.
				found = true
				break
			}
			if key.Trailer > next.keyTrailer {
				// We are done for this level, since prev.key < key < next.key.
				break
			}
		}

		// Keep moving right on this level.
		prev = next
	}

	return
}

func (s *Skiplist) keyIsAfterNode(nd *node, key base.InternalKey) bool {
	ndKey := s.arena.buf[nd.keyOffset : nd.keyOffset+nd.keySize]
	cmp := s.cmp(ndKey, key.UserKey)
	if cmp < 0 {
		return true
	}
	if cmp > 0 {
		return false
	}
	// User-key equality.
	if key.Trailer == nd.keyTrailer {
		// Internal key equality.
		return false
	}
	return key.Trailer < nd.keyTrailer
}

func (s *Skiplist) getNext(nd *node, h int) *node {
	offset := nd.tower[h].nextOffset.Load()
	return (*node)(s.arena.getPointer(offset))
}

func (s *Skiplist) getPrev(nd *node, h int) *node {
	offset := nd.tower[h].prevOffset.Load()
	return (*node)(s.arena.getPointer(offset))
}
