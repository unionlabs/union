// Copyright 2020 The go-ethereum Authors
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

package utils

import "sync"

// ExecQueue implements a queue that executes function calls in a single thread,
// in the same order as they have been queued.
type ExecQueue struct {
	mu        sync.Mutex
	cond      *sync.Cond
	funcs     []func()
	closeWait chan struct{}
}

// NewExecQueue creates a new execution Queue.
func NewExecQueue(capacity int) *ExecQueue {
	q := &ExecQueue{funcs: make([]func(), 0, capacity)}
	q.cond = sync.NewCond(&q.mu)
	go q.loop()
	return q
}

func (q *ExecQueue) loop() {
	for f := q.waitNext(false); f != nil; f = q.waitNext(true) {
		f()
	}
	close(q.closeWait)
}

func (q *ExecQueue) waitNext(drop bool) (f func()) {
	q.mu.Lock()
	if drop && len(q.funcs) > 0 {
		// Remove the function that just executed. We do this here instead of when
		// dequeuing so len(q.funcs) includes the function that is running.
		q.funcs = append(q.funcs[:0], q.funcs[1:]...)
	}
	for !q.isClosed() {
		if len(q.funcs) > 0 {
			f = q.funcs[0]
			break
		}
		q.cond.Wait()
	}
	q.mu.Unlock()
	return f
}

func (q *ExecQueue) isClosed() bool {
	return q.closeWait != nil
}

// CanQueue returns true if more function calls can be added to the execution Queue.
func (q *ExecQueue) CanQueue() bool {
	q.mu.Lock()
	ok := !q.isClosed() && len(q.funcs) < cap(q.funcs)
	q.mu.Unlock()
	return ok
}

// Queue adds a function call to the execution Queue. Returns true if successful.
func (q *ExecQueue) Queue(f func()) bool {
	q.mu.Lock()
	ok := !q.isClosed() && len(q.funcs) < cap(q.funcs)
	if ok {
		q.funcs = append(q.funcs, f)
		q.cond.Signal()
	}
	q.mu.Unlock()
	return ok
}

// Clear drops all queued functions.
func (q *ExecQueue) Clear() {
	q.mu.Lock()
	q.funcs = q.funcs[:0]
	q.mu.Unlock()
}

// Quit stops the exec Queue.
//
// Quit waits for the current execution to finish before returning.
func (q *ExecQueue) Quit() {
	q.mu.Lock()
	if !q.isClosed() {
		q.closeWait = make(chan struct{})
		q.cond.Signal()
	}
	q.mu.Unlock()
	<-q.closeWait
}
