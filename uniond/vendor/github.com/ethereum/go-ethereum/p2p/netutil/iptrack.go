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

package netutil

import (
	"time"

	"github.com/ethereum/go-ethereum/common/mclock"
)

// IPTracker predicts the external endpoint, i.e. IP address and port, of the local host
// based on statements made by other hosts.
type IPTracker struct {
	window          time.Duration
	contactWindow   time.Duration
	minStatements   int
	clock           mclock.Clock
	statements      map[string]ipStatement
	contact         map[string]mclock.AbsTime
	lastStatementGC mclock.AbsTime
	lastContactGC   mclock.AbsTime
}

type ipStatement struct {
	endpoint string
	time     mclock.AbsTime
}

// NewIPTracker creates an IP tracker.
//
// The window parameters configure the amount of past network events which are kept. The
// minStatements parameter enforces a minimum number of statements which must be recorded
// before any prediction is made. Higher values for these parameters decrease 'flapping' of
// predictions as network conditions change. Window duration values should typically be in
// the range of minutes.
func NewIPTracker(window, contactWindow time.Duration, minStatements int) *IPTracker {
	return &IPTracker{
		window:        window,
		contactWindow: contactWindow,
		statements:    make(map[string]ipStatement),
		minStatements: minStatements,
		contact:       make(map[string]mclock.AbsTime),
		clock:         mclock.System{},
	}
}

// PredictFullConeNAT checks whether the local host is behind full cone NAT. It predicts by
// checking whether any statement has been received from a node we didn't contact before
// the statement was made.
func (it *IPTracker) PredictFullConeNAT() bool {
	now := it.clock.Now()
	it.gcContact(now)
	it.gcStatements(now)
	for host, st := range it.statements {
		if c, ok := it.contact[host]; !ok || c > st.time {
			return true
		}
	}
	return false
}

// PredictEndpoint returns the current prediction of the external endpoint.
func (it *IPTracker) PredictEndpoint() string {
	it.gcStatements(it.clock.Now())

	// The current strategy is simple: find the endpoint with most statements.
	counts := make(map[string]int, len(it.statements))
	maxcount, max := 0, ""
	for _, s := range it.statements {
		c := counts[s.endpoint] + 1
		counts[s.endpoint] = c
		if c > maxcount && c >= it.minStatements {
			maxcount, max = c, s.endpoint
		}
	}
	return max
}

// AddStatement records that a certain host thinks our external endpoint is the one given.
func (it *IPTracker) AddStatement(host, endpoint string) {
	now := it.clock.Now()
	it.statements[host] = ipStatement{endpoint, now}
	if time.Duration(now-it.lastStatementGC) >= it.window {
		it.gcStatements(now)
	}
}

// AddContact records that a packet containing our endpoint information has been sent to a
// certain host.
func (it *IPTracker) AddContact(host string) {
	now := it.clock.Now()
	it.contact[host] = now
	if time.Duration(now-it.lastContactGC) >= it.contactWindow {
		it.gcContact(now)
	}
}

func (it *IPTracker) gcStatements(now mclock.AbsTime) {
	it.lastStatementGC = now
	cutoff := now.Add(-it.window)
	for host, s := range it.statements {
		if s.time < cutoff {
			delete(it.statements, host)
		}
	}
}

func (it *IPTracker) gcContact(now mclock.AbsTime) {
	it.lastContactGC = now
	cutoff := now.Add(-it.contactWindow)
	for host, ct := range it.contact {
		if ct < cutoff {
			delete(it.contact, host)
		}
	}
}
