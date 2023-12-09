// Copyright 2012 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package vfs

import (
	"io"
	"os"
	"path/filepath"
	"syscall"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/errors/oserror"
)

// File is a readable, writable sequence of bytes.
//
// Typically, it will be an *os.File, but test code may choose to substitute
// memory-backed implementations.
//
// Write-oriented operations (Write, Sync) must be called sequentially: At most
// 1 call to Write or Sync may be executed at any given time.
type File interface {
	io.Closer
	io.Reader
	io.ReaderAt
	// Unlike the specification for io.Writer.Write(), the vfs.File.Write()
	// method *is* allowed to modify the slice passed in, whether temporarily
	// or permanently. Callers of Write() need to take this into account.
	io.Writer
	// WriteAt() is only supported for files that were opened with FS.OpenReadWrite.
	io.WriterAt

	// Preallocate optionally preallocates storage for `length` at `offset`
	// within the file. Implementations may choose to do nothing.
	Preallocate(offset, length int64) error
	Stat() (os.FileInfo, error)
	Sync() error

	// SyncTo requests that a prefix of the file's data be synced to stable
	// storage. The caller passes provides a `length`, indicating how many bytes
	// to sync from the beginning of the file. SyncTo is a no-op for
	// directories, and therefore always returns false.
	//
	// SyncTo returns a fullSync return value, indicating one of two possible
	// outcomes.
	//
	// If fullSync is false, the first `length` bytes of the file was queued to
	// be synced to stable storage. The syncing of the file prefix may happen
	// asynchronously. No persistence guarantee is provided.
	//
	// If fullSync is true, the entirety of the file's contents were
	// synchronously synced to stable storage, and a persistence guarantee is
	// provided. In this outcome, any modified metadata for the file is not
	// guaranteed to be synced unless that metadata is needed in order to allow
	// a subsequent data retrieval to be correctly handled.
	SyncTo(length int64) (fullSync bool, err error)

	// SyncData requires that all written data be persisted. File metadata is
	// not required to be synced. Unsophisticated implementations may call Sync.
	SyncData() error

	// Prefetch signals the OS (on supported platforms) to fetch the next length
	// bytes in file (as returned by os.File.Fd()) after offset into cache. Any
	// subsequent reads in that range will not issue disk IO.
	Prefetch(offset int64, length int64) error

	// Fd returns the raw file descriptor when a File is backed by an *os.File.
	// It can be used for specific functionality like Prefetch.
	// Returns InvalidFd if not supported.
	Fd() uintptr
}

// InvalidFd is a special value returned by File.Fd() when the file is not
// backed by an OS descriptor.
// Note: the special value is consistent with what os.File implementation
// returns on a nil receiver.
const InvalidFd uintptr = ^(uintptr(0))

// OpenOption provide an interface to do work on file handles in the Open()
// call.
type OpenOption interface {
	// Apply is called on the file handle after it's opened.
	Apply(File)
}

// FS is a namespace for files.
//
// The names are filepath names: they may be / separated or \ separated,
// depending on the underlying operating system.
type FS interface {
	// Create creates the named file for reading and writing. If a file
	// already exists at the provided name, it's removed first ensuring the
	// resulting file descriptor points to a new inode.
	Create(name string) (File, error)

	// Link creates newname as a hard link to the oldname file.
	Link(oldname, newname string) error

	// Open opens the named file for reading. openOptions provides
	Open(name string, opts ...OpenOption) (File, error)

	// OpenReadWrite opens the named file for reading and writing. If the file
	// does not exist, it is created.
	OpenReadWrite(name string, opts ...OpenOption) (File, error)

	// OpenDir opens the named directory for syncing.
	OpenDir(name string) (File, error)

	// Remove removes the named file or directory.
	Remove(name string) error

	// Remove removes the named file or directory and any children it
	// contains. It removes everything it can but returns the first error it
	// encounters.
	RemoveAll(name string) error

	// Rename renames a file. It overwrites the file at newname if one exists,
	// the same as os.Rename.
	Rename(oldname, newname string) error

	// ReuseForWrite attempts to reuse the file with oldname by renaming it to newname and opening
	// it for writing without truncation. It is acceptable for the implementation to choose not
	// to reuse oldname, and simply create the file with newname -- in this case the implementation
	// should delete oldname. If the caller calls this function with an oldname that does not exist,
	// the implementation may return an error.
	ReuseForWrite(oldname, newname string) (File, error)

	// MkdirAll creates a directory and all necessary parents. The permission
	// bits perm have the same semantics as in os.MkdirAll. If the directory
	// already exists, MkdirAll does nothing and returns nil.
	MkdirAll(dir string, perm os.FileMode) error

	// Lock locks the given file, creating the file if necessary, and
	// truncating the file if it already exists. The lock is an exclusive lock
	// (a write lock), but locked files should neither be read from nor written
	// to. Such files should have zero size and only exist to co-ordinate
	// ownership across processes.
	//
	// A nil Closer is returned if an error occurred. Otherwise, close that
	// Closer to release the lock.
	//
	// On Linux and OSX, a lock has the same semantics as fcntl(2)'s advisory
	// locks. In particular, closing any other file descriptor for the same
	// file will release the lock prematurely.
	//
	// Attempting to lock a file that is already locked by the current process
	// returns an error and leaves the existing lock untouched.
	//
	// Lock is not yet implemented on other operating systems, and calling it
	// will return an error.
	Lock(name string) (io.Closer, error)

	// List returns a listing of the given directory. The names returned are
	// relative to dir.
	List(dir string) ([]string, error)

	// Stat returns an os.FileInfo describing the named file.
	Stat(name string) (os.FileInfo, error)

	// PathBase returns the last element of path. Trailing path separators are
	// removed before extracting the last element. If the path is empty, PathBase
	// returns ".".  If the path consists entirely of separators, PathBase returns a
	// single separator.
	PathBase(path string) string

	// PathJoin joins any number of path elements into a single path, adding a
	// separator if necessary.
	PathJoin(elem ...string) string

	// PathDir returns all but the last element of path, typically the path's directory.
	PathDir(path string) string

	// GetDiskUsage returns disk space statistics for the filesystem where
	// path is any file or directory within that filesystem.
	GetDiskUsage(path string) (DiskUsage, error)
}

// DiskUsage summarizes disk space usage on a filesystem.
type DiskUsage struct {
	// Total disk space available to the current process in bytes.
	AvailBytes uint64
	// Total disk space in bytes.
	TotalBytes uint64
	// Used disk space in bytes.
	UsedBytes uint64
}

// Default is a FS implementation backed by the underlying operating system's
// file system.
var Default FS = defaultFS{}

type defaultFS struct{}

// wrapOSFile takes a standard library OS file and returns a vfs.File. f may be
// nil, in which case wrapOSFile must not panic. In such cases, it's okay if the
// returned vfs.File may panic if used.
func wrapOSFile(f *os.File) File {
	// See the implementations in default_{linux,unix,windows}.go.
	return wrapOSFileImpl(f)
}

func (defaultFS) Create(name string) (File, error) {
	const openFlags = os.O_RDWR | os.O_CREATE | os.O_EXCL | syscall.O_CLOEXEC

	osFile, err := os.OpenFile(name, openFlags, 0666)
	// If the file already exists, remove it and try again.
	//
	// NB: We choose to remove the file instead of truncating it, despite the
	// fact that we can't do so atomically, because it's more resistant to
	// misuse when using hard links.

	// We must loop in case another goroutine/thread/process is also
	// attempting to create the a file at the same path.
	for oserror.IsExist(err) {
		if removeErr := os.Remove(name); removeErr != nil && !oserror.IsNotExist(removeErr) {
			return wrapOSFile(osFile), errors.WithStack(removeErr)
		}
		osFile, err = os.OpenFile(name, openFlags, 0666)
	}
	return wrapOSFile(osFile), errors.WithStack(err)
}

func (defaultFS) Link(oldname, newname string) error {
	return errors.WithStack(os.Link(oldname, newname))
}

func (defaultFS) Open(name string, opts ...OpenOption) (File, error) {
	osFile, err := os.OpenFile(name, os.O_RDONLY|syscall.O_CLOEXEC, 0)
	if err != nil {
		return nil, errors.WithStack(err)
	}
	file := wrapOSFile(osFile)
	for _, opt := range opts {
		opt.Apply(file)
	}
	return file, nil
}

func (defaultFS) OpenReadWrite(name string, opts ...OpenOption) (File, error) {
	osFile, err := os.OpenFile(name, os.O_RDWR|syscall.O_CLOEXEC|os.O_CREATE, 0666)
	if err != nil {
		return nil, errors.WithStack(err)
	}
	file := wrapOSFile(osFile)
	for _, opt := range opts {
		opt.Apply(file)
	}
	return file, nil
}

func (defaultFS) Remove(name string) error {
	return errors.WithStack(os.Remove(name))
}

func (defaultFS) RemoveAll(name string) error {
	return errors.WithStack(os.RemoveAll(name))
}

func (defaultFS) Rename(oldname, newname string) error {
	return errors.WithStack(os.Rename(oldname, newname))
}

func (fs defaultFS) ReuseForWrite(oldname, newname string) (File, error) {
	if err := fs.Rename(oldname, newname); err != nil {
		return nil, errors.WithStack(err)
	}
	f, err := os.OpenFile(newname, os.O_RDWR|os.O_CREATE|syscall.O_CLOEXEC, 0666)
	return wrapOSFile(f), errors.WithStack(err)
}

func (defaultFS) MkdirAll(dir string, perm os.FileMode) error {
	return errors.WithStack(os.MkdirAll(dir, perm))
}

func (defaultFS) List(dir string) ([]string, error) {
	f, err := os.Open(dir)
	if err != nil {
		return nil, err
	}
	defer f.Close()
	dirnames, err := f.Readdirnames(-1)
	return dirnames, errors.WithStack(err)
}

func (defaultFS) Stat(name string) (os.FileInfo, error) {
	finfo, err := os.Stat(name)
	return finfo, errors.WithStack(err)
}

func (defaultFS) PathBase(path string) string {
	return filepath.Base(path)
}

func (defaultFS) PathJoin(elem ...string) string {
	return filepath.Join(elem...)
}

func (defaultFS) PathDir(path string) string {
	return filepath.Dir(path)
}

type randomReadsOption struct{}

// RandomReadsOption is an OpenOption that optimizes opened file handle for
// random reads, by calling  fadvise() with POSIX_FADV_RANDOM on Linux systems
// to disable readahead.
var RandomReadsOption OpenOption = &randomReadsOption{}

// Apply implements the OpenOption interface.
func (randomReadsOption) Apply(f File) {
	if fd := f.Fd(); fd != InvalidFd {
		_ = fadviseRandom(fd)
	}
}

type sequentialReadsOption struct{}

// SequentialReadsOption is an OpenOption that optimizes opened file handle for
// sequential reads, by calling fadvise() with POSIX_FADV_SEQUENTIAL on Linux
// systems to enable readahead.
var SequentialReadsOption OpenOption = &sequentialReadsOption{}

// Apply implements the OpenOption interface.
func (sequentialReadsOption) Apply(f File) {
	if fd := f.Fd(); fd != InvalidFd {
		_ = fadviseSequential(fd)
	}
}

// Copy copies the contents of oldname to newname. If newname exists, it will
// be overwritten.
func Copy(fs FS, oldname, newname string) error {
	return CopyAcrossFS(fs, oldname, fs, newname)
}

// CopyAcrossFS copies the contents of oldname on srcFS to newname dstFS. If
// newname exists, it will be overwritten.
func CopyAcrossFS(srcFS FS, oldname string, dstFS FS, newname string) error {
	src, err := srcFS.Open(oldname, SequentialReadsOption)
	if err != nil {
		return err
	}
	defer src.Close()

	dst, err := dstFS.Create(newname)
	if err != nil {
		return err
	}
	defer dst.Close()

	if _, err := io.Copy(dst, src); err != nil {
		return err
	}
	return dst.Sync()
}

// LimitedCopy copies up to maxBytes from oldname to newname. If newname
// exists, it will be overwritten.
func LimitedCopy(fs FS, oldname, newname string, maxBytes int64) error {
	src, err := fs.Open(oldname, SequentialReadsOption)
	if err != nil {
		return err
	}
	defer src.Close()

	dst, err := fs.Create(newname)
	if err != nil {
		return err
	}
	defer dst.Close()

	if _, err := io.Copy(dst, &io.LimitedReader{R: src, N: maxBytes}); err != nil {
		return err
	}
	return dst.Sync()
}

// LinkOrCopy creates newname as a hard link to the oldname file. If creating
// the hard link fails, LinkOrCopy falls back to copying the file (which may
// also fail if oldname doesn't exist or newname already exists).
func LinkOrCopy(fs FS, oldname, newname string) error {
	err := fs.Link(oldname, newname)
	if err == nil {
		return nil
	}
	// Permit a handful of errors which we know won't be fixed by copying the
	// file. Note that we don't check for the specifics of the error code as it
	// isn't easy to do so in a portable manner. On Unix we'd have to check for
	// LinkError.Err == syscall.EXDEV. On Windows we'd have to check for
	// ERROR_NOT_SAME_DEVICE, ERROR_INVALID_FUNCTION, and
	// ERROR_INVALID_PARAMETER. Rather that such OS specific checks, we fall back
	// to always trying to copy if hard-linking failed.
	if oserror.IsExist(err) || oserror.IsNotExist(err) || oserror.IsPermission(err) {
		return err
	}
	return Copy(fs, oldname, newname)
}

// Root returns the base FS implementation, unwrapping all nested FSs that
// expose an Unwrap method.
func Root(fs FS) FS {
	type unwrapper interface {
		Unwrap() FS
	}

	for {
		u, ok := fs.(unwrapper)
		if !ok {
			break
		}
		fs = u.Unwrap()
	}
	return fs
}

// ErrUnsupported may be returned a FS when it does not support an operation.
var ErrUnsupported = errors.New("pebble: not supported")
