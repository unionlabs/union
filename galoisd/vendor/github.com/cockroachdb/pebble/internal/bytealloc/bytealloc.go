// Copyright 2016 The Cockroach Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License.

package bytealloc

import "github.com/cockroachdb/pebble/internal/rawalloc"

// An A provides chunk allocation of []byte, amortizing the overhead of each
// allocation. Because the underlying storage for the slices is shared, they
// should share a similar lifetime in order to avoid pinning large amounts of
// memory unnecessarily. The allocator itself is a []byte where cap() indicates
// the total amount of memory and len() is the amount already allocated. The
// size of the buffer to allocate from is grown exponentially when it runs out
// of room up to a maximum size (chunkAllocMaxSize).
type A []byte

const chunkAllocMinSize = 512
const chunkAllocMaxSize = 512 << 10 // 512 KB

func (a A) reserve(n int) A {
	allocSize := cap(a) * 2
	if allocSize < chunkAllocMinSize {
		allocSize = chunkAllocMinSize
	} else if allocSize > chunkAllocMaxSize {
		allocSize = chunkAllocMaxSize
	}
	if allocSize < n {
		allocSize = n
	}
	return rawalloc.New(0, allocSize)
}

// Alloc allocates a new chunk of memory with the specified length.
func (a A) Alloc(n int) (A, []byte) {
	if cap(a)-len(a) < n {
		a = a.reserve(n)
	}
	p := len(a)
	r := a[p : p+n : p+n]
	a = a[:p+n]
	return a, r
}

// Copy allocates a new chunk of memory, initializing it from src.
func (a A) Copy(src []byte) (A, []byte) {
	var alloc []byte
	a, alloc = a.Alloc(len(src))
	copy(alloc, src)
	return a, alloc
}

// Reset returns the current chunk, resetting allocated memory back to none.
// Future allocations will use memory previously allocated by previous calls to
// Alloc or Copy, so the caller must know know that none of the previously
// allocated byte slices are still in use.
func (a A) Reset() A {
	return a[:0]
}
