// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

// Package vfstest provides facilities for interacting with or faking
// filesystems during tests and benchmarks.
package vfstest

import (
	"os"

	"github.com/cockroachdb/pebble/vfs"
)

// DiscardFile implements vfs.File but discards all written data and reads
// without mutating input buffers.
var DiscardFile vfs.File = (*discardFile)(nil)

type discardFile struct{}

func (*discardFile) Close() error                                   { return nil }
func (*discardFile) Read(p []byte) (int, error)                     { return len(p), nil }
func (*discardFile) ReadAt(p []byte, off int64) (int, error)        { return len(p), nil }
func (*discardFile) Write(p []byte) (int, error)                    { return len(p), nil }
func (*discardFile) WriteAt(p []byte, ofs int64) (int, error)       { return len(p), nil }
func (*discardFile) Preallocate(offset, length int64) error         { return nil }
func (*discardFile) Stat() (os.FileInfo, error)                     { return nil, nil }
func (*discardFile) Sync() error                                    { return nil }
func (*discardFile) SyncTo(length int64) (fullSync bool, err error) { return false, nil }
func (*discardFile) SyncData() error                                { return nil }
func (*discardFile) Prefetch(offset int64, length int64) error      { return nil }
func (*discardFile) Fd() uintptr                                    { return 0 }
