// CookieJar - A contestant's algorithm toolbox
// Copyright (c) 2013 Peter Szilagyi. All rights reserved.
//
// CookieJar is dual licensed: use of this source code is governed by a BSD
// license that can be found in the LICENSE file. Alternatively, the CookieJar
// toolbox may be used in accordance with the terms and conditions contained
// in a signed written agreement between you and the author(s).

// This is a duplicated and slightly modified version of "gopkg.in/karalabe/cookiejar.v2/collections/prque".

// Package prque implements a priority queue data structure supporting arbitrary
// value types and int64 priorities.
//
// If you would like to use a min-priority queue, simply negate the priorities.
//
// Internally the queue is based on the standard heap package working on a
// sortable version of the block based stack.
package prque

import (
	"container/heap"

	"golang.org/x/exp/constraints"
)

// Priority queue data structure.
type Prque[P constraints.Ordered, V any] struct {
	cont *sstack[P, V]
}

// New creates a new priority queue.
func New[P constraints.Ordered, V any](setIndex SetIndexCallback[V]) *Prque[P, V] {
	return &Prque[P, V]{newSstack[P, V](setIndex)}
}

// Pushes a value with a given priority into the queue, expanding if necessary.
func (p *Prque[P, V]) Push(data V, priority P) {
	heap.Push(p.cont, &item[P, V]{data, priority})
}

// Peek returns the value with the greatest priority but does not pop it off.
func (p *Prque[P, V]) Peek() (V, P) {
	item := p.cont.blocks[0][0]
	return item.value, item.priority
}

// Pops the value with the greatest priority off the stack and returns it.
// Currently no shrinking is done.
func (p *Prque[P, V]) Pop() (V, P) {
	item := heap.Pop(p.cont).(*item[P, V])
	return item.value, item.priority
}

// Pops only the item from the queue, dropping the associated priority value.
func (p *Prque[P, V]) PopItem() V {
	return heap.Pop(p.cont).(*item[P, V]).value
}

// Remove removes the element with the given index.
func (p *Prque[P, V]) Remove(i int) V {
	return heap.Remove(p.cont, i).(*item[P, V]).value
}

// Checks whether the priority queue is empty.
func (p *Prque[P, V]) Empty() bool {
	return p.cont.Len() == 0
}

// Returns the number of element in the priority queue.
func (p *Prque[P, V]) Size() int {
	return p.cont.Len()
}

// Clears the contents of the priority queue.
func (p *Prque[P, V]) Reset() {
	*p = *New[P, V](p.cont.setIndex)
}
