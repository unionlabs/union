// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package cache

import "sync/atomic"

type entryType int8

const (
	etTest entryType = iota
	etCold
	etHot
)

func (p entryType) String() string {
	switch p {
	case etTest:
		return "test"
	case etCold:
		return "cold"
	case etHot:
		return "hot"
	}
	return "unknown"
}

// entry holds the metadata for a cache entry. The memory for an entry is
// allocated from manually managed memory.
//
// Using manual memory management for entries is technically a volation of the
// Cgo pointer rules:
//
//	https://golang.org/cmd/cgo/#hdr-Passing_pointers
//
// Specifically, Go pointers should not be stored in C allocated memory. The
// reason for this rule is that the Go GC will not look at C allocated memory
// to find pointers to Go objects. If the only reference to a Go object is
// stored in C allocated memory, the object will be reclaimed. The shard field
// of the entry struct points to a Go allocated object, thus the
// violation. What makes this "safe" is that the Cache guarantees that there
// are other pointers to the shard which will keep it alive.
type entry struct {
	key key
	// The value associated with the entry. The entry holds a reference on the
	// value which is maintained by entry.setValue().
	val       *Value
	blockLink struct {
		next *entry
		prev *entry
	}
	fileLink struct {
		next *entry
		prev *entry
	}
	size  int64
	ptype entryType
	// referenced is atomically set to indicate that this entry has been accessed
	// since the last time one of the clock hands swept it.
	referenced atomic.Bool
	shard      *shard
	// Reference count for the entry. The entry is freed when the reference count
	// drops to zero.
	ref refcnt
}

func newEntry(s *shard, key key, size int64) *entry {
	e := entryAllocNew()
	*e = entry{
		key:   key,
		size:  size,
		ptype: etCold,
		shard: s,
	}
	e.blockLink.next = e
	e.blockLink.prev = e
	e.fileLink.next = e
	e.fileLink.prev = e
	e.ref.init(1)
	return e
}

func (e *entry) free() {
	e.setValue(nil)
	*e = entry{}
	entryAllocFree(e)
}

func (e *entry) next() *entry {
	if e == nil {
		return nil
	}
	return e.blockLink.next
}

func (e *entry) prev() *entry {
	if e == nil {
		return nil
	}
	return e.blockLink.prev
}

func (e *entry) link(s *entry) {
	s.blockLink.prev = e.blockLink.prev
	s.blockLink.prev.blockLink.next = s
	s.blockLink.next = e
	s.blockLink.next.blockLink.prev = s
}

func (e *entry) unlink() *entry {
	next := e.blockLink.next
	e.blockLink.prev.blockLink.next = e.blockLink.next
	e.blockLink.next.blockLink.prev = e.blockLink.prev
	e.blockLink.prev = e
	e.blockLink.next = e
	return next
}

func (e *entry) linkFile(s *entry) {
	s.fileLink.prev = e.fileLink.prev
	s.fileLink.prev.fileLink.next = s
	s.fileLink.next = e
	s.fileLink.next.fileLink.prev = s
}

func (e *entry) unlinkFile() *entry {
	next := e.fileLink.next
	e.fileLink.prev.fileLink.next = e.fileLink.next
	e.fileLink.next.fileLink.prev = e.fileLink.prev
	e.fileLink.prev = e
	e.fileLink.next = e
	return next
}

func (e *entry) setValue(v *Value) {
	if v != nil {
		v.acquire()
	}
	old := e.val
	e.val = v
	old.release()
}

func (e *entry) peekValue() *Value {
	return e.val
}

func (e *entry) acquireValue() *Value {
	v := e.val
	if v != nil {
		v.acquire()
	}
	return v
}
