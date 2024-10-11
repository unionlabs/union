// Copyright 2024 The Cockroach Authors.
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

package fifo

import "sync"

// Queue implements an allocation efficient FIFO queue. It is not safe for
// concurrent access.
//
// Note that the queue provides pointer access to the internal storage (via
// PeekFront and PushBack) so it must be used with care. These pointers must not
// be used once the respective element is popped out of the queue.
//
// -- Implementation --
//
// The queue is implemented as a linked list of nodes, where each node is a
// small ring buffer. The nodes are allocated using a sync.Pool (a single pool
// is created for any given type and is used for all queues of that type).
type Queue[T any] struct {
	len        int
	head, tail *queueNode[T]

	pool *QueueBackingPool[T]
}

// MakeQueue constructs a new Queue.
//
// The pool should be a singleton object initialized with MakeQueueBackingPool.
// A single pool can and should be used by all queues of that type.
func MakeQueue[T any](pool *QueueBackingPool[T]) Queue[T] {
	return Queue[T]{
		pool: pool,
	}
}

// Len returns the current length of the queue.
func (q *Queue[T]) Len() int {
	return q.len
}

// PushBack adds t to the end of the queue.
//
// The returned pointer can be used to modify the element while it is in the
// queue; it is valid until the element is removed from the queue.
func (q *Queue[T]) PushBack(t T) *T {
	if q.head == nil {
		q.head = q.pool.get()
		q.tail = q.head
	} else if q.tail.IsFull() {
		newTail := q.pool.get()
		q.tail.next = newTail
		q.tail = newTail
	}
	q.len++
	return q.tail.PushBack(t)
}

// PeekFront returns the current head of the queue, or nil if the queue is
// empty.
//
// The result is only valid until the next call to PopFront.
func (q *Queue[T]) PeekFront() *T {
	if q.len == 0 {
		return nil
	}
	return q.head.PeekFront()
}

// PopFront removes the current head of the queue.
//
// It is illegal to call PopFront on an empty queue.
func (q *Queue[T]) PopFront() {
	q.head.PopFront()
	// If this is the only node, we don't want to release it; otherwise we would
	// allocate/free a node every time we transition between the queue being empty
	// and non-empty.
	if q.head.len == 0 && q.head.next != nil {
		oldHead := q.head
		q.head = oldHead.next
		q.pool.put(oldHead)
	}
	q.len--
}

// QueueBackingPool is a sync.Pool that used to allocate internal nodes
// for Queue[T].
type QueueBackingPool[T any] struct {
	pool sync.Pool
}

// MakeQueueBackingPool makes a queue backing pool. It is intented to be used to
// initialize a singleton (global) variable. A single pool can and should be
// used by all queues of that type.
func MakeQueueBackingPool[T any]() QueueBackingPool[T] {
	return QueueBackingPool[T]{
		pool: sync.Pool{
			New: func() interface{} { return &queueNode[T]{} },
		},
	}
}

func (qp *QueueBackingPool[T]) get() *queueNode[T] {
	return qp.pool.Get().(*queueNode[T])
}

func (qp *QueueBackingPool[T]) put(n *queueNode[T]) {
	*n = queueNode[T]{}
	qp.pool.Put(n)
}

// We batch the allocation of this many queue objects. The value was chosen
// without experimentation - it provides a reasonable amount of amortization
// without a very large increase in memory overhead if T is large.
const queueNodeSize = 8

type queueNode[T any] struct {
	buf       [queueNodeSize]T
	head, len int32
	next      *queueNode[T]
}

func (qn *queueNode[T]) IsFull() bool {
	return qn.len == queueNodeSize
}

func (qn *queueNode[T]) PushBack(t T) *T {
	if invariants && qn.len >= queueNodeSize {
		panic("cannot push back into a full node")
	}
	i := (qn.head + qn.len) % queueNodeSize
	qn.buf[i] = t
	qn.len++
	return &qn.buf[i]
}

func (qn *queueNode[T]) PeekFront() *T {
	return &qn.buf[qn.head]
}

func (qn *queueNode[T]) PopFront() T {
	if invariants && qn.len == 0 {
		panic("cannot pop from empty queue")
	}
	t := qn.buf[qn.head]
	var zero T
	qn.buf[qn.head] = zero
	qn.head = (qn.head + 1) % queueNodeSize
	qn.len--
	return t
}
