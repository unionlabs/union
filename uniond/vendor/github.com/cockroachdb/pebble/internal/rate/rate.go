// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

// Package rate provides a rate limiter.
package rate // import "github.com/cockroachdb/pebble/internal/rate"

import (
	"sync"
	"time"

	"github.com/cockroachdb/tokenbucket"
)

// A Limiter controls how frequently events are allowed to happen.
// It implements a "token bucket" of size b, initially full and refilled
// at rate r tokens per second.
//
// Informally, in any large enough time interval, the Limiter limits the
// rate to r tokens per second, with a maximum burst size of b events.
//
// Limiter is thread-safe.
type Limiter struct {
	mu struct {
		sync.Mutex
		tb    tokenbucket.TokenBucket
		rate  float64
		burst float64
	}
	sleepFn func(d time.Duration)
}

// NewLimiter returns a new Limiter that allows events up to rate r and permits
// bursts of at most b tokens.
func NewLimiter(r float64, b float64) *Limiter {
	l := &Limiter{}
	l.mu.tb.Init(tokenbucket.TokensPerSecond(r), tokenbucket.Tokens(b))
	l.mu.rate = r
	l.mu.burst = b
	return l
}

// NewLimiterWithCustomTime returns a new Limiter that allows events up to rate
// r and permits bursts of at most b tokens. The limiter uses the given
// functions to retrieve the current time and to sleep (useful for testing).
func NewLimiterWithCustomTime(
	r float64, b float64, nowFn func() time.Time, sleepFn func(d time.Duration),
) *Limiter {
	l := &Limiter{}
	l.mu.tb.InitWithNowFn(tokenbucket.TokensPerSecond(r), tokenbucket.Tokens(b), nowFn)
	l.mu.rate = r
	l.mu.burst = b
	l.sleepFn = sleepFn
	return l
}

// Wait sleeps until enough tokens are available. If n is more than the burst,
// the token bucket will go into debt, delaying future operations.
func (l *Limiter) Wait(n float64) {
	for {
		l.mu.Lock()
		ok, d := l.mu.tb.TryToFulfill(tokenbucket.Tokens(n))
		l.mu.Unlock()
		if ok {
			return
		}
		if l.sleepFn != nil {
			l.sleepFn(d)
		} else {
			time.Sleep(d)
		}
	}
}

// Remove removes tokens for an operation that bypassed any waiting; it can put
// the token bucket into debt, delaying future operations.
func (l *Limiter) Remove(n float64) {
	l.mu.Lock()
	defer l.mu.Unlock()
	l.mu.tb.Adjust(-tokenbucket.Tokens(n))
}

// Rate returns the current rate limit.
func (l *Limiter) Rate() float64 {
	l.mu.Lock()
	defer l.mu.Unlock()
	return l.mu.rate
}

// SetRate updates the rate limit.
func (l *Limiter) SetRate(r float64) {
	l.mu.Lock()
	defer l.mu.Unlock()
	l.mu.tb.UpdateConfig(tokenbucket.TokensPerSecond(r), tokenbucket.Tokens(l.mu.burst))
	l.mu.rate = r
}
