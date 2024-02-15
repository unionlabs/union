// Copyright 2009 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE_list file.
package nonblocking

// entry is an LRU entry
type entry[K comparable, V any] struct {
	// Next and previous pointers in the doubly-linked list of elements.
	// To simplify the implementation, internally a list l is implemented
	// as a ring, such that &l.root is both the next element of the last
	// list element (l.Back()) and the previous element of the first list
	// element (l.Front()).
	next, prev *entry[K, V]

	// The list to which this element belongs.
	list *lruList[K, V]

	// The LRU key of this element.
	key K

	// The value stored with this element.
	value V
}

// lruList represents a doubly linked list.
// The zero value for lruList is an empty list ready to use.
type lruList[K comparable, V any] struct {
	root entry[K, V] // sentinel list element, only &root, root.prev, and root.next are used
	len  int         // current list length excluding (this) sentinel element
}

// init initializes or clears list l.
func (l *lruList[K, V]) init() *lruList[K, V] {
	l.root.next = &l.root
	l.root.prev = &l.root
	l.len = 0
	return l
}

// newList returns an initialized list.
func newList[K comparable, V any]() *lruList[K, V] { return new(lruList[K, V]).init() }

// length returns the number of elements of list l.
// The complexity is O(1).
func (l *lruList[K, V]) length() int { return l.len }

// back returns the last element of list l or nil if the list is empty.
func (l *lruList[K, V]) back() *entry[K, V] {
	if l.len == 0 {
		return nil
	}
	return l.root.prev
}

// lazyInit lazily initializes a zero List value.
func (l *lruList[K, V]) lazyInit() {
	if l.root.next == nil {
		l.init()
	}
}

// insert inserts e after at, increments l.len, and returns e.
func (l *lruList[K, V]) insert(e, at *entry[K, V]) *entry[K, V] {
	e.prev = at
	e.next = at.next
	e.prev.next = e
	e.next.prev = e
	e.list = l
	l.len++
	return e
}

// insertValue is a convenience wrapper for insert(&Element{Value: v}, at).
func (l *lruList[K, V]) insertValue(k K, v V, at *entry[K, V]) *entry[K, V] {
	return l.insert(&entry[K, V]{value: v, key: k}, at)
}

// remove removes e from its list, decrements l.len
func (l *lruList[K, V]) remove(e *entry[K, V]) V {
	// If already removed, do nothing.
	if e.prev == nil && e.next == nil {
		return e.value
	}
	e.prev.next = e.next
	e.next.prev = e.prev
	e.next = nil // avoid memory leaks
	e.prev = nil // avoid memory leaks
	e.list = nil
	l.len--

	return e.value
}

// move moves e to next to at.
func (*lruList[K, V]) move(e, at *entry[K, V]) {
	if e == at {
		return
	}
	e.prev.next = e.next
	e.next.prev = e.prev

	e.prev = at
	e.next = at.next
	e.prev.next = e
	e.next.prev = e
}

// pushFront inserts a new element e with value v at the front of list l and returns e.
func (l *lruList[K, V]) pushFront(k K, v V) *entry[K, V] {
	l.lazyInit()
	return l.insertValue(k, v, &l.root)
}

// moveToFront moves element e to the front of list l.
// If e is not an element of l, the list is not modified.
// The element must not be nil.
func (l *lruList[K, V]) moveToFront(e *entry[K, V]) {
	if e.list != l || l.root.next == e {
		return
	}
	// see comment in List.Remove about initialization of l
	l.move(e, &l.root)
}
