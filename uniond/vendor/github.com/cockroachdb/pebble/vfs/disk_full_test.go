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
	"testing"
	"time"

	"github.com/cockroachdb/errors"
	"github.com/stretchr/testify/require"
)

var filesystemWriteOps = map[string]func(FS) error{
	"Create": func(fs FS) error {
		_, err := fs.Create("foo")
		return err
	},
	"Lock": func(fs FS) error {
		_, err := fs.Lock("foo")
		return err
	},
	"ReuseForWrite": func(fs FS) error {
		_, err := fs.ReuseForWrite("foo", "bar")
		return err
	},
	"Link":      func(fs FS) error { return fs.Link("foo", "bar") },
	"MkdirAll":  func(fs FS) error { return fs.MkdirAll("foo", os.ModePerm) },
	"Remove":    func(fs FS) error { return fs.Remove("foo") },
	"RemoveAll": func(fs FS) error { return fs.RemoveAll("foo") },
	"Rename":    func(fs FS) error { return fs.Rename("foo", "bar") },
}

func TestOnDiskFull_FS(t *testing.T) {
	for name, fn := range filesystemWriteOps {
		t.Run(name, func(t *testing.T) {
			innerFS := &enospcMockFS{}
			innerFS.enospcs.Store(1)
			var callbackInvocations int
			fs := OnDiskFull(innerFS, func() {
				callbackInvocations++
			})

			// Call this vfs.FS method on the wrapped filesystem. The first
			// call should return ENOSPC. Our registered callback should be
			// invoked, then the method should be retried and return a nil
			// error.
			require.NoError(t, fn(fs))
			require.Equal(t, 1, callbackInvocations)
			// The inner filesystem should be invoked twice because of the
			// retry.
			require.Equal(t, uint32(2), innerFS.invocations.Load())
		})
	}
}

func TestOnDiskFull_File(t *testing.T) {
	t.Run("Write", func(t *testing.T) {
		innerFS := &enospcMockFS{bytesWritten: 6}
		var callbackInvocations int
		fs := OnDiskFull(innerFS, func() {
			callbackInvocations++
		})

		f, err := fs.Create("foo")
		require.NoError(t, err)

		// The next Write should ENOSPC.
		innerFS.enospcs.Store(1)

		// Call the Write method on the wrapped file. The first call should return
		// ENOSPC, but also that six bytes were written. Our registered callback
		// should be invoked, then Write should be retried and return a nil error
		// and five bytes written.
		n, err := f.Write([]byte("hello world"))
		require.NoError(t, err)
		require.Equal(t, 11, n)
		require.Equal(t, 1, callbackInvocations)
		// The inner filesystem should be invoked 3 times. Once during Create
		// and twice during Write.
		require.Equal(t, uint32(3), innerFS.invocations.Load())
	})
	t.Run("Sync", func(t *testing.T) {
		innerFS := &enospcMockFS{bytesWritten: 6}
		var callbackInvocations int
		fs := OnDiskFull(innerFS, func() {
			callbackInvocations++
		})

		f, err := fs.Create("foo")
		require.NoError(t, err)

		// The next Sync should ENOSPC. The callback should be invoked, but a
		// Sync cannot be retried.
		innerFS.enospcs.Store(1)

		err = f.Sync()
		require.Error(t, err)
		require.Equal(t, 1, callbackInvocations)
		// The inner filesystem should be invoked 2 times. Once during Create
		// and once during Sync.
		require.Equal(t, uint32(2), innerFS.invocations.Load())
	})
}

func TestOnDiskFull_Concurrent(t *testing.T) {
	innerFS := &enospcMockFS{
		opDelay: 10 * time.Millisecond,
	}
	innerFS.enospcs.Store(10)
	var callbackInvocations atomic.Int32
	fs := OnDiskFull(innerFS, func() {
		callbackInvocations.Add(1)
	})

	var wg sync.WaitGroup
	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			_, err := fs.Create("foo")
			// They all should succeed on retry.
			require.NoError(t, err)
		}()
	}
	wg.Wait()
	// Since all operations should start before the first one returns an
	// ENOSPC, the callback should only be invoked once.
	require.Equal(t, int32(1), callbackInvocations.Load())
	require.Equal(t, uint32(20), innerFS.invocations.Load())
}

type enospcMockFS struct {
	FS
	opDelay      time.Duration
	bytesWritten int
	enospcs      atomic.Int32
	invocations  atomic.Uint32
}

func (fs *enospcMockFS) maybeENOSPC() error {
	fs.invocations.Add(1)
	v := fs.enospcs.Add(-1)

	// Sleep before returning so that tests may issue concurrent writes that
	// fall into the same write generation.
	time.Sleep(fs.opDelay)

	if v >= 0 {
		// Wrap the error to test error unwrapping.
		err := &os.PathError{Op: "mock", Path: "mock", Err: syscall.ENOSPC}
		return errors.Wrap(err, "uh oh")
	}
	return nil
}

func (fs *enospcMockFS) Create(name string) (File, error) {
	if err := fs.maybeENOSPC(); err != nil {
		return nil, err
	}
	return &enospcMockFile{fs: fs}, nil
}

func (fs *enospcMockFS) Link(oldname, newname string) error {
	if err := fs.maybeENOSPC(); err != nil {
		return err
	}
	return nil
}

func (fs *enospcMockFS) Remove(name string) error {
	if err := fs.maybeENOSPC(); err != nil {
		return err
	}
	return nil
}

func (fs *enospcMockFS) RemoveAll(name string) error {
	if err := fs.maybeENOSPC(); err != nil {
		return err
	}
	return nil
}

func (fs *enospcMockFS) Rename(oldname, newname string) error {
	if err := fs.maybeENOSPC(); err != nil {
		return err
	}
	return nil
}

func (fs *enospcMockFS) ReuseForWrite(oldname, newname string) (File, error) {
	if err := fs.maybeENOSPC(); err != nil {
		return nil, err
	}
	return &enospcMockFile{fs: fs}, nil
}

func (fs *enospcMockFS) MkdirAll(dir string, perm os.FileMode) error {
	if err := fs.maybeENOSPC(); err != nil {
		return err
	}
	return nil
}

func (fs *enospcMockFS) Lock(name string) (io.Closer, error) {
	if err := fs.maybeENOSPC(); err != nil {
		return nil, err
	}
	return nil, nil
}

type enospcMockFile struct {
	fs *enospcMockFS
	File
}

func (f *enospcMockFile) Write(b []byte) (int, error) {

	if err := f.fs.maybeENOSPC(); err != nil {
		n := len(b)
		if f.fs.bytesWritten < n {
			n = f.fs.bytesWritten
		}
		return n, err
	}
	return len(b), nil
}

func (f *enospcMockFile) Sync() error {
	return f.fs.maybeENOSPC()
}

// BenchmarkOnDiskFull benchmarks the overhead of the OnDiskFull filesystem
// wrapper during a Write when there is no ENOSPC.
func BenchmarkOnDiskFull(b *testing.B) {
	fs := OnDiskFull(NewMem(), func() {})

	f, err := fs.Create("foo")
	require.NoError(b, err)
	defer func() { require.NoError(b, f.Close()) }()

	payload := []byte("hello world")
	for i := 0; i < b.N; i++ {
		_, err := f.Write(payload)
		require.NoError(b, err)
	}
}
