// Copyright 2018 The go-ethereum Authors
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

package mclock

import (
	"container/heap"
	"sync"
	"time"
)

// Simulated implements a virtual Clock for reproducible time-sensitive tests. It
// simulates a scheduler on a virtual timescale where actual processing takes zero time.
//
// The virtual clock doesn't advance on its own, call Run to advance it and execute timers.
// Since there is no way to influence the Go scheduler, testing timeout behaviour involving
// goroutines needs special care. A good way to test such timeouts is as follows: First
// perform the action that is supposed to time out. Ensure that the timer you want to test
// is created. Then run the clock until after the timeout. Finally observe the effect of
// the timeout using a channel or semaphore.
type Simulated struct {
	now       AbsTime
	scheduled simTimerHeap
	mu        sync.RWMutex
	cond      *sync.Cond
}

// simTimer implements ChanTimer on the virtual clock.
type simTimer struct {
	at    AbsTime
	index int // position in s.scheduled
	s     *Simulated
	do    func()
	ch    <-chan AbsTime
}

func (s *Simulated) init() {
	if s.cond == nil {
		s.cond = sync.NewCond(&s.mu)
	}
}

// Run moves the clock by the given duration, executing all timers before that duration.
func (s *Simulated) Run(d time.Duration) {
	s.mu.Lock()
	s.init()

	end := s.now.Add(d)
	var do []func()
	for len(s.scheduled) > 0 && s.scheduled[0].at <= end {
		ev := heap.Pop(&s.scheduled).(*simTimer)
		do = append(do, ev.do)
	}
	s.now = end
	s.mu.Unlock()

	for _, fn := range do {
		fn()
	}
}

// ActiveTimers returns the number of timers that haven't fired.
func (s *Simulated) ActiveTimers() int {
	s.mu.RLock()
	defer s.mu.RUnlock()

	return len(s.scheduled)
}

// WaitForTimers waits until the clock has at least n scheduled timers.
func (s *Simulated) WaitForTimers(n int) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.init()

	for len(s.scheduled) < n {
		s.cond.Wait()
	}
}

// Now returns the current virtual time.
func (s *Simulated) Now() AbsTime {
	s.mu.RLock()
	defer s.mu.RUnlock()

	return s.now
}

// Sleep blocks until the clock has advanced by d.
func (s *Simulated) Sleep(d time.Duration) {
	<-s.After(d)
}

// NewTimer creates a timer which fires when the clock has advanced by d.
func (s *Simulated) NewTimer(d time.Duration) ChanTimer {
	s.mu.Lock()
	defer s.mu.Unlock()

	ch := make(chan AbsTime, 1)
	var timer *simTimer
	timer = s.schedule(d, func() { ch <- timer.at })
	timer.ch = ch
	return timer
}

// After returns a channel which receives the current time after the clock
// has advanced by d.
func (s *Simulated) After(d time.Duration) <-chan AbsTime {
	return s.NewTimer(d).C()
}

// AfterFunc runs fn after the clock has advanced by d. Unlike with the system
// clock, fn runs on the goroutine that calls Run.
func (s *Simulated) AfterFunc(d time.Duration, fn func()) Timer {
	s.mu.Lock()
	defer s.mu.Unlock()

	return s.schedule(d, fn)
}

func (s *Simulated) schedule(d time.Duration, fn func()) *simTimer {
	s.init()

	at := s.now.Add(d)
	ev := &simTimer{do: fn, at: at, s: s}
	heap.Push(&s.scheduled, ev)
	s.cond.Broadcast()
	return ev
}

func (ev *simTimer) Stop() bool {
	ev.s.mu.Lock()
	defer ev.s.mu.Unlock()

	if ev.index < 0 {
		return false
	}
	heap.Remove(&ev.s.scheduled, ev.index)
	ev.s.cond.Broadcast()
	ev.index = -1
	return true
}

func (ev *simTimer) Reset(d time.Duration) {
	if ev.ch == nil {
		panic("mclock: Reset() on timer created by AfterFunc")
	}

	ev.s.mu.Lock()
	defer ev.s.mu.Unlock()
	ev.at = ev.s.now.Add(d)
	if ev.index < 0 {
		heap.Push(&ev.s.scheduled, ev) // already expired
	} else {
		heap.Fix(&ev.s.scheduled, ev.index) // hasn't fired yet, reschedule
	}
	ev.s.cond.Broadcast()
}

func (ev *simTimer) C() <-chan AbsTime {
	if ev.ch == nil {
		panic("mclock: C() on timer created by AfterFunc")
	}
	return ev.ch
}

type simTimerHeap []*simTimer

func (h *simTimerHeap) Len() int {
	return len(*h)
}

func (h *simTimerHeap) Less(i, j int) bool {
	return (*h)[i].at < (*h)[j].at
}

func (h *simTimerHeap) Swap(i, j int) {
	(*h)[i], (*h)[j] = (*h)[j], (*h)[i]
	(*h)[i].index = i
	(*h)[j].index = j
}

func (h *simTimerHeap) Push(x interface{}) {
	t := x.(*simTimer)
	t.index = len(*h)
	*h = append(*h, t)
}

func (h *simTimerHeap) Pop() interface{} {
	end := len(*h) - 1
	t := (*h)[end]
	t.index = -1
	(*h)[end] = nil
	*h = (*h)[:end]
	return t
}
