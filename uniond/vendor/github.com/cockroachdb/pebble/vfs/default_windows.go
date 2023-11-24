// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build windows
// +build windows

package vfs

import (
	"os"
	"syscall"

	"github.com/cockroachdb/errors"
)

func wrapOSFileImpl(f *os.File) File {
	return &windowsFile{f}
}

func (defaultFS) OpenDir(name string) (File, error) {
	f, err := os.OpenFile(name, syscall.O_CLOEXEC, 0)
	if err != nil {
		return nil, errors.WithStack(err)
	}
	return &windowsDir{f}, nil
}

// Assert that windowsFile and windowsDir implement vfs.File.
var (
	_ File = (*windowsFile)(nil)
	_ File = (*windowsDir)(nil)
)

type windowsDir struct {
	*os.File
}

func (*windowsDir) Prefetch(offset int64, length int64) error { return nil }
func (*windowsDir) Preallocate(off, length int64) error       { return nil }

// Silently ignore Sync() on Windows. This is the same behavior as
// RocksDB. See port/win/io_win.cc:WinDirectory::Fsync().
func (*windowsDir) Sync() error                                    { return nil }
func (*windowsDir) SyncData() error                                { return nil }
func (*windowsDir) SyncTo(length int64) (fullSync bool, err error) { return false, nil }

type windowsFile struct {
	*os.File
}

func (*windowsFile) Prefetch(offset int64, length int64) error { return nil }
func (*windowsFile) Preallocate(offset, length int64) error    { return nil }

func (f *windowsFile) SyncData() error { return f.Sync() }
func (f *windowsFile) SyncTo(length int64) (fullSync bool, err error) {
	if err = f.Sync(); err != nil {
		return false, err
	}
	return true, nil
}
