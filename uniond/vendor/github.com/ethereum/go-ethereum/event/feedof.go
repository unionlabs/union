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

package event

import (
	"reflect"
	"sync"
)

// FeedOf implements one-to-many subscriptions where the carrier of events is a channel.
// Values sent to a Feed are delivered to all subscribed channels simultaneously.
//
// The zero value is ready to use.
type FeedOf[T any] struct {
	once      sync.Once     // ensures that init only runs once
	sendLock  chan struct{} // sendLock has a one-element buffer and is empty when held.It protects sendCases.
	removeSub chan chan<- T // interrupts Send
	sendCases caseList      // the active set of select cases used by Send

	// The inbox holds newly subscribed channels until they are added to sendCases.
	mu    sync.Mutex
	inbox caseList
}

func (f *FeedOf[T]) init() {
	f.removeSub = make(chan chan<- T)
	f.sendLock = make(chan struct{}, 1)
	f.sendLock <- struct{}{}
	f.sendCases = caseList{{Chan: reflect.ValueOf(f.removeSub), Dir: reflect.SelectRecv}}
}

// Subscribe adds a channel to the feed. Future sends will be delivered on the channel
// until the subscription is canceled.
//
// The channel should have ample buffer space to avoid blocking other subscribers. Slow
// subscribers are not dropped.
func (f *FeedOf[T]) Subscribe(channel chan<- T) Subscription {
	f.once.Do(f.init)

	chanval := reflect.ValueOf(channel)
	sub := &feedOfSub[T]{feed: f, channel: channel, err: make(chan error, 1)}

	// Add the select case to the inbox.
	// The next Send will add it to f.sendCases.
	f.mu.Lock()
	defer f.mu.Unlock()
	cas := reflect.SelectCase{Dir: reflect.SelectSend, Chan: chanval}
	f.inbox = append(f.inbox, cas)
	return sub
}

func (f *FeedOf[T]) remove(sub *feedOfSub[T]) {
	// Delete from inbox first, which covers channels
	// that have not been added to f.sendCases yet.
	f.mu.Lock()
	index := f.inbox.find(sub.channel)
	if index != -1 {
		f.inbox = f.inbox.delete(index)
		f.mu.Unlock()
		return
	}
	f.mu.Unlock()

	select {
	case f.removeSub <- sub.channel:
		// Send will remove the channel from f.sendCases.
	case <-f.sendLock:
		// No Send is in progress, delete the channel now that we have the send lock.
		f.sendCases = f.sendCases.delete(f.sendCases.find(sub.channel))
		f.sendLock <- struct{}{}
	}
}

// Send delivers to all subscribed channels simultaneously.
// It returns the number of subscribers that the value was sent to.
func (f *FeedOf[T]) Send(value T) (nsent int) {
	rvalue := reflect.ValueOf(value)

	f.once.Do(f.init)
	<-f.sendLock

	// Add new cases from the inbox after taking the send lock.
	f.mu.Lock()
	f.sendCases = append(f.sendCases, f.inbox...)
	f.inbox = nil
	f.mu.Unlock()

	// Set the sent value on all channels.
	for i := firstSubSendCase; i < len(f.sendCases); i++ {
		f.sendCases[i].Send = rvalue
	}

	// Send until all channels except removeSub have been chosen. 'cases' tracks a prefix
	// of sendCases. When a send succeeds, the corresponding case moves to the end of
	// 'cases' and it shrinks by one element.
	cases := f.sendCases
	for {
		// Fast path: try sending without blocking before adding to the select set.
		// This should usually succeed if subscribers are fast enough and have free
		// buffer space.
		for i := firstSubSendCase; i < len(cases); i++ {
			if cases[i].Chan.TrySend(rvalue) {
				nsent++
				cases = cases.deactivate(i)
				i--
			}
		}
		if len(cases) == firstSubSendCase {
			break
		}
		// Select on all the receivers, waiting for them to unblock.
		chosen, recv, _ := reflect.Select(cases)
		if chosen == 0 /* <-f.removeSub */ {
			index := f.sendCases.find(recv.Interface())
			f.sendCases = f.sendCases.delete(index)
			if index >= 0 && index < len(cases) {
				// Shrink 'cases' too because the removed case was still active.
				cases = f.sendCases[:len(cases)-1]
			}
		} else {
			cases = cases.deactivate(chosen)
			nsent++
		}
	}

	// Forget about the sent value and hand off the send lock.
	for i := firstSubSendCase; i < len(f.sendCases); i++ {
		f.sendCases[i].Send = reflect.Value{}
	}
	f.sendLock <- struct{}{}
	return nsent
}

type feedOfSub[T any] struct {
	feed    *FeedOf[T]
	channel chan<- T
	errOnce sync.Once
	err     chan error
}

func (sub *feedOfSub[T]) Unsubscribe() {
	sub.errOnce.Do(func() {
		sub.feed.remove(sub)
		close(sub.err)
	})
}

func (sub *feedOfSub[T]) Err() <-chan error {
	return sub.err
}
