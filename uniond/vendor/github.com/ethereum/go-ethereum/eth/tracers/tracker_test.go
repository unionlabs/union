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
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>

package tracers

import (
	"reflect"
	"testing"
	"time"
)

func TestTracker(t *testing.T) {
	var cases = []struct {
		limit   int
		calls   []uint64
		expHead uint64
	}{
		// Release in order
		{
			limit:   3,
			calls:   []uint64{0, 1, 2},
			expHead: 3,
		},
		{
			limit:   3,
			calls:   []uint64{0, 1, 2, 3, 4, 5},
			expHead: 6,
		},

		// Release out of order
		{
			limit:   3,
			calls:   []uint64{1, 2, 0},
			expHead: 3,
		},
		{
			limit:   3,
			calls:   []uint64{1, 2, 0, 5, 4, 3},
			expHead: 6,
		},
	}
	for _, c := range cases {
		tracker := newStateTracker(c.limit, 0)
		for _, call := range c.calls {
			tracker.releaseState(call, func() {})
		}
		tracker.lock.RLock()
		head := tracker.oldest
		tracker.lock.RUnlock()

		if head != c.expHead {
			t.Fatalf("Unexpected head want %d got %d", c.expHead, head)
		}
	}

	var calls = []struct {
		number  uint64
		expUsed []bool
		expHead uint64
	}{
		// Release the first one, update the oldest flag
		{
			number:  0,
			expUsed: []bool{false, false, false, false, false},
			expHead: 1,
		},
		// Release the second one, oldest shouldn't be updated
		{
			number:  2,
			expUsed: []bool{false, true, false, false, false},
			expHead: 1,
		},
		// Release the forth one, oldest shouldn't be updated
		{
			number:  4,
			expUsed: []bool{false, true, false, true, false},
			expHead: 1,
		},
		// Release the first one, the first two should all be cleaned,
		// and the remaining flags should all be left-shifted.
		{
			number:  1,
			expUsed: []bool{false, true, false, false, false},
			expHead: 3,
		},
		// Release the first one, the first two should all be cleaned
		{
			number:  3,
			expUsed: []bool{false, false, false, false, false},
			expHead: 5,
		},
	}
	tracker := newStateTracker(5, 0) // limit = 5, oldest = 0
	for _, call := range calls {
		tracker.releaseState(call.number, nil)
		tracker.lock.RLock()
		if !reflect.DeepEqual(tracker.used, call.expUsed) {
			t.Fatalf("Unexpected used array")
		}
		if tracker.oldest != call.expHead {
			t.Fatalf("Unexpected head")
		}
		tracker.lock.RUnlock()
	}
}

func TestTrackerWait(t *testing.T) {
	var (
		tracker = newStateTracker(5, 0) // limit = 5, oldest = 0
		result  = make(chan error, 1)
		doCall  = func(number uint64) {
			go func() {
				result <- tracker.wait(number)
			}()
		}
		checkNoWait = func() {
			select {
			case <-result:
				return
			case <-time.NewTimer(time.Second).C:
				t.Fatal("No signal fired")
			}
		}
		checkWait = func() {
			select {
			case <-result:
				t.Fatal("Unexpected signal")
			case <-time.NewTimer(time.Millisecond * 100).C:
			}
		}
	)
	// States [0, 5) should all be available
	doCall(0)
	checkNoWait()

	doCall(4)
	checkNoWait()

	// State 5 is not available
	doCall(5)
	checkWait()

	// States [1, 6) are available
	tracker.releaseState(0, nil)
	checkNoWait()

	// States [1, 6) are available
	doCall(7)
	checkWait()

	// States [2, 7) are available
	tracker.releaseState(1, nil)
	checkWait()

	// States [3, 8) are available
	tracker.releaseState(2, nil)
	checkNoWait()
}
