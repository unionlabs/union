// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package sstable

import (
	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/cache"
)

// A bufferHandle is a handle to manually-managed memory. The handle may point
// to a block in the block cache (h.Get() != nil), or a buffer that exists
// outside the block cache allocated from a BufferPool (b.Valid()).
type bufferHandle struct {
	h cache.Handle
	b Buf
}

// Get retrieves the underlying buffer referenced by the handle.
func (bh bufferHandle) Get() []byte {
	if v := bh.h.Get(); v != nil {
		return v
	} else if bh.b.p != nil {
		return bh.b.p.pool[bh.b.i].b
	}
	return nil
}

// Release releases the buffer, either back to the block cache or BufferPool.
func (bh bufferHandle) Release() {
	bh.h.Release()
	bh.b.Release()
}

// A BufferPool holds a pool of buffers for holding sstable blocks. An initial
// size of the pool is provided on Init, but a BufferPool will grow to meet the
// largest working set size. It'll never shrink. When a buffer is released, the
// BufferPool recycles the buffer for future allocations.
//
// A BufferPool should only be used for short-lived allocations with
// well-understood working set sizes to avoid excessive memory consumption.
//
// BufferPool is not thread-safe.
type BufferPool struct {
	// pool contains all the buffers held by the pool, including buffers that
	// are in-use. For every i < len(pool): pool[i].v is non-nil.
	pool []allocedBuffer
}

type allocedBuffer struct {
	v *cache.Value
	// b holds the current byte slice. It's backed by v, but may be a subslice
	// of v's memory while the buffer is in-use [ len(b) ≤ len(v.Buf()) ].
	//
	// If the buffer is not currently in-use, b is nil. When being recycled, the
	// BufferPool.Alloc will reset b to be a subslice of v.Buf().
	b []byte
}

// Init initializes the pool with an initial working set buffer size of
// `initialSize`.
func (p *BufferPool) Init(initialSize int) {
	*p = BufferPool{
		pool: make([]allocedBuffer, 0, initialSize),
	}
}

// initPreallocated is like Init but for internal sstable package use in
// instances where a pre-allocated slice of []allocedBuffer already exists. It's
// used to avoid an extra allocation initializing BufferPool.pool.
func (p *BufferPool) initPreallocated(pool []allocedBuffer) {
	*p = BufferPool{
		pool: pool[:0],
	}
}

// Release releases all buffers held by the pool and resets the pool to an
// uninitialized state.
func (p *BufferPool) Release() {
	for i := range p.pool {
		if p.pool[i].b != nil {
			panic(errors.AssertionFailedf("Release called on a BufferPool with in-use buffers"))
		}
		cache.Free(p.pool[i].v)
	}
	*p = BufferPool{}
}

// Alloc allocates a new buffer of size n. If the pool already holds a buffer at
// least as large as n, the pooled buffer is used instead.
//
// Alloc is O(MAX(N,M)) where N is the largest number of concurrently in-use
// buffers allocated and M is the initialSize passed to Init.
func (p *BufferPool) Alloc(n int) Buf {
	unusableBufferIdx := -1
	for i := 0; i < len(p.pool); i++ {
		if p.pool[i].b == nil {
			if len(p.pool[i].v.Buf()) >= n {
				p.pool[i].b = p.pool[i].v.Buf()[:n]
				return Buf{p: p, i: i}
			}
			unusableBufferIdx = i
		}
	}

	// If we would need to grow the size of the pool to allocate another buffer,
	// but there was a slot available occupied by a buffer that's just too
	// small, replace the too-small buffer.
	if len(p.pool) == cap(p.pool) && unusableBufferIdx >= 0 {
		i := unusableBufferIdx
		cache.Free(p.pool[i].v)
		p.pool[i].v = cache.Alloc(n)
		p.pool[i].b = p.pool[i].v.Buf()
		return Buf{p: p, i: i}
	}

	// Allocate a new buffer.
	v := cache.Alloc(n)
	p.pool = append(p.pool, allocedBuffer{v: v, b: v.Buf()[:n]})
	return Buf{p: p, i: len(p.pool) - 1}
}

// A Buf holds a reference to a manually-managed, pooled byte buffer.
type Buf struct {
	p *BufferPool
	// i holds the index into p.pool where the buffer may be found. This scheme
	// avoids needing to allocate the handle to the buffer on the heap at the
	// cost of copying two words instead of one.
	i int
}

// Valid returns true if the buf holds a valid buffer.
func (b Buf) Valid() bool {
	return b.p != nil
}

// Release releases the buffer back to the pool.
func (b *Buf) Release() {
	if b.p == nil {
		return
	}
	// Clear the allocedBuffer's byte slice. This signals the allocated buffer
	// is no longer in use and a future call to BufferPool.Alloc may reuse this
	// buffer.
	b.p.pool[b.i].b = nil
	b.p = nil
}
