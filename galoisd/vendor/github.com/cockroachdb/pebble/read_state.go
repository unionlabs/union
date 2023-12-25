// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import "sync/atomic"

// readState encapsulates the state needed for reading (the current version and
// list of memtables). Loading the readState is done without grabbing
// DB.mu. Instead, a separate DB.readState.RWMutex is used for
// synchronization. This mutex solely covers the current readState object which
// means it is rarely or ever contended.
//
// Note that various fancy lock-free mechanisms can be imagined for loading the
// readState, but benchmarking showed the ones considered to purely be
// pessimizations. The RWMutex version is a single atomic increment for the
// RLock and an atomic decrement for the RUnlock. It is difficult to do better
// than that without something like thread-local storage which isn't available
// in Go.
type readState struct {
	db        *DB
	refcnt    atomic.Int32
	current   *version
	memtables flushableList
}

// ref adds a reference to the readState.
func (s *readState) ref() {
	s.refcnt.Add(1)
}

// unref removes a reference to the readState. If this was the last reference,
// the reference the readState holds on the version is released. Requires DB.mu
// is NOT held as version.unref() will acquire it. See unrefLocked() if DB.mu
// is held by the caller.
func (s *readState) unref() {
	if s.refcnt.Add(-1) != 0 {
		return
	}
	s.current.Unref()
	for _, mem := range s.memtables {
		mem.readerUnref(true)
	}

	// The last reference to the readState was released. Check to see if there
	// are new obsolete tables to delete.
	s.db.maybeScheduleObsoleteTableDeletion()
}

// unrefLocked removes a reference to the readState. If this was the last
// reference, the reference the readState holds on the version is
// released.
//
// DB.mu must be held. See unref() if DB.mu is NOT held by the caller.
func (s *readState) unrefLocked() {
	if s.refcnt.Add(-1) != 0 {
		return
	}
	s.current.UnrefLocked()
	for _, mem := range s.memtables {
		mem.readerUnrefLocked(true)
	}

	// In this code path, the caller is responsible for scheduling obsolete table
	// deletion as necessary.
}

// loadReadState returns the current readState. The returned readState must be
// unreferenced when the caller is finished with it.
func (d *DB) loadReadState() *readState {
	d.readState.RLock()
	state := d.readState.val
	state.ref()
	d.readState.RUnlock()
	return state
}

// updateReadStateLocked creates a new readState from the current version and
// list of memtables. Requires DB.mu is held. If checker is not nil, it is
// called after installing the new readState.
func (d *DB) updateReadStateLocked(checker func(*DB) error) {
	s := &readState{
		db:        d,
		current:   d.mu.versions.currentVersion(),
		memtables: d.mu.mem.queue,
	}
	s.refcnt.Store(1)
	s.current.Ref()
	for _, mem := range s.memtables {
		mem.readerRef()
	}

	d.readState.Lock()
	old := d.readState.val
	d.readState.val = s
	d.readState.Unlock()
	if checker != nil {
		if err := checker(d); err != nil {
			d.opts.Logger.Fatalf("checker failed with error: %s", err)
		}
	}
	if old != nil {
		old.unrefLocked()
	}
}
