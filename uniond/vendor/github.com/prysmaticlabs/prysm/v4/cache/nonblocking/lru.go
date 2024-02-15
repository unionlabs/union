// Copyright (c) HashiCorp, Inc.
// SPDX-License-Identifier: MPL-2.0
package nonblocking

import (
	"errors"
	"sync"
)

// EvictCallback is used to get a callback when a cache entry is evicted
type EvictCallback[K comparable, V any] func(key K, value V)

// LRU implements a non-thread safe fixed size LRU cache
type LRU[K comparable, V any] struct {
	itemsLock     sync.RWMutex
	evictListLock sync.RWMutex
	size          int
	evictList     *lruList[K, V]
	items         map[K]*entry[K, V]
	onEvict       EvictCallback[K, V]
	getChan       chan *entry[K, V]
}

// NewLRU constructs an LRU of the given size
func NewLRU[K comparable, V any](size int, onEvict EvictCallback[K, V]) (*LRU[K, V], error) {
	if size <= 0 {
		return nil, errors.New("must provide a positive size")
	}
	// Initialize the channel buffer size as being 10% of the cache size.
	chanSize := size / 10

	c := &LRU[K, V]{
		size:      size,
		evictList: newList[K, V](),
		items:     make(map[K]*entry[K, V]),
		onEvict:   onEvict,
		getChan:   make(chan *entry[K, V], chanSize),
	}
	// Spin off separate go-routine to handle evict list
	// operations.
	go c.handleGetRequests()
	return c, nil
}

// Add adds a value to the cache. Returns true if an eviction occurred.
func (c *LRU[K, V]) Add(key K, value V) (evicted bool) {
	// Check for existing item
	c.itemsLock.RLock()
	if ent, ok := c.items[key]; ok {
		c.itemsLock.RUnlock()

		c.evictListLock.Lock()
		c.evictList.moveToFront(ent)
		c.evictListLock.Unlock()
		ent.value = value
		return false
	}
	c.itemsLock.RUnlock()

	// Add new item
	c.evictListLock.Lock()
	ent := c.evictList.pushFront(key, value)
	c.evictListLock.Unlock()

	c.itemsLock.Lock()
	c.items[key] = ent
	c.itemsLock.Unlock()

	c.evictListLock.RLock()
	evict := c.evictList.length() > c.size
	c.evictListLock.RUnlock()

	// Verify size not exceeded
	if evict {
		c.removeOldest()
	}
	return evict
}

// Get looks up a key's value from the cache.
func (c *LRU[K, V]) Get(key K) (value V, ok bool) {
	c.itemsLock.RLock()
	if ent, ok := c.items[key]; ok {
		c.itemsLock.RUnlock()

		// Make this get function non-blocking for multiple readers.
		c.getChan <- ent
		return ent.value, true
	}
	c.itemsLock.RUnlock()
	return
}

// Len returns the number of items in the cache.
func (c *LRU[K, V]) Len() int {
	c.evictListLock.RLock()
	defer c.evictListLock.RUnlock()
	return c.evictList.length()
}

// Resize changes the cache size.
func (c *LRU[K, V]) Resize(size int) (evicted int) {
	diff := c.Len() - size
	if diff < 0 {
		diff = 0
	}
	for i := 0; i < diff; i++ {
		c.removeOldest()
	}
	c.size = size
	return diff
}

// removeOldest removes the oldest item from the cache.
func (c *LRU[K, V]) removeOldest() {
	c.evictListLock.RLock()
	if ent := c.evictList.back(); ent != nil {
		c.evictListLock.RUnlock()
		c.removeElement(ent)
		return
	}
	c.evictListLock.RUnlock()
}

// removeElement is used to remove a given list element from the cache
func (c *LRU[K, V]) removeElement(e *entry[K, V]) {
	c.evictListLock.Lock()
	c.evictList.remove(e)
	c.evictListLock.Unlock()

	c.itemsLock.Lock()
	delete(c.items, e.key)
	c.itemsLock.Unlock()
	if c.onEvict != nil {
		c.onEvict(e.key, e.value)
	}
}

func (c *LRU[K, V]) handleGetRequests() {
	for {
		entry := <-c.getChan
		c.evictListLock.Lock()
		c.evictList.moveToFront(entry)
		c.evictListLock.Unlock()
	}
}
