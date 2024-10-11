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

import (
	"context"
	"errors"
	"fmt"
	"sync"
)

// Semaphore implements a weighted, dynamically reconfigurable semaphore which
// respects context cancellation.
//
// The semaphore implements a FIFO policy, where Acquire requests are satisfied
// in order. This policy provides fairness and prevents starvation but is
// susceptible to head-of-line blocking, where a large request that can't be
// satisfied blocks many other small requests that could be.
type Semaphore struct {
	mu struct {
		sync.Mutex

		capacity int64
		// outstanding can exceed capacity if the capacity is dynamically decreased.
		outstanding int64

		waiters Queue[semaWaiter]

		// numCanceled is the number of waiters in the waiters queue which have been
		// canceled. It is used to determine the current number of active waiters in
		// the queue which is waiters.Len() minus this value.
		numCanceled int

		// numHadToWait accumulates the total number of Acquire requests which had
		// to wait because the semaphore was exhausted.
		numHadToWait int64
	}
}

// NewSemaphore creates a new semaphore with the given capacity.
func NewSemaphore(capacity int64) *Semaphore {
	if capacity <= 0 {
		panic("invalid capacity")
	}
	s := &Semaphore{}
	s.mu.capacity = capacity
	s.mu.waiters = MakeQueue[semaWaiter](&semaQueuePool)
	return s
}

var semaQueuePool = MakeQueueBackingPool[semaWaiter]()

var ErrRequestExceedsCapacity = errors.New("request exceeds semaphore capacity")

// TryAcquire attempts to acquire n units from the semaphore without waiting. On
// success, returns true and the caller must later Release the units.
func (s *Semaphore) TryAcquire(n int64) bool {
	s.mu.Lock()
	defer s.mu.Unlock()

	if s.numWaitersLocked() == 0 && s.mu.outstanding+n <= s.mu.capacity {
		s.mu.outstanding += n
		return true
	}

	return false
}

// Acquire n units from the semaphore, waiting if necessary.
//
// If the context is canceled while we are waiting, returns the context error.
// If n exceeds the current capacity, returns ErrRequestExceedsCapacity.
// On success, the caller must later Release the units.
func (s *Semaphore) Acquire(ctx context.Context, n int64) error {
	s.mu.Lock()

	// Fast path.
	if s.numWaitersLocked() == 0 && s.mu.outstanding+n <= s.mu.capacity {
		s.mu.outstanding += n
		s.mu.Unlock()
		return nil
	}

	if n > s.mu.capacity {
		s.mu.Unlock()
		return ErrRequestExceedsCapacity
	}

	c := chanSyncPool.Get().(chan error)
	defer chanSyncPool.Put(c)
	w := s.mu.waiters.PushBack(semaWaiter{n: n, c: c})
	s.mu.numHadToWait++
	s.mu.Unlock()

	select {
	case <-ctx.Done():
		s.mu.Lock()
		defer s.mu.Unlock()
		// We need to check if we raced with a channel notify (which happens under
		// the lock).
		select {
		case err := <-c:
			// We actually fulfilled or failed the request.
			return err
		default:
		}
		// Mark the request as canceled.
		w.c = nil
		s.mu.numCanceled++
		// If we are the head of the queue, we may be able to fulfill other waiters.
		s.processWaitersLocked()
		return ctx.Err()

	case err := <-c:
		return err
	}
}

// Release n units back. These must be units that were acquired by a previous
// Acquire call. It is legal to split up or coalesce units when releasing.
func (s *Semaphore) Release(n int64) {
	s.mu.Lock()
	defer s.mu.Unlock()

	s.mu.outstanding -= n
	if s.mu.outstanding < 0 {
		panic("releasing more than was acquired")
	}
	s.processWaitersLocked()
}

// UpdateCapacity changes the capacity of the semaphore. If the new capacity is
// smaller, the already outstanding acquisitions might exceed the new capacity
// until they are released.
//
// If there are Acquire calls that are waiting which are requesting more than
// the new capacity, they will error out with ErrRequestExceedsCapacity.
func (s *Semaphore) UpdateCapacity(capacity int64) {
	if capacity <= 0 {
		panic("invalid capacity")
	}
	s.mu.Lock()
	defer s.mu.Unlock()
	s.mu.capacity = capacity
	s.processWaitersLocked()
}

// Stats returns the current state of the semaphore.
func (s *Semaphore) Stats() SemaphoreStats {
	s.mu.Lock()
	defer s.mu.Unlock()
	return SemaphoreStats{
		Capacity:     s.mu.capacity,
		Outstanding:  s.mu.outstanding,
		NumHadToWait: s.mu.numHadToWait,
	}
}

// SemaphoreStats contains information about the current state of the semaphore.
type SemaphoreStats struct {
	// Capacity is the current capacity of the semaphore.
	Capacity int64
	// Outstanding is the number of units that have been acquired. Note that this
	// can exceed Capacity if the capacity was recently decreased.
	Outstanding int64
	// NumHadToWait is the total number of Acquire calls (since the semaphore was
	// created) that had to wait because the semaphore was exhausted. Useful for
	// cumulative metrics.
	NumHadToWait int64

	// TODO(radu): consider keeping track of the total amount of time the
	// semaphore was exhausted (i.e. there were waiters queued).
}

func (ss SemaphoreStats) String() string {
	return fmt.Sprintf("capacity: %d, outstanding: %d, num-had-to-wait: %d",
		ss.Capacity, ss.Outstanding, ss.NumHadToWait)
}

type semaWaiter struct {
	// n is the amount that the waiter is trying to acquire.
	n int64
	// c is the channel on which Acquire is blocked. If the request is canceled,
	// it is set to nil.
	c chan error
}

// numWaitersLocked returns how many requests (that have not been canceled) are
// waiting in the queue.
func (s *Semaphore) numWaitersLocked() int {
	return s.mu.waiters.Len() - s.mu.numCanceled
}

// processWaitersLocked processes and notifies as many waiters from the head of
// the queue as possible.
func (s *Semaphore) processWaitersLocked() {
	for ; s.mu.waiters.Len() > 0; s.mu.waiters.PopFront() {
		switch w := s.mu.waiters.PeekFront(); {
		case w.c == nil:
			// Request was canceled, we can just clean it up.
			s.mu.numCanceled--
			if invariants && s.mu.numCanceled < 0 {
				panic("negative numCanceled")
			}

		case s.mu.outstanding+w.n <= s.mu.capacity:
			// Request can be fulfilled.
			s.mu.outstanding += w.n
			w.c <- nil

		case w.n > s.mu.capacity:
			// Request must be failed. This can happen if the capacity was decreased
			// while the element was queued.
			w.c <- ErrRequestExceedsCapacity

		default:
			// Head of the queue needs to wait some more.
			return
		}
	}
}

// chanSyncPool is used to pool allocations of the channels used to notify
// goroutines waiting in Acquire.
var chanSyncPool = sync.Pool{
	New: func() interface{} { return make(chan error, 1) },
}
