// Copyright 2014 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build darwin || dragonfly || freebsd || linux || netbsd || openbsd || solaris
// +build darwin dragonfly freebsd linux netbsd openbsd solaris

package vfs

import (
	"io"
	"os"
	"sync"

	"github.com/cockroachdb/errors"
	"golang.org/x/sys/unix"
)

var lockedFiles struct {
	mu struct {
		sync.Mutex
		files map[string]bool
	}
}

// lockCloser hides all of an os.File's methods, except for Close.
type lockCloser struct {
	name string
	f    *os.File
}

func (l lockCloser) Close() error {
	lockedFiles.mu.Lock()
	defer lockedFiles.mu.Unlock()
	if !lockedFiles.mu.files[l.name] {
		panic(errors.Errorf("lock file %q is not locked", l.name))
	}
	delete(lockedFiles.mu.files, l.name)

	return l.f.Close()
}

func (defaultFS) Lock(name string) (io.Closer, error) {
	lockedFiles.mu.Lock()
	defer lockedFiles.mu.Unlock()
	if lockedFiles.mu.files == nil {
		lockedFiles.mu.files = map[string]bool{}
	}
	if lockedFiles.mu.files[name] {
		return nil, errors.New("lock held by current process")
	}

	f, err := os.Create(name)
	if err != nil {
		return nil, err
	}
	spec := unix.Flock_t{
		Type:   unix.F_WRLCK,
		Whence: io.SeekStart,
		Start:  0,
		Len:    0, // 0 means to lock the entire file.
		Pid:    int32(os.Getpid()),
	}
	if err := unix.FcntlFlock(f.Fd(), unix.F_SETLK, &spec); err != nil {
		f.Close()
		return nil, err
	}
	lockedFiles.mu.files[name] = true
	return lockCloser{name, f}, nil
}
