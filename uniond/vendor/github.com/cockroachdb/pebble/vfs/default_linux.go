// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build linux
// +build linux

package vfs

import (
	"os"
	"syscall"

	"github.com/cockroachdb/errors"
	"golang.org/x/sys/unix"
)

func wrapOSFileImpl(f *os.File) File {
	lf := &linuxFile{File: f, fd: f.Fd()}
	if lf.fd != InvalidFd {
		lf.useSyncRange = isSyncRangeSupported(lf.fd)
	}
	return lf
}

func (defaultFS) OpenDir(name string) (File, error) {
	f, err := os.OpenFile(name, syscall.O_CLOEXEC, 0)
	if err != nil {
		return nil, errors.WithStack(err)
	}
	return &linuxDir{f}, nil
}

// Assert that linuxFile and linuxDir implement vfs.File.
var (
	_ File = (*linuxDir)(nil)
	_ File = (*linuxFile)(nil)
)

type linuxDir struct {
	*os.File
}

func (d *linuxDir) Prefetch(offset int64, length int64) error      { return nil }
func (d *linuxDir) Preallocate(offset, length int64) error         { return nil }
func (d *linuxDir) SyncData() error                                { return d.Sync() }
func (d *linuxDir) SyncTo(offset int64) (fullSync bool, err error) { return false, nil }

type linuxFile struct {
	*os.File
	fd           uintptr
	useSyncRange bool
}

func (f *linuxFile) Prefetch(offset int64, length int64) error {
	_, _, err := unix.Syscall(unix.SYS_READAHEAD, uintptr(f.fd), uintptr(offset), uintptr(length))
	return err
}

func (f *linuxFile) Preallocate(offset, length int64) error {
	return unix.Fallocate(int(f.fd), unix.FALLOC_FL_KEEP_SIZE, offset, length)
}

func (f *linuxFile) SyncData() error {
	return unix.Fdatasync(int(f.fd))
}

func (f *linuxFile) SyncTo(offset int64) (fullSync bool, err error) {
	if !f.useSyncRange {
		// Use fdatasync, which does provide persistence guarantees but won't
		// update all file metadata. From the `fdatasync` man page:
		//
		// fdatasync() is similar to fsync(), but does not flush modified
		// metadata unless that metadata is needed in order to allow a
		// subsequent data retrieval to be correctly handled. For example,
		// changes to st_atime or st_mtime (respectively, time of last access
		// and time of last modification; see stat(2)) do not require flushing
		// because they are not necessary for a subsequent data read to be
		// handled correctly. On the other hand, a change to the file size
		// (st_size, as made by say ftruncate(2)), would require a metadata
		// flush.
		if err = unix.Fdatasync(int(f.fd)); err != nil {
			return false, err
		}
		return true, nil
	}

	const (
		waitBefore = 0x1
		write      = 0x2
		// waitAfter = 0x4
	)

	// By specifying write|waitBefore for the flags, we're instructing
	// SyncFileRange to a) wait for any outstanding data being written to finish,
	// and b) to queue any other dirty data blocks in the range [0,offset] for
	// writing. The actual writing of this data will occur asynchronously. The
	// use of `waitBefore` is to limit how much dirty data is allowed to
	// accumulate. Linux sometimes behaves poorly when a large amount of dirty
	// data accumulates, impacting other I/O operations.
	return false, unix.SyncFileRange(int(f.fd), 0, offset, write|waitBefore)
}

type syncFileRange func(fd int, off int64, n int64, flags int) (err error)

// sync_file_range depends on both the filesystem, and the broader kernel
// support. In particular, Windows Subsystem for Linux does not support
// sync_file_range, even when used with ext{2,3,4}. syncRangeSmokeTest performs
// a test of of sync_file_range, returning false on ENOSYS, and true otherwise.
func syncRangeSmokeTest(fd uintptr, syncFn syncFileRange) bool {
	err := syncFn(int(fd), 0 /* offset */, 0 /* nbytes */, 0 /* flags */)
	return err != unix.ENOSYS
}

func isSyncRangeSupported(fd uintptr) bool {
	var stat unix.Statfs_t
	if err := unix.Fstatfs(int(fd), &stat); err != nil {
		return false
	}

	// Allowlist which filesystems we allow using sync_file_range with as some
	// filesystems treat that syscall as a noop (notably ZFS). A allowlist is
	// used instead of a denylist in order to have a more graceful failure mode
	// in case a filesystem we haven't tested is encountered. Currently only
	// ext2/3/4 are known to work properly.
	const extMagic = 0xef53
	switch stat.Type {
	case extMagic:
		return syncRangeSmokeTest(fd, unix.SyncFileRange)
	}
	return false
}
