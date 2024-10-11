//go:build !deadlock
// +build !deadlock

// Package sync may be internalized (made private) in future  releases.
// XXX Deprecated.
package sync

import "sync"

// A Mutex is a mutual exclusion lock.
type Mutex struct {
	sync.Mutex
}

// An RWMutex is a reader/writer mutual exclusion lock.
type RWMutex struct {
	sync.RWMutex
}
