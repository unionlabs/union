// Copyright 2016 The go-ethereum Authors
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

package event_test

import (
	"fmt"

	"github.com/ethereum/go-ethereum/event"
)

func ExampleFeed_acknowledgedEvents() {
	// This example shows how the return value of Send can be used for request/reply
	// interaction between event consumers and producers.
	var feed event.Feed
	type ackedEvent struct {
		i   int
		ack chan<- struct{}
	}

	// Consumers wait for events on the feed and acknowledge processing.
	done := make(chan struct{})
	defer close(done)
	for i := 0; i < 3; i++ {
		ch := make(chan ackedEvent, 100)
		sub := feed.Subscribe(ch)
		go func() {
			defer sub.Unsubscribe()
			for {
				select {
				case ev := <-ch:
					fmt.Println(ev.i) // "process" the event
					ev.ack <- struct{}{}
				case <-done:
					return
				}
			}
		}()
	}

	// The producer sends values of type ackedEvent with increasing values of i.
	// It waits for all consumers to acknowledge before sending the next event.
	for i := 0; i < 3; i++ {
		acksignal := make(chan struct{})
		n := feed.Send(ackedEvent{i, acksignal})
		for ack := 0; ack < n; ack++ {
			<-acksignal
		}
	}
	// Output:
	// 0
	// 0
	// 0
	// 1
	// 1
	// 1
	// 2
	// 2
	// 2
}
