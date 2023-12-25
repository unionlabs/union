// Copyright (c) 2021 Oasis Labs Inc. All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
// 1. Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright
// notice, this list of conditions and the following disclaimer in the
// documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS
// IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED
// TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A
// PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED
// TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

package cache

import (
	"container/list"
	"sync"

	"github.com/oasisprotocol/curve25519-voi/curve"
	"github.com/oasisprotocol/curve25519-voi/primitives/ed25519"
)

type lruCache struct {
	sync.Mutex

	store map[curve.CompressedEdwardsY]*lruEntry
	list  list.List

	capacity int
}

type lruEntry struct {
	publicKey *ed25519.ExpandedPublicKey
	element   *list.Element
}

func (cache *lruCache) getLocked(publicKey *curve.CompressedEdwardsY) *ed25519.ExpandedPublicKey {
	entry := cache.store[*publicKey]
	if entry == nil {
		return nil
	}

	cache.list.Remove(entry.element)
	entry.element = cache.list.PushFront(entry)

	return entry.publicKey
}

func (cache *lruCache) Get(publicKey *curve.CompressedEdwardsY) *ed25519.ExpandedPublicKey {
	cache.Lock()
	defer cache.Unlock()

	return cache.getLocked(publicKey)
}

func (cache *lruCache) Put(publicKey *curve.CompressedEdwardsY, expanded *ed25519.ExpandedPublicKey) {
	cache.Lock()
	defer cache.Unlock()

	// Do a lookup to see if the entry already exists.
	if entry := cache.getLocked(publicKey); entry != nil {
		// Already in the cache, and now marked as most-recently-used.
		return
	}

	entry := &lruEntry{
		publicKey: expanded,
	}

	// Evict the Least-Recently-Used entry if any.
	if l := cache.list.Len(); l == cache.capacity {
		element := cache.list.Back()
		entryValue := cache.list.Remove(element)

		entry := entryValue.(*lruEntry)
		delete(cache.store, entry.publicKey.CompressedY())
	}

	cache.store[*publicKey] = entry
	entry.element = cache.list.PushFront(entry)
}

// NewLRUCache creates a new cache with a Least-Recently-Used replacement
// policy.  Cache instances returned are thread-safe.
func NewLRUCache(capacity int) Cache {
	if capacity <= 0 {
		panic("edcache: capacity must be < 0")
	}

	return &lruCache{
		store:    make(map[curve.CompressedEdwardsY]*lruEntry),
		capacity: capacity,
	}
}
