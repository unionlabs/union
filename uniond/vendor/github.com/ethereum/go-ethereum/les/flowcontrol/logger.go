// Copyright 2019 The go-ethereum Authors
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

package flowcontrol

import (
	"fmt"
	"time"

	"github.com/ethereum/go-ethereum/common/mclock"
)

// logger collects events in string format and discards events older than the
// "keep" parameter
type logger struct {
	events           map[uint64]logEvent
	writePtr, delPtr uint64
	keep             time.Duration
}

// logEvent describes a single event
type logEvent struct {
	time  mclock.AbsTime
	event string
}

// newLogger creates a new logger
func newLogger(keep time.Duration) *logger {
	return &logger{
		events: make(map[uint64]logEvent),
		keep:   keep,
	}
}

// add adds a new event and discards old events if possible
func (l *logger) add(now mclock.AbsTime, event string) {
	keepAfter := now - mclock.AbsTime(l.keep)
	for l.delPtr < l.writePtr && l.events[l.delPtr].time <= keepAfter {
		delete(l.events, l.delPtr)
		l.delPtr++
	}
	l.events[l.writePtr] = logEvent{now, event}
	l.writePtr++
}

// dump prints all stored events
func (l *logger) dump(now mclock.AbsTime) {
	for i := l.delPtr; i < l.writePtr; i++ {
		e := l.events[i]
		fmt.Println(time.Duration(e.time-now), e.event)
	}
}
