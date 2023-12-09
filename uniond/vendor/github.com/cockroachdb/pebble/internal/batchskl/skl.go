/*
 * Copyright 2017 Dgraph Labs, Inc. and Contributors
 * Modifications copyright (C) 2017 Andy Kimball and Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License")
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

/*
Further adapted from arenaskl: https://github.com/andy-kimball/arenaskl

Key differences:
- Removed support for deletion.
- Removed support for concurrency.
- External storage of keys.
- Node storage grows to an arbitrary size.
*/

package batchskl // import "github.com/cockroachdb/pebble/internal/batchskl"

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"math"
	"time"
	"unsafe"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/constants"
	"golang.org/x/exp/rand"
)

const (
	maxHeight    = 20
	maxNodeSize  = uint64(unsafe.Sizeof(node{}))
	linksSize    = uint64(unsafe.Sizeof(links{}))
	maxNodesSize = constants.MaxUint32OrInt
)

var (
	// ErrExists indicates that a duplicate record was inserted. This should never
	// happen for normal usage of batchskl as every key should have a unique
	// sequence number.
	ErrExists = errors.New("record with this key already exists")

	// ErrTooManyRecords is a sentinel error returned when the size of the raw
	// nodes slice exceeds the maximum allowed size (currently 1 << 32 - 1). This
	// corresponds to ~117 M skiplist entries.
	ErrTooManyRecords = errors.New("too many records")
)

type links struct {
	next uint32
	prev uint32
}

type node struct {
	// The offset of the start of the record in the storage.
	offset uint32
	// The offset of the start and end of the key in storage.
	keyStart uint32
	keyEnd   uint32
	// A fixed 8-byte abbreviation of the key, used to avoid retrieval of the key
	// during seek operations. The key retrieval can be expensive purely due to
	// cache misses while the abbreviatedKey stored here will be in the same
	// cache line as the key and the links making accessing and comparing against
	// it almost free.
	abbreviatedKey uint64
	// Most nodes do not need to use the full height of the link tower, since the
	// probability of each successive level decreases exponentially. Because
	// these elements are never accessed, they do not need to be allocated.
	// Therefore, when a node is allocated, its memory footprint is deliberately
	// truncated to not include unneeded link elements.
	links [maxHeight]links
}

// Skiplist is a fast, non-cocnurrent skiplist implementation that supports
// forward and backward iteration. See arenaskl.Skiplist for a concurrent
// skiplist. Keys and values are stored externally from the skiplist via the
// Storage interface. Deletion is not supported. Instead, higher-level code is
// expected to perform deletion via tombstones and needs to process those
// tombstones appropriately during retrieval operations.
type Skiplist struct {
	storage        *[]byte
	cmp            base.Compare
	abbreviatedKey base.AbbreviatedKey
	nodes          []byte
	head           uint32
	tail           uint32
	height         uint32 // Current height: 1 <= height <= maxHeight
	rand           rand.PCGSource
}

var (
	probabilities [maxHeight]uint32
)

func init() {
	const pValue = 1 / math.E

	// Precompute the skiplist probabilities so that only a single random number
	// needs to be generated and so that the optimal pvalue can be used (inverse
	// of Euler's number).
	p := float64(1.0)
	for i := 0; i < maxHeight; i++ {
		probabilities[i] = uint32(float64(math.MaxUint32) * p)
		p *= pValue
	}
}

// NewSkiplist constructs and initializes a new, empty skiplist.
func NewSkiplist(storage *[]byte, cmp base.Compare, abbreviatedKey base.AbbreviatedKey) *Skiplist {
	s := &Skiplist{}
	s.Init(storage, cmp, abbreviatedKey)
	return s
}

// Reset the fields in the skiplist for reuse.
func (s *Skiplist) Reset() {
	*s = Skiplist{
		nodes:  s.nodes[:0],
		height: 1,
	}
	const batchMaxRetainedSize = 1 << 20 // 1 MB
	if cap(s.nodes) > batchMaxRetainedSize {
		s.nodes = nil
	}
}

// Init the skiplist to empty and re-initialize.
func (s *Skiplist) Init(storage *[]byte, cmp base.Compare, abbreviatedKey base.AbbreviatedKey) {
	*s = Skiplist{
		storage:        storage,
		cmp:            cmp,
		abbreviatedKey: abbreviatedKey,
		nodes:          s.nodes[:0],
		height:         1,
	}
	s.rand.Seed(uint64(time.Now().UnixNano()))

	const initBufSize = 256
	if cap(s.nodes) < initBufSize {
		s.nodes = make([]byte, 0, initBufSize)
	}

	// Allocate head and tail nodes. While allocating a new node can fail, in the
	// context of initializing the skiplist we consider it unrecoverable.
	var err error
	s.head, err = s.newNode(maxHeight, 0, 0, 0, 0)
	if err != nil {
		panic(err)
	}
	s.tail, err = s.newNode(maxHeight, 0, 0, 0, 0)
	if err != nil {
		panic(err)
	}

	// Link all head/tail levels together.
	headNode := s.node(s.head)
	tailNode := s.node(s.tail)
	for i := uint32(0); i < maxHeight; i++ {
		headNode.links[i].next = s.tail
		tailNode.links[i].prev = s.head
	}
}

// Add adds a new key to the skiplist if it does not yet exist. If the record
// already exists, then Add returns ErrRecordExists.
func (s *Skiplist) Add(keyOffset uint32) error {
	data := (*s.storage)[keyOffset+1:]
	v, n := binary.Uvarint(data)
	if n <= 0 {
		return errors.Errorf("corrupted batch entry: %d", errors.Safe(keyOffset))
	}
	data = data[n:]
	if v > uint64(len(data)) {
		return errors.Errorf("corrupted batch entry: %d", errors.Safe(keyOffset))
	}
	keyStart := 1 + keyOffset + uint32(n)
	keyEnd := keyStart + uint32(v)
	key := data[:v]
	abbreviatedKey := s.abbreviatedKey(key)

	// spl holds the list of next and previous links for each level in the
	// skiplist indicating where the new node will be inserted.
	var spl [maxHeight]splice

	// Fast-path for in-order insertion of keys: compare the new key against the
	// last key.
	prev := s.getPrev(s.tail, 0)
	if prevNode := s.node(prev); prev == s.head ||
		abbreviatedKey > prevNode.abbreviatedKey ||
		(abbreviatedKey == prevNode.abbreviatedKey &&
			s.cmp(key, (*s.storage)[prevNode.keyStart:prevNode.keyEnd]) > 0) {
		for level := uint32(0); level < s.height; level++ {
			spl[level].prev = s.getPrev(s.tail, level)
			spl[level].next = s.tail
		}
	} else {
		s.findSplice(key, abbreviatedKey, &spl)
	}

	height := s.randomHeight()
	// Increase s.height as necessary.
	for ; s.height < height; s.height++ {
		spl[s.height].next = s.tail
		spl[s.height].prev = s.head
	}

	// We always insert from the base level and up. After you add a node in base
	// level, we cannot create a node in the level above because it would have
	// discovered the node in the base level.
	nd, err := s.newNode(height, keyOffset, keyStart, keyEnd, abbreviatedKey)
	if err != nil {
		return err
	}
	newNode := s.node(nd)
	for level := uint32(0); level < height; level++ {
		next := spl[level].next
		prev := spl[level].prev
		newNode.links[level].next = next
		newNode.links[level].prev = prev
		s.node(next).links[level].prev = nd
		s.node(prev).links[level].next = nd
	}

	return nil
}

// NewIter returns a new Iterator object. The lower and upper bound parameters
// control the range of keys the iterator will return. Specifying for nil for
// lower or upper bound disables the check for that boundary. Note that lower
// bound is not checked on {SeekGE,First} and upper bound is not check on
// {SeekLT,Last}. The user is expected to perform that check. Note that it is
// safe for an iterator to be copied by value.
func (s *Skiplist) NewIter(lower, upper []byte) Iterator {
	return Iterator{list: s, lower: lower, upper: upper}
}

func (s *Skiplist) newNode(
	height,
	offset, keyStart, keyEnd uint32, abbreviatedKey uint64,
) (uint32, error) {
	if height < 1 || height > maxHeight {
		panic("height cannot be less than one or greater than the max height")
	}

	unusedSize := uint64(maxHeight-int(height)) * linksSize
	nodeOffset, err := s.alloc(uint32(maxNodeSize - unusedSize))
	if err != nil {
		return 0, err
	}
	nd := s.node(nodeOffset)

	nd.offset = offset
	nd.keyStart = keyStart
	nd.keyEnd = keyEnd
	nd.abbreviatedKey = abbreviatedKey
	return nodeOffset, nil
}

func (s *Skiplist) alloc(size uint32) (uint32, error) {
	offset := uint64(len(s.nodes))

	// We only have a need for memory up to offset + size, but we never want
	// to allocate a node whose tail points into unallocated memory.
	minAllocSize := offset + maxNodeSize
	if uint64(cap(s.nodes)) < minAllocSize {
		allocSize := uint64(cap(s.nodes)) * 2
		if allocSize < minAllocSize {
			allocSize = minAllocSize
		}
		// Cap the allocation at the max allowed size to avoid wasted capacity.
		if allocSize > maxNodesSize {
			// The new record may still not fit within the allocation, in which case
			// we return early with an error. This avoids the panic below when we
			// resize the slice. It also avoids the allocation and copy.
			if uint64(offset)+uint64(size) > maxNodesSize {
				return 0, errors.Wrapf(ErrTooManyRecords,
					"alloc of new record (size=%d) would overflow uint32 (current size=%d)",
					uint64(offset)+uint64(size), offset,
				)
			}
			allocSize = maxNodesSize
		}
		tmp := make([]byte, len(s.nodes), allocSize)
		copy(tmp, s.nodes)
		s.nodes = tmp
	}

	newSize := uint32(offset) + size
	s.nodes = s.nodes[:newSize]
	return uint32(offset), nil
}

func (s *Skiplist) node(offset uint32) *node {
	return (*node)(unsafe.Pointer(&s.nodes[offset]))
}

func (s *Skiplist) randomHeight() uint32 {
	rnd := uint32(s.rand.Uint64())
	h := uint32(1)
	for h < maxHeight && rnd <= probabilities[h] {
		h++
	}
	return h
}

func (s *Skiplist) findSplice(key []byte, abbreviatedKey uint64, spl *[maxHeight]splice) {
	prev := s.head

	for level := s.height - 1; ; level-- {
		// The code in this loop is the same as findSpliceForLevel(). For some
		// reason, calling findSpliceForLevel() here is much much slower than the
		// inlined code below. The excess time is also caught up in the final
		// return statement which makes little sense. Revisit when in go1.14 or
		// later if inlining improves.

		next := s.getNext(prev, level)
		for next != s.tail {
			// Assume prev.key < key.
			nextNode := s.node(next)
			nextAbbreviatedKey := nextNode.abbreviatedKey
			if abbreviatedKey < nextAbbreviatedKey {
				// We are done for this level, since prev.key < key < next.key.
				break
			}
			if abbreviatedKey == nextAbbreviatedKey {
				if s.cmp(key, (*s.storage)[nextNode.keyStart:nextNode.keyEnd]) <= 0 {
					// We are done for this level, since prev.key < key <= next.key.
					break
				}
			}

			// Keep moving right on this level.
			prev = next
			next = nextNode.links[level].next
		}

		spl[level].prev = prev
		spl[level].next = next
		if level == 0 {
			break
		}
	}
}

func (s *Skiplist) findSpliceForLevel(
	key []byte, abbreviatedKey uint64, level, start uint32,
) (prev, next uint32) {
	prev = start
	next = s.getNext(prev, level)

	for next != s.tail {
		// Assume prev.key < key.
		nextNode := s.node(next)
		nextAbbreviatedKey := nextNode.abbreviatedKey
		if abbreviatedKey < nextAbbreviatedKey {
			// We are done for this level, since prev.key < key < next.key.
			break
		}
		if abbreviatedKey == nextAbbreviatedKey {
			if s.cmp(key, (*s.storage)[nextNode.keyStart:nextNode.keyEnd]) <= 0 {
				// We are done for this level, since prev.key < key < next.key.
				break
			}
		}

		// Keep moving right on this level.
		prev = next
		next = nextNode.links[level].next
	}

	return
}

func (s *Skiplist) getKey(nd uint32) base.InternalKey {
	n := s.node(nd)
	kind := base.InternalKeyKind((*s.storage)[n.offset])
	key := (*s.storage)[n.keyStart:n.keyEnd]
	return base.MakeInternalKey(key, uint64(n.offset)|base.InternalKeySeqNumBatch, kind)
}

func (s *Skiplist) getNext(nd, h uint32) uint32 {
	return s.node(nd).links[h].next
}

func (s *Skiplist) getPrev(nd, h uint32) uint32 {
	return s.node(nd).links[h].prev
}

func (s *Skiplist) debug() string {
	var buf bytes.Buffer
	for level := uint32(0); level < s.height; level++ {
		var count int
		for nd := s.head; nd != s.tail; nd = s.getNext(nd, level) {
			count++
		}
		fmt.Fprintf(&buf, "%d: %d\n", level, count)
	}
	return buf.String()
}

// Silence unused warning.
var _ = (*Skiplist).debug
