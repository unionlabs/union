// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.
//
//go:build (!invariants && !tracing) || race
// +build !invariants,!tracing race

package cache

import (
	"runtime"
	"sync"
	"unsafe"

	"github.com/cockroachdb/pebble/internal/invariants"
	"github.com/cockroachdb/pebble/internal/manual"
)

const (
	entrySize            = int(unsafe.Sizeof(entry{}))
	entryAllocCacheLimit = 128
	// Avoid using runtime.SetFinalizer in race builds as finalizers tickle a bug
	// in the Go race detector in go1.15 and earlier versions. This requires that
	// entries are Go allocated rather than manually allocated.
	//
	// If cgo is disabled we need to allocate the entries using the Go allocator
	// and is violates the Go GC rules to put Go pointers (such as the entry
	// pointer fields) into untyped memory (i.e. a []byte).
	entriesGoAllocated = invariants.RaceEnabled || !cgoEnabled
)

var entryAllocPool = sync.Pool{
	New: func() interface{} {
		return newEntryAllocCache()
	},
}

func entryAllocNew() *entry {
	a := entryAllocPool.Get().(*entryAllocCache)
	e := a.alloc()
	entryAllocPool.Put(a)
	return e
}

func entryAllocFree(e *entry) {
	a := entryAllocPool.Get().(*entryAllocCache)
	a.free(e)
	entryAllocPool.Put(a)
}

type entryAllocCache struct {
	entries []*entry
}

func newEntryAllocCache() *entryAllocCache {
	c := &entryAllocCache{}
	if !entriesGoAllocated {
		// Note the use of a "real" finalizer here (as opposed to a build tag-gated
		// no-op finalizer). Without the finalizer, objects released from the pool
		// and subsequently GC'd by the Go runtime would fail to have their manually
		// allocated memory freed, which results in a memory leak.
		// lint:ignore SetFinalizer
		runtime.SetFinalizer(c, freeEntryAllocCache)
	}
	return c
}

func freeEntryAllocCache(obj interface{}) {
	c := obj.(*entryAllocCache)
	for i, e := range c.entries {
		c.dealloc(e)
		c.entries[i] = nil
	}
}

func (c *entryAllocCache) alloc() *entry {
	n := len(c.entries)
	if n == 0 {
		if entriesGoAllocated {
			return &entry{}
		}
		b := manual.New(entrySize)
		return (*entry)(unsafe.Pointer(&b[0]))
	}
	e := c.entries[n-1]
	c.entries = c.entries[:n-1]
	return e
}

func (c *entryAllocCache) dealloc(e *entry) {
	if !entriesGoAllocated {
		buf := (*[manual.MaxArrayLen]byte)(unsafe.Pointer(e))[:entrySize:entrySize]
		manual.Free(buf)
	}
}

func (c *entryAllocCache) free(e *entry) {
	if len(c.entries) == entryAllocCacheLimit {
		c.dealloc(e)
		return
	}
	c.entries = append(c.entries, e)
}
