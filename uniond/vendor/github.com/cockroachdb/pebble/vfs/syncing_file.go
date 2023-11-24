// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package vfs

import (
	"sync/atomic"

	"github.com/cockroachdb/errors"
)

// SyncingFileOptions holds the options for a syncingFile.
type SyncingFileOptions struct {
	// NoSyncOnClose elides the automatic Sync during Close if it's not possible
	// to sync the remainder of the file in a non-blocking way.
	NoSyncOnClose   bool
	BytesPerSync    int
	PreallocateSize int
}

type syncingFile struct {
	File
	// fd can be InvalidFd if the underlying File does not support it.
	fd              uintptr
	noSyncOnClose   bool
	bytesPerSync    int64
	preallocateSize int64
	// The offset at which dirty data has been written.
	offset atomic.Int64
	// The offset at which data has been synced. Note that if SyncFileRange is
	// being used, the periodic syncing of data during writing will only ever
	// sync up to offset-1MB. This is done to avoid rewriting the tail of the
	// file multiple times, but has the side effect of ensuring that Close will
	// sync the file's metadata.
	syncOffset         atomic.Int64
	preallocatedBlocks int64
}

// NewSyncingFile wraps a writable file and ensures that data is synced
// periodically as it is written. The syncing does not provide persistency
// guarantees for these periodic syncs, but is used to avoid latency spikes if
// the OS automatically decides to write out a large chunk of dirty filesystem
// buffers. The underlying file is fully synced upon close.
func NewSyncingFile(f File, opts SyncingFileOptions) File {
	s := &syncingFile{
		File:            f,
		fd:              f.Fd(),
		noSyncOnClose:   bool(opts.NoSyncOnClose),
		bytesPerSync:    int64(opts.BytesPerSync),
		preallocateSize: int64(opts.PreallocateSize),
	}
	// Ensure a file that is opened and then closed will be synced, even if no
	// data has been written to it.
	s.syncOffset.Store(-1)
	return s
}

// NB: syncingFile.Write is unsafe for concurrent use!
func (f *syncingFile) Write(p []byte) (n int, err error) {
	_ = f.preallocate(f.offset.Load())

	n, err = f.File.Write(p)
	if err != nil {
		return n, errors.WithStack(err)
	}
	// The offset is updated atomically so that it can be accessed safely from
	// Sync.
	f.offset.Add(int64(n))
	if err := f.maybeSync(); err != nil {
		return 0, err
	}
	return n, nil
}

func (f *syncingFile) preallocate(offset int64) error {
	if f.fd == InvalidFd || f.preallocateSize == 0 {
		return nil
	}

	newPreallocatedBlocks := (offset + f.preallocateSize - 1) / f.preallocateSize
	if newPreallocatedBlocks <= f.preallocatedBlocks {
		return nil
	}

	length := f.preallocateSize * (newPreallocatedBlocks - f.preallocatedBlocks)
	offset = f.preallocateSize * f.preallocatedBlocks
	f.preallocatedBlocks = newPreallocatedBlocks
	return f.Preallocate(offset, length)
}

func (f *syncingFile) ratchetSyncOffset(offset int64) {
	for {
		syncOffset := f.syncOffset.Load()
		if syncOffset >= offset {
			return
		}
		if f.syncOffset.CompareAndSwap(syncOffset, offset) {
			return
		}
	}
}

func (f *syncingFile) Sync() error {
	// We update syncOffset (atomically) in order to avoid spurious syncs in
	// maybeSync. Note that even if syncOffset is larger than the current file
	// offset, we still need to call the underlying file's sync for persistence
	// guarantees which are not provided by SyncTo (or by sync_file_range on
	// Linux).
	f.ratchetSyncOffset(f.offset.Load())
	return f.SyncData()
}

func (f *syncingFile) maybeSync() error {
	if f.bytesPerSync <= 0 {
		return nil
	}

	// From the RocksDB source:
	//
	//   We try to avoid sync to the last 1MB of data. For two reasons:
	//   (1) avoid rewrite the same page that is modified later.
	//   (2) for older version of OS, write can block while writing out
	//       the page.
	//   Xfs does neighbor page flushing outside of the specified ranges. We
	//   need to make sure sync range is far from the write offset.
	const syncRangeBuffer = 1 << 20 // 1 MB
	offset := f.offset.Load()
	if offset <= syncRangeBuffer {
		return nil
	}

	const syncRangeAlignment = 4 << 10 // 4 KB
	syncToOffset := offset - syncRangeBuffer
	syncToOffset -= syncToOffset % syncRangeAlignment
	syncOffset := f.syncOffset.Load()
	if syncToOffset < 0 || (syncToOffset-syncOffset) < f.bytesPerSync {
		return nil
	}

	if f.fd == InvalidFd {
		return errors.WithStack(f.Sync())
	}

	// Note that SyncTo will always be called with an offset < atomic.offset.
	// The SyncTo implementation may choose to sync the entire file (i.e. on
	// OSes which do not support syncing a portion of the file).
	fullSync, err := f.SyncTo(syncToOffset)
	if err != nil {
		return errors.WithStack(err)
	}
	if fullSync {
		f.ratchetSyncOffset(offset)
	} else {
		f.ratchetSyncOffset(syncToOffset)
	}
	return nil
}

func (f *syncingFile) Close() error {
	// Sync any data that has been written but not yet synced unless the file
	// has noSyncOnClose option explicitly set.
	//
	// NB: If the file is capable of non-durability-guarantee SyncTos, and the
	// caller has not called Sync since the last write, syncOffset is guaranteed
	// to be less than atomic.offset. This ensures we fall into the below
	// conditional and perform a full sync to durably persist the file.
	if off := f.offset.Load(); off > f.syncOffset.Load() {
		// There's still remaining dirty data.

		if f.noSyncOnClose {
			// If NoSyncOnClose is set, only perform a SyncTo. On linux, SyncTo
			// translates to a non-blocking `sync_file_range` call which
			// provides no persistence guarantee. Since it's non-blocking,
			// there's no latency hit of a blocking sync call, but we still
			// ensure we're not allowing significant dirty data to accumulate.
			if _, err := f.File.SyncTo(off); err != nil {
				return err
			}
			f.ratchetSyncOffset(off)
		} else if err := f.Sync(); err != nil {
			return errors.WithStack(err)
		}
	}
	return errors.WithStack(f.File.Close())
}

// NewSyncingFS wraps a vfs.FS with one that wraps newly created files with
// vfs.NewSyncingFile.
func NewSyncingFS(fs FS, syncOpts SyncingFileOptions) FS {
	return &syncingFS{
		FS:       fs,
		syncOpts: syncOpts,
	}
}

type syncingFS struct {
	FS
	syncOpts SyncingFileOptions
}

var _ FS = (*syncingFS)(nil)

func (fs *syncingFS) Create(name string) (File, error) {
	f, err := fs.FS.Create(name)
	if err != nil {
		return nil, err
	}
	return NewSyncingFile(f, fs.syncOpts), nil
}

func (fs *syncingFS) ReuseForWrite(oldname, newname string) (File, error) {
	// TODO(radu): implement this if needed.
	panic("unimplemented")
}
