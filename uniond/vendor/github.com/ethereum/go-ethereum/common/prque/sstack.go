// CookieJar - A contestant's algorithm toolbox
// Copyright (c) 2013 Peter Szilagyi. All rights reserved.
//
// CookieJar is dual licensed: use of this source code is governed by a BSD
// license that can be found in the LICENSE file. Alternatively, the CookieJar
// toolbox may be used in accordance with the terms and conditions contained
// in a signed written agreement between you and the author(s).

// This is a duplicated and slightly modified version of "gopkg.in/karalabe/cookiejar.v2/collections/prque".

package prque

import "golang.org/x/exp/constraints"

// The size of a block of data
const blockSize = 4096

// A prioritized item in the sorted stack.
type item[P constraints.Ordered, V any] struct {
	value    V
	priority P
}

// SetIndexCallback is called when the element is moved to a new index.
// Providing SetIndexCallback is optional, it is needed only if the application needs
// to delete elements other than the top one.
type SetIndexCallback[V any] func(data V, index int)

// Internal sortable stack data structure. Implements the Push and Pop ops for
// the stack (heap) functionality and the Len, Less and Swap methods for the
// sortability requirements of the heaps.
type sstack[P constraints.Ordered, V any] struct {
	setIndex SetIndexCallback[V]
	size     int
	capacity int
	offset   int

	blocks [][]*item[P, V]
	active []*item[P, V]
}

// Creates a new, empty stack.
func newSstack[P constraints.Ordered, V any](setIndex SetIndexCallback[V]) *sstack[P, V] {
	result := new(sstack[P, V])
	result.setIndex = setIndex
	result.active = make([]*item[P, V], blockSize)
	result.blocks = [][]*item[P, V]{result.active}
	result.capacity = blockSize
	return result
}

// Pushes a value onto the stack, expanding it if necessary. Required by
// heap.Interface.
func (s *sstack[P, V]) Push(data any) {
	if s.size == s.capacity {
		s.active = make([]*item[P, V], blockSize)
		s.blocks = append(s.blocks, s.active)
		s.capacity += blockSize
		s.offset = 0
	} else if s.offset == blockSize {
		s.active = s.blocks[s.size/blockSize]
		s.offset = 0
	}
	if s.setIndex != nil {
		s.setIndex(data.(*item[P, V]).value, s.size)
	}
	s.active[s.offset] = data.(*item[P, V])
	s.offset++
	s.size++
}

// Pops a value off the stack and returns it. Currently no shrinking is done.
// Required by heap.Interface.
func (s *sstack[P, V]) Pop() (res any) {
	s.size--
	s.offset--
	if s.offset < 0 {
		s.offset = blockSize - 1
		s.active = s.blocks[s.size/blockSize]
	}
	res, s.active[s.offset] = s.active[s.offset], nil
	if s.setIndex != nil {
		s.setIndex(res.(*item[P, V]).value, -1)
	}
	return
}

// Returns the length of the stack. Required by sort.Interface.
func (s *sstack[P, V]) Len() int {
	return s.size
}

// Compares the priority of two elements of the stack (higher is first).
// Required by sort.Interface.
func (s *sstack[P, V]) Less(i, j int) bool {
	return s.blocks[i/blockSize][i%blockSize].priority > s.blocks[j/blockSize][j%blockSize].priority
}

// Swaps two elements in the stack. Required by sort.Interface.
func (s *sstack[P, V]) Swap(i, j int) {
	ib, io, jb, jo := i/blockSize, i%blockSize, j/blockSize, j%blockSize
	a, b := s.blocks[jb][jo], s.blocks[ib][io]
	if s.setIndex != nil {
		s.setIndex(a.value, i)
		s.setIndex(b.value, j)
	}
	s.blocks[ib][io], s.blocks[jb][jo] = a, b
}

// Resets the stack, effectively clearing its contents.
func (s *sstack[P, V]) Reset() {
	*s = *newSstack[P, V](s.setIndex)
}
