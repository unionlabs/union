// Copyright 2021 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package vfs

import (
	"io"
	"os"
	"sync"
	"sync/atomic"
	"syscall"

	"github.com/cockroachdb/errors"
)

// OnDiskFull wraps the provided FS with an FS that examines returned errors,
// looking for ENOSPC errors. It invokes the provided callback when the
// underlying filesystem returns an error signifying the storage is out of
// disk space.
//
// All new writes to the filesystem are blocked while the callback executes,
// so care must be taken to avoid expensive work from within the callback.
//
// Once the callback completes, any write-oriented operations that encountered
// ENOSPC are retried exactly once. Once the callback completes, it will not
// be invoked again until a new operation that began after the callback
// returned encounters an ENOSPC error.
//
// OnDiskFull may be used to automatically manage a ballast file, which is
// removed from the filesystem from within the callback. Note that if managing
// a ballast, the caller should maintain a reference to the inner FS and
// remove the ballast on the unwrapped FS.
func OnDiskFull(fs FS, fn func()) FS {
	newFS := &enospcFS{inner: fs}
	newFS.mu.Cond.L = &newFS.mu.Mutex
	newFS.mu.onDiskFull = fn
	return newFS
}

type enospcFS struct {
	inner FS
	// generation is a monotonically increasing number that encodes the
	// current state of ENOSPC error handling. Incoming writes are
	// organized into generations to provide strong guarantees on when the
	// disk full callback is invoked. The callback is invoked once per
	// write generation.
	//
	// Special significance is given to the parity of this generation
	// field to optimize incoming writes in the normal state, which only
	// need to perform a single atomic load. If generation is odd, an
	// ENOSPC error is being actively handled. The generations associated
	// with writes are always even.
	//
	// The lifecycle of a write is:
	//
	// 1. Atomically load the current generation.
	//    a. If it's even, this is the write's generation number.
	//    b. If it's odd, an ENOSPC was recently encountered and the
	//       corresponding invocation of the disk full callback has not
	//       yet completed. The write must wait until the callback has
	//       completed and generation is updated to an even number, which
	//       becomes the write's generation number.
	// 2. Perform the write. If it encounters no error or an error other
	//    than ENOSPC, the write returns and proceeds no further in this
	//    lifecycle.
	// 3. Handle ENOSPC. If the write encounters ENOSPC, the callback must
	//    be invoked for the write's generation. The write's goroutine
	//    acquires the FS's mutex.
	//    a. If the FS's current generation is still equal to the write's
	//       generation, the write is the first write of its generation to
	//       encounter ENOSPC. It increments the FS's current generation
	//       to an odd number, signifying that an ENOSPC is being handled
	//       and invokes the callback.
	//    b. If the FS's current generation has changed, some other write
	//       from the same generation encountered an ENOSPC first. This
	//       write waits on the condition variable until the FS's current
	//       generation is updated indicating that the generation's
	//       callback invocation has completed.
	// 3. Retry the write once. The callback for the write's generation
	//    has completed, either by this write's goroutine or another's.
	//    The write may proceed with the expectation that the callback
	//    remedied the full disk by freeing up disk space and an ENOSPC
	//    should not be encountered again for at least a few minutes. If
	//    we do encounter another ENOSPC on the retry, the callback was
	//    unable to remedy the full disk and another retry won't be
	//    useful. Any error, including ENOSPC, during the retry is
	//    returned without further handling.  None of the retries invoke
	//    the callback.
	//
	// This scheme has a few nice properties:
	// * Once the disk-full callback completes, it won't be invoked
	//   again unless a write that started strictly later encounters an
	//   ENOSPC. This is convenient if the callback strives to 'fix' the
	//   full disk, for example, by removing a ballast file. A new
	//   invocation of the callback guarantees a new problem.
	// * Incoming writes block if there's an unhandled ENOSPC. Some
	//   writes, like WAL or MANIFEST fsyncs, are fatal if they encounter
	//   an ENOSPC.
	generation atomic.Uint32
	mu         struct {
		sync.Mutex
		sync.Cond
		onDiskFull func()
	}
}

// Unwrap returns the underlying FS. This may be called by vfs.Root to access
// the underlying filesystem.
func (fs *enospcFS) Unwrap() FS {
	return fs.inner
}

// waitUntilReady is called before every FS or File operation that
// might return ENOSPC. If an ENOSPC was encountered and the corresponding
// invocation of the `onDiskFull` callback has not yet returned,
// waitUntilReady blocks until the callback returns. The returned generation
// is always even.
func (fs *enospcFS) waitUntilReady() uint32 {
	gen := fs.generation.Load()
	if gen%2 == 0 {
		// An even generation indicates that we're not currently handling an
		// ENOSPC. Allow the write to proceed.
		return gen
	}

	// We're currently handling an ENOSPC error. Wait on the condition
	// variable until we're not handling an ENOSPC.
	fs.mu.Lock()
	defer fs.mu.Unlock()

	// Load the generation again with fs.mu locked.
	gen = fs.generation.Load()
	for gen%2 == 1 {
		fs.mu.Wait()
		gen = fs.generation.Load()
	}
	return gen
}

func (fs *enospcFS) handleENOSPC(gen uint32) {
	fs.mu.Lock()
	defer fs.mu.Unlock()

	currentGeneration := fs.generation.Load()

	// If the current generation is still `gen`, this is the first goroutine
	// to hit an ENOSPC within this write generation, so this goroutine is
	// responsible for invoking the callback.
	if currentGeneration == gen {
		// Increment the generation to an odd number, indicating that the FS
		// is out-of-disk space and incoming writes should pause and wait for
		// the next generation before continuing.
		fs.generation.Store(gen + 1)

		func() {
			// Drop the mutex while we invoke the callback, re-acquiring
			// afterwards.
			fs.mu.Unlock()
			defer fs.mu.Lock()
			fs.mu.onDiskFull()
		}()

		// Update the current generation again to an even number, indicating
		// that the callback has completed for the write generation `gen`.
		fs.generation.Store(gen + 2)
		fs.mu.Broadcast()
		return
	}

	// The current generation has already been incremented, so either the
	// callback is currently being run by another goroutine or it's already
	// completed. Wait for it complete if it hasn't already.
	//
	// The current generation may be updated multiple times, including to an
	// odd number signifying a later write generation has already encountered
	// ENOSPC. In that case, the callback was not able to remedy the full disk
	// and waiting is unlikely to be helpful.  Continuing to wait risks
	// blocking an unbounded number of generations.  Retrying and bubbling the
	// ENOSPC up might be helpful if we can abort a large compaction that
	// started before we became more selective about compaction picking, so
	// this loop only waits for this write generation's callback and no
	// subsequent generations' callbacks.
	for currentGeneration == gen+1 {
		fs.mu.Wait()
		currentGeneration = fs.generation.Load()
	}
}

func (fs *enospcFS) Create(name string) (File, error) {
	gen := fs.waitUntilReady()

	f, err := fs.inner.Create(name)

	if err != nil && isENOSPC(err) {
		fs.handleENOSPC(gen)
		f, err = fs.inner.Create(name)
	}
	if f != nil {
		f = &enospcFile{
			fs:    fs,
			inner: f,
		}
	}
	return f, err
}

func (fs *enospcFS) Link(oldname, newname string) error {
	gen := fs.waitUntilReady()

	err := fs.inner.Link(oldname, newname)

	if err != nil && isENOSPC(err) {
		fs.handleENOSPC(gen)
		err = fs.inner.Link(oldname, newname)
	}
	return err
}

func (fs *enospcFS) Open(name string, opts ...OpenOption) (File, error) {
	f, err := fs.inner.Open(name, opts...)
	if f != nil {
		f = &enospcFile{
			fs:    fs,
			inner: f,
		}
	}
	return f, err
}

func (fs *enospcFS) OpenReadWrite(name string, opts ...OpenOption) (File, error) {
	f, err := fs.inner.OpenReadWrite(name, opts...)
	if f != nil {
		f = &enospcFile{
			fs:    fs,
			inner: f,
		}
	}
	return f, err
}

func (fs *enospcFS) OpenDir(name string) (File, error) {
	f, err := fs.inner.OpenDir(name)
	if f != nil {
		f = &enospcFile{
			fs:    fs,
			inner: f,
		}
	}
	return f, err
}

func (fs *enospcFS) Remove(name string) error {
	gen := fs.waitUntilReady()

	err := fs.inner.Remove(name)

	if err != nil && isENOSPC(err) {
		fs.handleENOSPC(gen)
		err = fs.inner.Remove(name)
	}
	return err
}

func (fs *enospcFS) RemoveAll(name string) error {
	gen := fs.waitUntilReady()

	err := fs.inner.RemoveAll(name)

	if err != nil && isENOSPC(err) {
		fs.handleENOSPC(gen)
		err = fs.inner.RemoveAll(name)
	}
	return err
}

func (fs *enospcFS) Rename(oldname, newname string) error {
	gen := fs.waitUntilReady()

	err := fs.inner.Rename(oldname, newname)

	if err != nil && isENOSPC(err) {
		fs.handleENOSPC(gen)
		err = fs.inner.Rename(oldname, newname)
	}
	return err
}

func (fs *enospcFS) ReuseForWrite(oldname, newname string) (File, error) {
	gen := fs.waitUntilReady()

	f, err := fs.inner.ReuseForWrite(oldname, newname)

	if err != nil && isENOSPC(err) {
		fs.handleENOSPC(gen)
		f, err = fs.inner.ReuseForWrite(oldname, newname)
	}

	if f != nil {
		f = &enospcFile{
			fs:    fs,
			inner: f,
		}
	}
	return f, err
}

func (fs *enospcFS) MkdirAll(dir string, perm os.FileMode) error {
	gen := fs.waitUntilReady()

	err := fs.inner.MkdirAll(dir, perm)

	if err != nil && isENOSPC(err) {
		fs.handleENOSPC(gen)
		err = fs.inner.MkdirAll(dir, perm)
	}
	return err
}

func (fs *enospcFS) Lock(name string) (io.Closer, error) {
	gen := fs.waitUntilReady()

	closer, err := fs.inner.Lock(name)

	if err != nil && isENOSPC(err) {
		fs.handleENOSPC(gen)
		closer, err = fs.inner.Lock(name)
	}
	return closer, err
}

func (fs *enospcFS) List(dir string) ([]string, error) {
	return fs.inner.List(dir)
}

func (fs *enospcFS) Stat(name string) (os.FileInfo, error) {
	return fs.inner.Stat(name)
}

func (fs *enospcFS) PathBase(path string) string {
	return fs.inner.PathBase(path)
}

func (fs *enospcFS) PathJoin(elem ...string) string {
	return fs.inner.PathJoin(elem...)
}

func (fs *enospcFS) PathDir(path string) string {
	return fs.inner.PathDir(path)
}

func (fs *enospcFS) GetDiskUsage(path string) (DiskUsage, error) {
	return fs.inner.GetDiskUsage(path)
}

type enospcFile struct {
	fs    *enospcFS
	inner File
}

var _ File = (*enospcFile)(nil)

func (f *enospcFile) Close() error {
	return f.inner.Close()
}

func (f *enospcFile) Read(p []byte) (n int, err error) {
	return f.inner.Read(p)
}

func (f *enospcFile) ReadAt(p []byte, off int64) (n int, err error) {
	return f.inner.ReadAt(p, off)
}

func (f *enospcFile) Write(p []byte) (n int, err error) {
	gen := f.fs.waitUntilReady()

	n, err = f.inner.Write(p)

	if err != nil && isENOSPC(err) {
		f.fs.handleENOSPC(gen)
		var n2 int
		n2, err = f.inner.Write(p[n:])
		n += n2
	}
	return n, err
}

func (f *enospcFile) WriteAt(p []byte, ofs int64) (n int, err error) {
	gen := f.fs.waitUntilReady()

	n, err = f.inner.WriteAt(p, ofs)

	if err != nil && isENOSPC(err) {
		f.fs.handleENOSPC(gen)
		var n2 int
		n2, err = f.inner.WriteAt(p[n:], ofs+int64(n))
		n += n2
	}
	return n, err
}

func (f *enospcFile) Prefetch(offset, length int64) error {
	return f.inner.Prefetch(offset, length)
}

func (f *enospcFile) Preallocate(offset, length int64) error {
	return f.inner.Preallocate(offset, length)
}

func (f *enospcFile) Stat() (os.FileInfo, error) {
	return f.inner.Stat()
}

func (f *enospcFile) Sync() error {
	gen := f.fs.waitUntilReady()

	err := f.inner.Sync()

	if err != nil && isENOSPC(err) {
		f.fs.handleENOSPC(gen)

		// NB: It is NOT safe to retry the Sync. See the PostgreSQL
		// 'fsyncgate' discussion. A successful Sync after a failed one does
		// not provide any guarantees and (always?) loses the unsynced writes.
		// We need to bubble the error up and hope we weren't syncing a WAL or
		// MANIFEST, because we'll have no choice but to crash. Errors while
		// syncing an sstable will result in a failed flush/compaction, and
		// the relevant sstable(s) will be marked as obsolete and deleted.
		// See: https://lwn.net/Articles/752063/
	}
	return err
}

func (f *enospcFile) SyncData() error {
	return f.inner.SyncData()
}

func (f *enospcFile) SyncTo(length int64) (fullSync bool, err error) {
	return f.inner.SyncTo(length)
}

func (f *enospcFile) Fd() uintptr {
	return f.inner.Fd()
}

var _ FS = (*enospcFS)(nil)

func isENOSPC(err error) bool {
	err = errors.UnwrapAll(err)
	e, ok := err.(syscall.Errno)
	return ok && e == syscall.ENOSPC
}
