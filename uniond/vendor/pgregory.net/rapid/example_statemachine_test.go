// Copyright 2019 Gregory Petrosyan <gregory.petrosyan@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package rapid_test

import (
	"testing"

	"pgregory.net/rapid"
)

// Queue implements integer queue with a fixed maximum size.
type Queue struct {
	buf []int
	in  int
	out int
}

func NewQueue(n int) *Queue {
	return &Queue{
		buf: make([]int, n+1),
	}
}

// Precondition: Size() > 0.
func (q *Queue) Get() int {
	i := q.buf[q.out]
	q.out = (q.out + 1) % len(q.buf)
	return i
}

// Precondition: Size() < n.
func (q *Queue) Put(i int) {
	q.buf[q.in] = i
	q.in = (q.in + 1) % len(q.buf)
}

func (q *Queue) Size() int {
	return (q.in - q.out) % len(q.buf)
}

func testQueue(t *rapid.T) {
	n := rapid.IntRange(1, 1000).Draw(t, "n") // maximum queue size
	q := NewQueue(n)                          // queue being tested
	var state []int                           // model of the queue

	t.Repeat(map[string]func(*rapid.T){
		"get": func(t *rapid.T) {
			if q.Size() == 0 {
				t.Skip("queue empty")
			}

			i := q.Get()
			if i != state[0] {
				t.Fatalf("got invalid value: %v vs expected %v", i, state[0])
			}
			state = state[1:]
		},
		"put": func(t *rapid.T) {
			if q.Size() == n {
				t.Skip("queue full")
			}

			i := rapid.Int().Draw(t, "i")
			q.Put(i)
			state = append(state, i)
		},
		"": func(t *rapid.T) {
			if q.Size() != len(state) {
				t.Fatalf("queue size mismatch: %v vs expected %v", q.Size(), len(state))
			}
		},
	})
}

// Rename to TestQueue(t *testing.T) to make an actual (failing) test.
func ExampleT_Repeat_queue() {
	var t *testing.T
	rapid.Check(t, testQueue)
}
