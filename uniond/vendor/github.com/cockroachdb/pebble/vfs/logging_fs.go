// Copyright 2021 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package vfs

import (
	"fmt"
	"io"
	"os"
)

// WithLogging wraps an FS and logs filesystem modification operations to the
// given logFn.
func WithLogging(fs FS, logFn LogFn) FS {
	return &loggingFS{
		FS:    fs,
		logFn: logFn,
	}
}

// LogFn is a function that is used to capture a log when WithLogging is used.
type LogFn func(fmt string, args ...interface{})

type loggingFS struct {
	FS
	logFn LogFn
}

var _ FS = (*loggingFS)(nil)

func (fs *loggingFS) Create(name string) (File, error) {
	fs.logFn("create: %s", name)
	f, err := fs.FS.Create(name)
	if err != nil {
		return nil, err
	}
	return newLoggingFile(f, name, fs.logFn), nil
}

func (fs *loggingFS) Open(name string, opts ...OpenOption) (File, error) {
	var optsStr string
	if len(opts) > 0 {
		optsStr = " (options:"
		for i := range opts {
			optsStr += fmt.Sprintf(" %T", opts[i])
		}
		optsStr += ")"
	}
	fs.logFn("open: %s%s", name, optsStr)
	f, err := fs.FS.Open(name, opts...)
	if err != nil {
		return nil, err
	}
	return newLoggingFile(f, name, fs.logFn), nil
}

func (fs *loggingFS) OpenReadWrite(name string, opts ...OpenOption) (File, error) {
	fs.logFn("open-read-write: %s", name)
	f, err := fs.FS.OpenReadWrite(name, opts...)
	if err != nil {
		return nil, err
	}
	return newLoggingFile(f, name, fs.logFn), nil
}

func (fs *loggingFS) Link(oldname, newname string) error {
	fs.logFn("link: %s -> %s", oldname, newname)
	return fs.FS.Link(oldname, newname)
}

func (fs *loggingFS) OpenDir(name string) (File, error) {
	fs.logFn("open-dir: %s", name)
	f, err := fs.FS.OpenDir(name)
	if err != nil {
		return nil, err
	}
	return newLoggingFile(f, name, fs.logFn), nil
}

func (fs *loggingFS) Rename(oldname, newname string) error {
	fs.logFn("rename: %s -> %s", oldname, newname)
	return fs.FS.Rename(oldname, newname)
}

func (fs *loggingFS) ReuseForWrite(oldname, newname string) (File, error) {
	fs.logFn("reuseForWrite: %s -> %s", oldname, newname)
	f, err := fs.FS.ReuseForWrite(oldname, newname)
	if err != nil {
		return nil, err
	}
	return newLoggingFile(f, newname, fs.logFn), nil
}

func (fs *loggingFS) MkdirAll(dir string, perm os.FileMode) error {
	fs.logFn("mkdir-all: %s %#o", dir, perm)
	return fs.FS.MkdirAll(dir, perm)
}

func (fs *loggingFS) Lock(name string) (io.Closer, error) {
	fs.logFn("lock: %s", name)
	return fs.FS.Lock(name)
}

func (fs loggingFS) Remove(name string) error {
	fs.logFn("remove: %s", name)
	err := fs.FS.Remove(name)
	return err
}

func (fs loggingFS) RemoveAll(name string) error {
	fs.logFn("remove-all: %s", name)
	err := fs.FS.RemoveAll(name)
	return err
}

type loggingFile struct {
	File
	name  string
	logFn LogFn
}

var _ File = (*loggingFile)(nil)

func newLoggingFile(f File, name string, logFn LogFn) *loggingFile {
	return &loggingFile{
		File:  f,
		name:  name,
		logFn: logFn,
	}
}

func (f *loggingFile) Close() error {
	f.logFn("close: %s", f.name)
	return f.File.Close()
}

func (f *loggingFile) Sync() error {
	f.logFn("sync: %s", f.name)
	return f.File.Sync()
}

func (f *loggingFile) SyncData() error {
	f.logFn("sync-data: %s", f.name)
	return f.File.SyncData()
}

func (f *loggingFile) SyncTo(length int64) (fullSync bool, err error) {
	f.logFn("sync-to(%d): %s", length, f.name)
	return f.File.SyncTo(length)
}

func (f *loggingFile) ReadAt(p []byte, offset int64) (int, error) {
	f.logFn("read-at(%d, %d): %s", offset, len(p), f.name)
	return f.File.ReadAt(p, offset)
}

func (f *loggingFile) WriteAt(p []byte, offset int64) (int, error) {
	f.logFn("write-at(%d, %d): %s", offset, len(p), f.name)
	return f.File.WriteAt(p, offset)
}

func (f *loggingFile) Prefetch(offset int64, length int64) error {
	f.logFn("prefetch(%d, %d): %s", offset, length, f.name)
	return f.File.Prefetch(offset, length)
}
