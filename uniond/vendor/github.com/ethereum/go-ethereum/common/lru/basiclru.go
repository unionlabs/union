// Copyright 2022 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

// Package lru implements generically-typed LRU caches.
package lru

// BasicLRU is a simple LRU cache.
//
// This type is not safe for concurrent use.
// The zero value is not valid, instances must be created using NewCache.
type BasicLRU[K comparable, V any] struct {
	list  *list[K]
	items map[K]cacheItem[K, V]
	cap   int
}

type cacheItem[K any, V any] struct {
	elem  *listElem[K]
	value V
}

// NewBasicLRU creates a new LRU cache.
func NewBasicLRU[K comparable, V any](capacity int) BasicLRU[K, V] {
	if capacity <= 0 {
		capacity = 1
	}
	c := BasicLRU[K, V]{
		items: make(map[K]cacheItem[K, V]),
		list:  newList[K](),
		cap:   capacity,
	}
	return c
}

// Add adds a value to the cache. Returns true if an item was evicted to store the new item.
func (c *BasicLRU[K, V]) Add(key K, value V) (evicted bool) {
	item, ok := c.items[key]
	if ok {
		// Already exists in cache.
		item.value = value
		c.items[key] = item
		c.list.moveToFront(item.elem)
		return false
	}

	var elem *listElem[K]
	if c.Len() >= c.cap {
		elem = c.list.removeLast()
		delete(c.items, elem.v)
		evicted = true
	} else {
		elem = new(listElem[K])
	}

	// Store the new item.
	// Note that, if another item was evicted, we re-use its list element here.
	elem.v = key
	c.items[key] = cacheItem[K, V]{elem, value}
	c.list.pushElem(elem)
	return evicted
}

// Contains reports whether the given key exists in the cache.
func (c *BasicLRU[K, V]) Contains(key K) bool {
	_, ok := c.items[key]
	return ok
}

// Get retrieves a value from the cache. This marks the key as recently used.
func (c *BasicLRU[K, V]) Get(key K) (value V, ok bool) {
	item, ok := c.items[key]
	if !ok {
		return value, false
	}
	c.list.moveToFront(item.elem)
	return item.value, true
}

// GetOldest retrieves the least-recently-used item.
// Note that this does not update the item's recency.
func (c *BasicLRU[K, V]) GetOldest() (key K, value V, ok bool) {
	lastElem := c.list.last()
	if lastElem == nil {
		return key, value, false
	}
	key = lastElem.v
	item := c.items[key]
	return key, item.value, true
}

// Len returns the current number of items in the cache.
func (c *BasicLRU[K, V]) Len() int {
	return len(c.items)
}

// Peek retrieves a value from the cache, but does not mark the key as recently used.
func (c *BasicLRU[K, V]) Peek(key K) (value V, ok bool) {
	item, ok := c.items[key]
	return item.value, ok
}

// Purge empties the cache.
func (c *BasicLRU[K, V]) Purge() {
	c.list.init()
	for k := range c.items {
		delete(c.items, k)
	}
}

// Remove drops an item from the cache. Returns true if the key was present in cache.
func (c *BasicLRU[K, V]) Remove(key K) bool {
	item, ok := c.items[key]
	if ok {
		delete(c.items, key)
		c.list.remove(item.elem)
	}
	return ok
}

// RemoveOldest drops the least recently used item.
func (c *BasicLRU[K, V]) RemoveOldest() (key K, value V, ok bool) {
	lastElem := c.list.last()
	if lastElem == nil {
		return key, value, false
	}

	key = lastElem.v
	item := c.items[key]
	delete(c.items, key)
	c.list.remove(lastElem)
	return key, item.value, true
}

// Keys returns all keys in the cache.
func (c *BasicLRU[K, V]) Keys() []K {
	keys := make([]K, 0, len(c.items))
	return c.list.appendTo(keys)
}

// list is a doubly-linked list holding items of type he.
// The zero value is not valid, use newList to create lists.
type list[T any] struct {
	root listElem[T]
}

type listElem[T any] struct {
	next *listElem[T]
	prev *listElem[T]
	v    T
}

func newList[T any]() *list[T] {
	l := new(list[T])
	l.init()
	return l
}

// init reinitializes the list, making it empty.
func (l *list[T]) init() {
	l.root.next = &l.root
	l.root.prev = &l.root
}

// push adds an element to the front of the list.
func (l *list[T]) pushElem(e *listElem[T]) {
	e.prev = &l.root
	e.next = l.root.next
	l.root.next = e
	e.next.prev = e
}

// moveToFront makes 'node' the head of the list.
func (l *list[T]) moveToFront(e *listElem[T]) {
	e.prev.next = e.next
	e.next.prev = e.prev
	l.pushElem(e)
}

// remove removes an element from the list.
func (l *list[T]) remove(e *listElem[T]) {
	e.prev.next = e.next
	e.next.prev = e.prev
	e.next, e.prev = nil, nil
}

// removeLast removes the last element of the list.
func (l *list[T]) removeLast() *listElem[T] {
	last := l.last()
	if last != nil {
		l.remove(last)
	}
	return last
}

// last returns the last element of the list, or nil if the list is empty.
func (l *list[T]) last() *listElem[T] {
	e := l.root.prev
	if e == &l.root {
		return nil
	}
	return e
}

// appendTo appends all list elements to a slice.
func (l *list[T]) appendTo(slice []T) []T {
	for e := l.root.prev; e != &l.root; e = e.prev {
		slice = append(slice, e.v)
	}
	return slice
}
