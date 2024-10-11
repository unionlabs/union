// CookieJar - A contestant's algorithm toolbox
// Copyright (c) 2013 Peter Szilagyi. All rights reserved.
//
// CookieJar is dual licensed: use of this source code is governed by a BSD
// license that can be found in the LICENSE file. Alternatively, the CookieJar
// toolbox may be used in accordance with the terms and conditions contained
// in a signed written agreement between you and the author(s).

package prque

import (
	"math/rand"
	"sort"
	"testing"
)

func TestSstack(t *testing.T) {
	// Create some initial data
	size := 16 * blockSize
	data := make([]*item[int64, int], size)
	for i := 0; i < size; i++ {
		data[i] = &item[int64, int]{rand.Int(), rand.Int63()}
	}
	stack := newSstack[int64, int](nil)
	for rep := 0; rep < 2; rep++ {
		// Push all the data into the stack, pop out every second
		secs := []*item[int64, int]{}
		for i := 0; i < size; i++ {
			stack.Push(data[i])
			if i%2 == 0 {
				secs = append(secs, stack.Pop().(*item[int64, int]))
			}
		}
		rest := []*item[int64, int]{}
		for stack.Len() > 0 {
			rest = append(rest, stack.Pop().(*item[int64, int]))
		}
		// Make sure the contents of the resulting slices are ok
		for i := 0; i < size; i++ {
			if i%2 == 0 && data[i] != secs[i/2] {
				t.Errorf("push/pop mismatch: have %v, want %v.", secs[i/2], data[i])
			}
			if i%2 == 1 && data[i] != rest[len(rest)-i/2-1] {
				t.Errorf("push/pop mismatch: have %v, want %v.", rest[len(rest)-i/2-1], data[i])
			}
		}
	}
}

func TestSstackSort(t *testing.T) {
	// Create some initial data
	size := 16 * blockSize
	data := make([]*item[int64, int], size)
	for i := 0; i < size; i++ {
		data[i] = &item[int64, int]{rand.Int(), int64(i)}
	}
	// Push all the data into the stack
	stack := newSstack[int64, int](nil)
	for _, val := range data {
		stack.Push(val)
	}
	// Sort and pop the stack contents (should reverse the order)
	sort.Sort(stack)
	for _, val := range data {
		out := stack.Pop()
		if out != val {
			t.Errorf("push/pop mismatch after sort: have %v, want %v.", out, val)
		}
	}
}

func TestSstackReset(t *testing.T) {
	// Create some initial data
	size := 16 * blockSize
	data := make([]*item[int64, int], size)
	for i := 0; i < size; i++ {
		data[i] = &item[int64, int]{rand.Int(), rand.Int63()}
	}
	stack := newSstack[int64, int](nil)
	for rep := 0; rep < 2; rep++ {
		// Push all the data into the stack, pop out every second
		secs := []*item[int64, int]{}
		for i := 0; i < size; i++ {
			stack.Push(data[i])
			if i%2 == 0 {
				secs = append(secs, stack.Pop().(*item[int64, int]))
			}
		}
		// Reset and verify both pulled and stack contents
		stack.Reset()
		if stack.Len() != 0 {
			t.Errorf("stack not empty after reset: %v", stack)
		}
		for i := 0; i < size; i++ {
			if i%2 == 0 && data[i] != secs[i/2] {
				t.Errorf("push/pop mismatch: have %v, want %v.", secs[i/2], data[i])
			}
		}
	}
}
