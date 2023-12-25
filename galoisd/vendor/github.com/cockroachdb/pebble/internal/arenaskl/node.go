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

package arenaskl

import (
	"math"
	"sync/atomic"

	"github.com/cockroachdb/pebble/internal/base"
)

// MaxNodeSize returns the maximum space needed for a node with the specified
// key and value sizes. This could overflow a uint32, which is why a uint64
// is used here. If a key/value overflows a uint32, it should not be added to
// the skiplist.
func MaxNodeSize(keySize, valueSize uint32) uint64 {
	const maxPadding = nodeAlignment - 1
	return uint64(maxNodeSize) + uint64(keySize) + uint64(valueSize) + maxPadding
}

type links struct {
	nextOffset atomic.Uint32
	prevOffset atomic.Uint32
}

func (l *links) init(prevOffset, nextOffset uint32) {
	l.nextOffset.Store(nextOffset)
	l.prevOffset.Store(prevOffset)
}

type node struct {
	// Immutable fields, so no need to lock to access key.
	keyOffset  uint32
	keySize    uint32
	keyTrailer uint64
	valueSize  uint32
	allocSize  uint32

	// Most nodes do not need to use the full height of the tower, since the
	// probability of each successive level decreases exponentially. Because
	// these elements are never accessed, they do not need to be allocated.
	// Therefore, when a node is allocated in the arena, its memory footprint
	// is deliberately truncated to not include unneeded tower elements.
	//
	// All accesses to elements should use CAS operations, with no need to lock.
	tower [maxHeight]links
}

func newNode(
	arena *Arena, height uint32, key base.InternalKey, value []byte,
) (nd *node, err error) {
	if height < 1 || height > maxHeight {
		panic("height cannot be less than one or greater than the max height")
	}
	keySize := len(key.UserKey)
	if int64(keySize) > math.MaxUint32 {
		panic("key is too large")
	}
	valueSize := len(value)
	if int64(len(value)) > math.MaxUint32 {
		panic("value is too large")
	}
	if int64(len(value))+int64(keySize)+int64(maxNodeSize) > math.MaxUint32 {
		panic("combined key and value size is too large")
	}

	nd, err = newRawNode(arena, height, uint32(keySize), uint32(valueSize))
	if err != nil {
		return
	}
	nd.keyTrailer = key.Trailer
	copy(nd.getKeyBytes(arena), key.UserKey)
	copy(nd.getValue(arena), value)
	return
}

func newRawNode(arena *Arena, height uint32, keySize, valueSize uint32) (nd *node, err error) {
	// Compute the amount of the tower that will never be used, since the height
	// is less than maxHeight.
	unusedSize := uint32((maxHeight - int(height)) * linksSize)
	nodeSize := uint32(maxNodeSize) - unusedSize

	nodeOffset, allocSize, err := arena.alloc(nodeSize+keySize+valueSize, nodeAlignment, unusedSize)
	if err != nil {
		return
	}

	nd = (*node)(arena.getPointer(nodeOffset))
	nd.keyOffset = nodeOffset + nodeSize
	nd.keySize = keySize
	nd.valueSize = valueSize
	nd.allocSize = allocSize
	return
}

func (n *node) getKeyBytes(arena *Arena) []byte {
	return arena.getBytes(n.keyOffset, n.keySize)
}

func (n *node) getValue(arena *Arena) []byte {
	return arena.getBytes(n.keyOffset+n.keySize, uint32(n.valueSize))
}

func (n *node) nextOffset(h int) uint32 {
	return n.tower[h].nextOffset.Load()
}

func (n *node) prevOffset(h int) uint32 {
	return n.tower[h].prevOffset.Load()
}

func (n *node) casNextOffset(h int, old, val uint32) bool {
	return n.tower[h].nextOffset.CompareAndSwap(old, val)
}

func (n *node) casPrevOffset(h int, old, val uint32) bool {
	return n.tower[h].prevOffset.CompareAndSwap(old, val)
}
