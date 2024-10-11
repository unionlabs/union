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

package tracers

import (
	"fmt"
	"sync"
)

// stateTracker is an auxiliary tool used to cache the release functions of all
// used trace states, and to determine whether the creation of trace state needs
// to be paused in case there are too many states waiting for tracing.
type stateTracker struct {
	limit    int                // Maximum number of states allowed waiting for tracing
	oldest   uint64             // The number of the oldest state which is still using for trace
	used     []bool             // List of flags indicating whether the trace state has been used up
	releases []StateReleaseFunc // List of trace state release functions waiting to be called
	cond     *sync.Cond
	lock     *sync.RWMutex
}

// newStateTracker initializes the tracker with provided state limits and
// the number of the first state that will be used.
func newStateTracker(limit int, oldest uint64) *stateTracker {
	lock := new(sync.RWMutex)
	return &stateTracker{
		limit:  limit,
		oldest: oldest,
		used:   make([]bool, limit),
		cond:   sync.NewCond(lock),
		lock:   lock,
	}
}

// releaseState marks the state specified by the number as released and caches
// the corresponding release functions internally.
func (t *stateTracker) releaseState(number uint64, release StateReleaseFunc) {
	t.lock.Lock()
	defer t.lock.Unlock()

	// Set the state as used, the corresponding flag is indexed by
	// the distance between the specified state and the oldest state
	// which is still using for trace.
	t.used[int(number-t.oldest)] = true

	// If the oldest state is used up, update the oldest marker by moving
	// it to the next state which is not used up.
	if number == t.oldest {
		var count int
		for _, used := range t.used {
			if !used {
				break
			}
			count += 1
		}
		t.oldest += uint64(count)
		copy(t.used, t.used[count:])

		// Clean up the array tail since they are useless now.
		for i := t.limit - count; i < t.limit; i++ {
			t.used[i] = false
		}
		// Fire the signal to all waiters that oldest marker is updated.
		t.cond.Broadcast()
	}
	t.releases = append(t.releases, release)
}

// callReleases invokes all cached release functions.
func (t *stateTracker) callReleases() {
	t.lock.Lock()
	defer t.lock.Unlock()

	for _, release := range t.releases {
		release()
	}
	t.releases = t.releases[:0]
}

// wait blocks until the accumulated trace states are less than the limit.
func (t *stateTracker) wait(number uint64) error {
	t.lock.Lock()
	defer t.lock.Unlock()

	for {
		if number < t.oldest {
			return fmt.Errorf("invalid state number %d head %d", number, t.oldest)
		}
		if number < t.oldest+uint64(t.limit) {
			// number is now within limit, wait over
			return nil
		}
		t.cond.Wait()
	}
}
