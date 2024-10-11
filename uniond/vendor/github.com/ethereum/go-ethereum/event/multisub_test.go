// Copyright 2023 The go-ethereum Authors
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

package event

import (
	"testing"
	"time"
)

func TestMultisub(t *testing.T) {
	// Create a double subscription and ensure events propagate through
	var (
		feed1 Feed
		feed2 Feed
	)
	sink1 := make(chan int, 1)
	sink2 := make(chan int, 1)

	sub1 := feed1.Subscribe(sink1)
	sub2 := feed2.Subscribe(sink2)

	sub := JoinSubscriptions(sub1, sub2)

	feed1.Send(1)
	select {
	case n := <-sink1:
		if n != 1 {
			t.Errorf("sink 1 delivery mismatch: have %d, want %d", n, 1)
		}
	default:
		t.Error("sink 1 missing delivery")
	}

	feed2.Send(2)
	select {
	case n := <-sink2:
		if n != 2 {
			t.Errorf("sink 2 delivery mismatch: have %d, want %d", n, 2)
		}
	default:
		t.Error("sink 2 missing delivery")
	}
	// Unsubscribe and ensure no more events are delivered
	sub.Unsubscribe()
	select {
	case <-sub.Err():
	case <-time.After(50 * time.Millisecond):
		t.Error("multisub didn't propagate closure")
	}

	feed1.Send(11)
	select {
	case n := <-sink1:
		t.Errorf("sink 1 unexpected delivery: %d", n)
	default:
	}

	feed2.Send(22)
	select {
	case n := <-sink2:
		t.Errorf("sink 2 unexpected delivery: %d", n)
	default:
	}
}

func TestMutisubPartialUnsubscribe(t *testing.T) {
	// Create a double subscription but terminate one half, ensuring no error
	// is propagated yet up to the outer subscription
	var (
		feed1 Feed
		feed2 Feed
	)
	sink1 := make(chan int, 1)
	sink2 := make(chan int, 1)

	sub1 := feed1.Subscribe(sink1)
	sub2 := feed2.Subscribe(sink2)

	sub := JoinSubscriptions(sub1, sub2)

	sub1.Unsubscribe()
	select {
	case <-sub.Err():
		t.Error("multisub propagated closure")
	case <-time.After(50 * time.Millisecond):
	}
	// Ensure that events cross only the second feed
	feed1.Send(1)
	select {
	case n := <-sink1:
		t.Errorf("sink 1 unexpected delivery: %d", n)
	default:
	}

	feed2.Send(2)
	select {
	case n := <-sink2:
		if n != 2 {
			t.Errorf("sink 2 delivery mismatch: have %d, want %d", n, 2)
		}
	default:
		t.Error("sink 2 missing delivery")
	}
	// Unsubscribe and ensure no more events are delivered
	sub.Unsubscribe()
	select {
	case <-sub.Err():
	case <-time.After(50 * time.Millisecond):
		t.Error("multisub didn't propagate closure")
	}

	feed1.Send(11)
	select {
	case n := <-sink1:
		t.Errorf("sink 1 unexpected delivery: %d", n)
	default:
	}

	feed2.Send(22)
	select {
	case n := <-sink2:
		t.Errorf("sink 2 unexpected delivery: %d", n)
	default:
	}
}

func TestMultisubFullUnsubscribe(t *testing.T) {
	// Create a double subscription and terminate the multi sub, ensuring an
	// error is propagated up.
	var (
		feed1 Feed
		feed2 Feed
	)
	sink1 := make(chan int, 1)
	sink2 := make(chan int, 1)

	sub1 := feed1.Subscribe(sink1)
	sub2 := feed2.Subscribe(sink2)

	sub := JoinSubscriptions(sub1, sub2)
	sub.Unsubscribe()
	select {
	case <-sub.Err():
	case <-time.After(50 * time.Millisecond):
		t.Error("multisub didn't propagate closure")
	}
	// Ensure no more events are delivered
	feed1.Send(1)
	select {
	case n := <-sink1:
		t.Errorf("sink 1 unexpected delivery: %d", n)
	default:
	}

	feed2.Send(2)
	select {
	case n := <-sink2:
		t.Errorf("sink 2 unexpected delivery: %d", n)
	default:
	}
}
