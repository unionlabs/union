// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package ackseq

import (
	"sync"
	"sync/atomic"

	"github.com/cockroachdb/errors"
)

const (
	// The window size constants. These values specify a window that can hold ~1m
	// pending unacknowledged sequence numbers using 128 KB of memory.
	windowSize  = 1 << 20
	windowMask  = windowSize - 1
	windowBytes = (windowSize + 7) / 8
)

// S keeps track of the largest sequence number such that all sequence numbers
// in the range [0,v) have been acknowledged.
type S struct {
	next atomic.Uint64
	mu   struct {
		sync.Mutex
		base   uint64
		window [windowBytes]uint8
	}
}

// New creates a new acknowledged sequence tracker with the specified base
// sequence number. All of the sequence numbers in the range [0,base) are
// considered acknowledged. Next() will return base upon first call.
func New(base uint64) *S {
	s := &S{}
	s.next.Store(base)
	s.mu.base = base
	return s
}

// Next returns the next sequence number to use.
func (s *S) Next() uint64 {
	return s.next.Add(1) - 1
}

// Ack acknowledges the specified seqNum, adjusting base as necessary,
// returning the number of newly acknowledged sequence numbers.
func (s *S) Ack(seqNum uint64) (int, error) {
	s.mu.Lock()
	if s.getLocked(seqNum) {
		defer s.mu.Unlock()
		return 0, errors.Errorf(
			"pending acks exceeds window size: %d has been acked, but %d has not",
			errors.Safe(seqNum), errors.Safe(s.mu.base))
	}

	var count int
	s.setLocked(seqNum)
	for s.getLocked(s.mu.base) {
		s.clearLocked(s.mu.base)
		s.mu.base++
		count++
	}
	s.mu.Unlock()
	return count, nil
}

func (s *S) getLocked(seqNum uint64) bool {
	bit := seqNum & windowMask
	return (s.mu.window[bit/8] & (1 << (bit % 8))) != 0
}

func (s *S) setLocked(seqNum uint64) {
	bit := seqNum & windowMask
	s.mu.window[bit/8] |= (1 << (bit % 8))
}

func (s *S) clearLocked(seqNum uint64) {
	bit := seqNum & windowMask
	s.mu.window[bit/8] &^= (1 << (bit % 8))
}
