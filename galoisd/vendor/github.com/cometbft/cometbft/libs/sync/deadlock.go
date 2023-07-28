//go:build deadlock
// +build deadlock

package sync

import (
	deadlock "github.com/sasha-s/go-deadlock"
)

// A Mutex is a mutual exclusion lock.
type Mutex struct {
	deadlock.Mutex
}

// An RWMutex is a reader/writer mutual exclusion lock.
type RWMutex struct {
	deadlock.RWMutex
}
