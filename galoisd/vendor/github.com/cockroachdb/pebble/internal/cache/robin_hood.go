// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package cache

import (
	"fmt"
	"math/bits"
	"os"
	"runtime/debug"
	"strings"
	"time"
	"unsafe"

	"github.com/cockroachdb/pebble/internal/invariants"
	"github.com/cockroachdb/pebble/internal/manual"
)

var hashSeed = uint64(time.Now().UnixNano())

// Fibonacci hash: https://probablydance.com/2018/06/16/fibonacci-hashing-the-optimization-that-the-world-forgot-or-a-better-alternative-to-integer-modulo/
func robinHoodHash(k key, shift uint32) uint32 {
	const m = 11400714819323198485
	h := hashSeed
	h ^= k.id * m
	h ^= uint64(k.fileNum.FileNum()) * m
	h ^= k.offset * m
	return uint32(h >> shift)
}

type robinHoodEntry struct {
	key key
	// Note that value may point to a Go allocated object (if the "invariants"
	// build tag was specified), even though the memory for the entry itself is
	// manually managed. This is technically a volation of the Cgo pointer rules:
	//
	//   https://golang.org/cmd/cgo/#hdr-Passing_pointers
	//
	// Specifically, Go pointers should not be stored in C allocated memory. The
	// reason for this rule is that the Go GC will not look at C allocated memory
	// to find pointers to Go objects. If the only reference to a Go object is
	// stored in C allocated memory, the object will be reclaimed. What makes
	// this "safe" is that the Cache guarantees that there are other pointers to
	// the entry and shard which will keep them alive. In particular, every Go
	// allocated entry in the cache is referenced by the shard.entries map. And
	// every shard is referenced by the Cache.shards map.
	value *entry
	// The distance the entry is from its desired position.
	dist uint32
}

type robinHoodEntries struct {
	ptr unsafe.Pointer
	len uint32
}

func newRobinHoodEntries(n uint32) robinHoodEntries {
	size := uintptr(n) * unsafe.Sizeof(robinHoodEntry{})
	return robinHoodEntries{
		ptr: unsafe.Pointer(&(manual.New(int(size)))[0]),
		len: n,
	}
}

func (e robinHoodEntries) at(i uint32) *robinHoodEntry {
	return (*robinHoodEntry)(unsafe.Pointer(uintptr(e.ptr) +
		uintptr(i)*unsafe.Sizeof(robinHoodEntry{})))
}

func (e robinHoodEntries) free() {
	size := uintptr(e.len) * unsafe.Sizeof(robinHoodEntry{})
	buf := (*[manual.MaxArrayLen]byte)(e.ptr)[:size:size]
	manual.Free(buf)
}

// robinHoodMap is an implementation of Robin Hood hashing. Robin Hood hashing
// is an open-address hash table using linear probing. The twist is that the
// linear probe distance is reduced by moving existing entries when inserting
// and deleting. This is accomplished by keeping track of how far an entry is
// from its "desired" slot (hash of key modulo number of slots). During
// insertion, if the new entry being inserted is farther from its desired slot
// than the target entry, we swap the target and new entry. This effectively
// steals from the "rich" target entry and gives to the "poor" new entry (thus
// the origin of the name).
//
// An extension over the base Robin Hood hashing idea comes from
// https://probablydance.com/2017/02/26/i-wrote-the-fastest-hashtable/. A cap
// is placed on the max distance an entry can be from its desired slot. When
// this threshold is reached during insertion, the size of the table is doubled
// and insertion is restarted. Additionally, the entries slice is given "max
// dist" extra entries on the end. The very last entry in the entries slice is
// never used and acts as a sentinel which terminates loops. The previous
// maxDist-1 entries act as the extra entries. For example, if the size of the
// table is 2, maxDist is computed as 4 and the actual size of the entry slice
// is 6.
//
//	+---+---+---+---+---+---+
//	| 0 | 1 | 2 | 3 | 4 | 5 |
//	+---+---+---+---+---+---+
//	        ^
//	       size
//
// In this scenario, the target entry for a key will always be in the range
// [0,1]. Valid entries may reside in the range [0,4] due to the linear probing
// of up to maxDist entries. The entry at index 5 will never contain a value,
// and instead acts as a sentinel (its distance is always 0). The max distance
// threshold is set to log2(num-entries). This ensures that retrieval is O(log
// N), though note that N is the number of total entries, not the count of
// valid entries.
//
// Deletion is implemented via the backward shift delete mechanism instead of
// tombstones. This preserves the performance of the table in the presence of
// deletions. See
// http://codecapsule.com/2013/11/17/robin-hood-hashing-backward-shift-deletion
// for details.
type robinHoodMap struct {
	entries robinHoodEntries
	size    uint32
	shift   uint32
	count   uint32
	maxDist uint32
}

func maxDistForSize(size uint32) uint32 {
	desired := uint32(bits.Len32(size))
	if desired < 4 {
		desired = 4
	}
	return desired
}

func newRobinHoodMap(initialCapacity int) *robinHoodMap {
	m := &robinHoodMap{}
	m.init(initialCapacity)

	// Note: this is a no-op if invariants are disabled or race is enabled.
	invariants.SetFinalizer(m, func(obj interface{}) {
		m := obj.(*robinHoodMap)
		if m.entries.ptr != nil {
			fmt.Fprintf(os.Stderr, "%p: robin-hood map not freed\n", m)
			os.Exit(1)
		}
	})
	return m
}

func (m *robinHoodMap) init(initialCapacity int) {
	if initialCapacity < 1 {
		initialCapacity = 1
	}
	targetSize := 1 << (uint(bits.Len(uint(2*initialCapacity-1))) - 1)
	m.rehash(uint32(targetSize))
}

func (m *robinHoodMap) free() {
	if m.entries.ptr != nil {
		m.entries.free()
		m.entries.ptr = nil
	}
}

func (m *robinHoodMap) rehash(size uint32) {
	oldEntries := m.entries

	m.size = size
	m.shift = uint32(64 - bits.Len32(m.size-1))
	m.maxDist = maxDistForSize(size)
	m.entries = newRobinHoodEntries(size + m.maxDist)
	m.count = 0

	for i := uint32(0); i < oldEntries.len; i++ {
		e := oldEntries.at(i)
		if e.value != nil {
			m.Put(e.key, e.value)
		}
	}

	if oldEntries.ptr != nil {
		oldEntries.free()
	}
}

// Find an entry containing the specified value. This is intended to be used
// from debug and test code.
func (m *robinHoodMap) findByValue(v *entry) *robinHoodEntry {
	for i := uint32(0); i < m.entries.len; i++ {
		e := m.entries.at(i)
		if e.value == v {
			return e
		}
	}
	return nil
}

func (m *robinHoodMap) Count() int {
	return int(m.count)
}

func (m *robinHoodMap) Put(k key, v *entry) {
	maybeExists := true
	n := robinHoodEntry{key: k, value: v, dist: 0}
	for i := robinHoodHash(k, m.shift); ; i++ {
		e := m.entries.at(i)
		if maybeExists && k == e.key {
			// Entry already exists: overwrite.
			e.value = n.value
			m.checkEntry(i)
			return
		}

		if e.value == nil {
			// Found an empty entry: insert here.
			*e = n
			m.count++
			m.checkEntry(i)
			return
		}

		if e.dist < n.dist {
			// Swap the new entry with the current entry because the current is
			// rich. We then continue to loop, looking for a new location for the
			// current entry. Note that this is also the not-found condition for
			// retrieval, which means that "k" is not present in the map. See Get().
			n, *e = *e, n
			m.checkEntry(i)
			maybeExists = false
		}

		// The new entry gradually moves away from its ideal position.
		n.dist++

		// If we've reached the max distance threshold, grow the table and restart
		// the insertion.
		if n.dist == m.maxDist {
			m.rehash(2 * m.size)
			i = robinHoodHash(n.key, m.shift) - 1
			n.dist = 0
			maybeExists = false
		}
	}
}

func (m *robinHoodMap) Get(k key) *entry {
	var dist uint32
	for i := robinHoodHash(k, m.shift); ; i++ {
		e := m.entries.at(i)
		if k == e.key {
			// Found.
			return e.value
		}
		if e.dist < dist {
			// Not found.
			return nil
		}
		dist++
	}
}

func (m *robinHoodMap) Delete(k key) {
	var dist uint32
	for i := robinHoodHash(k, m.shift); ; i++ {
		e := m.entries.at(i)
		if k == e.key {
			m.checkEntry(i)
			// We found the entry to delete. Shift the following entries backwards
			// until the next empty value or entry with a zero distance. Note that
			// empty values are guaranteed to have "dist == 0".
			m.count--
			for j := i + 1; ; j++ {
				t := m.entries.at(j)
				if t.dist == 0 {
					*e = robinHoodEntry{}
					return
				}
				e.key = t.key
				e.value = t.value
				e.dist = t.dist - 1
				e = t
				m.checkEntry(j)
			}
		}
		if dist > e.dist {
			// Not found.
			return
		}
		dist++
	}
}

func (m *robinHoodMap) checkEntry(i uint32) {
	if invariants.Enabled {
		e := m.entries.at(i)
		if e.value != nil {
			pos := robinHoodHash(e.key, m.shift)
			if (uint32(i) - pos) != e.dist {
				fmt.Fprintf(os.Stderr, "%d: invalid dist=%d, expected %d: %s\n%s",
					i, e.dist, uint32(i)-pos, e.key, debug.Stack())
				os.Exit(1)
			}
			if e.dist > m.maxDist {
				fmt.Fprintf(os.Stderr, "%d: invalid dist=%d > maxDist=%d: %s\n%s",
					i, e.dist, m.maxDist, e.key, debug.Stack())
				os.Exit(1)
			}
		}
	}
}

func (m *robinHoodMap) String() string {
	var buf strings.Builder
	fmt.Fprintf(&buf, "count: %d\n", m.count)
	for i := uint32(0); i < m.entries.len; i++ {
		e := m.entries.at(i)
		if e.value != nil {
			fmt.Fprintf(&buf, "%d: [%s,%p,%d]\n", i, e.key, e.value, e.dist)
		}
	}
	return buf.String()
}
