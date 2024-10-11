// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package errorfs

import (
	"fmt"
	"io"
	"math/rand"
	"os"
	"sync"
	"sync/atomic"
	"time"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/errors/oserror"
	"github.com/cockroachdb/pebble/vfs"
)

// ErrInjected is an error artificially injected for testing fs error paths.
var ErrInjected = errors.New("injected error")

// Op is an enum describing the type of operation.
type Op int

const (
	// OpCreate describes a create file operation.
	OpCreate Op = iota
	// OpLink describes a hardlink operation.
	OpLink
	// OpOpen describes a file open operation.
	OpOpen
	// OpOpenDir describes a directory open operation.
	OpOpenDir
	// OpRemove describes a remove file operation.
	OpRemove
	// OpRemoveAll describes a recursive remove operation.
	OpRemoveAll
	// OpRename describes a rename operation.
	OpRename
	// OpReuseForRewrite describes a reuse for rewriting operation.
	OpReuseForRewrite
	// OpMkdirAll describes a make directory including parents operation.
	OpMkdirAll
	// OpLock describes a lock file operation.
	OpLock
	// OpList describes a list directory operation.
	OpList
	// OpFilePreallocate describes a file preallocate operation.
	OpFilePreallocate
	// OpStat describes a path-based stat operation.
	OpStat
	// OpGetDiskUsage describes a disk usage operation.
	OpGetDiskUsage
	// OpFileClose describes a close file operation.
	OpFileClose
	// OpFileRead describes a file read operation.
	OpFileRead
	// OpFileReadAt describes a file seek read operation.
	OpFileReadAt
	// OpFileWrite describes a file write operation.
	OpFileWrite
	// OpFileWriteAt describes a file seek write operation.
	OpFileWriteAt
	// OpFileStat describes a file stat operation.
	OpFileStat
	// OpFileSync describes a file sync operation.
	OpFileSync
	// OpFileFlush describes a file flush operation.
	OpFileFlush
)

// OpKind returns the operation's kind.
func (o Op) OpKind() OpKind {
	switch o {
	case OpOpen, OpOpenDir, OpList, OpStat, OpGetDiskUsage, OpFileRead, OpFileReadAt, OpFileStat:
		return OpKindRead
	case OpCreate, OpLink, OpRemove, OpRemoveAll, OpRename, OpReuseForRewrite, OpMkdirAll, OpLock, OpFileClose, OpFileWrite, OpFileWriteAt, OpFileSync, OpFileFlush, OpFilePreallocate:
		return OpKindWrite
	default:
		panic(fmt.Sprintf("unrecognized op %v\n", o))
	}
}

// OpKind is an enum describing whether an operation is a read or write
// operation.
type OpKind int

const (
	// OpKindRead describes read operations.
	OpKindRead OpKind = iota
	// OpKindWrite describes write operations.
	OpKindWrite
)

// OnIndex constructs an injector that returns an error on
// the (n+1)-th invocation of its MaybeError function. It
// may be passed to Wrap to inject an error into an FS.
func OnIndex(index int32) *InjectIndex {
	ii := &InjectIndex{}
	ii.index.Store(index)
	return ii
}

// InjectIndex implements Injector, injecting an error at a specific index.
type InjectIndex struct {
	index atomic.Int32
}

// Index returns the index at which the error will be injected.
func (ii *InjectIndex) Index() int32 { return ii.index.Load() }

// SetIndex sets the index at which the error will be injected.
func (ii *InjectIndex) SetIndex(v int32) { ii.index.Store(v) }

// MaybeError implements the Injector interface.
func (ii *InjectIndex) MaybeError(_ Op, _ string) error {
	if ii.index.Add(-1) == -1 {
		return errors.WithStack(ErrInjected)
	}
	return nil
}

// WithProbability returns a function that returns an error with the provided
// probability when passed op. It may be passed to Wrap to inject an error
// into an ErrFS with the provided probability. p should be within the range
// [0.0,1.0].
func WithProbability(op OpKind, p float64) Injector {
	mu := new(sync.Mutex)
	rnd := rand.New(rand.NewSource(time.Now().UnixNano()))
	return InjectorFunc(func(currOp Op, _ string) error {
		mu.Lock()
		defer mu.Unlock()
		if currOp.OpKind() == op && rnd.Float64() < p {
			return errors.WithStack(ErrInjected)
		}
		return nil
	})
}

// InjectorFunc implements the Injector interface for a function with
// MaybeError's signature.
type InjectorFunc func(Op, string) error

// MaybeError implements the Injector interface.
func (f InjectorFunc) MaybeError(op Op, path string) error { return f(op, path) }

// Injector injects errors into FS operations.
type Injector interface {
	// MaybeError is invoked by an errorfs before an operation is executed. It
	// is passed an enum indicating the type of operation and a path of the
	// subject file or directory. If the operation takes two paths (eg,
	// Rename, Link), the original source path is provided.
	MaybeError(op Op, path string) error
}

// FS implements vfs.FS, injecting errors into
// its operations.
type FS struct {
	fs  vfs.FS
	inj Injector
}

// Wrap wraps an existing vfs.FS implementation, returning a new
// vfs.FS implementation that shadows operations to the provided FS.
// It uses the provided Injector for deciding when to inject errors.
// If an error is injected, FS propagates the error instead of
// shadowing the operation.
func Wrap(fs vfs.FS, inj Injector) *FS {
	return &FS{
		fs:  fs,
		inj: inj,
	}
}

// WrapFile wraps an existing vfs.File, returning a new vfs.File that shadows
// operations to the provided vfs.File. It uses the provided Injector for
// deciding when to inject errors. If an error is injected, the file
// propagates the error instead of shadowing the operation.
func WrapFile(f vfs.File, inj Injector) vfs.File {
	return &errorFile{file: f, inj: inj}
}

// Unwrap returns the FS implementation underlying fs.
// See pebble/vfs.Root.
func (fs *FS) Unwrap() vfs.FS {
	return fs.fs
}

// Create implements FS.Create.
func (fs *FS) Create(name string) (vfs.File, error) {
	if err := fs.inj.MaybeError(OpCreate, name); err != nil {
		return nil, err
	}
	f, err := fs.fs.Create(name)
	if err != nil {
		return nil, err
	}
	return &errorFile{name, f, fs.inj}, nil
}

// Link implements FS.Link.
func (fs *FS) Link(oldname, newname string) error {
	if err := fs.inj.MaybeError(OpLink, oldname); err != nil {
		return err
	}
	return fs.fs.Link(oldname, newname)
}

// Open implements FS.Open.
func (fs *FS) Open(name string, opts ...vfs.OpenOption) (vfs.File, error) {
	if err := fs.inj.MaybeError(OpOpen, name); err != nil {
		return nil, err
	}
	f, err := fs.fs.Open(name)
	if err != nil {
		return nil, err
	}
	ef := &errorFile{name, f, fs.inj}
	for _, opt := range opts {
		opt.Apply(ef)
	}
	return ef, nil
}

// OpenReadWrite implements FS.OpenReadWrite.
func (fs *FS) OpenReadWrite(name string, opts ...vfs.OpenOption) (vfs.File, error) {
	if err := fs.inj.MaybeError(OpOpen, name); err != nil {
		return nil, err
	}
	f, err := fs.fs.OpenReadWrite(name)
	if err != nil {
		return nil, err
	}
	ef := &errorFile{name, f, fs.inj}
	for _, opt := range opts {
		opt.Apply(ef)
	}
	return ef, nil
}

// OpenDir implements FS.OpenDir.
func (fs *FS) OpenDir(name string) (vfs.File, error) {
	if err := fs.inj.MaybeError(OpOpenDir, name); err != nil {
		return nil, err
	}
	f, err := fs.fs.OpenDir(name)
	if err != nil {
		return nil, err
	}
	return &errorFile{name, f, fs.inj}, nil
}

// GetDiskUsage implements FS.GetDiskUsage.
func (fs *FS) GetDiskUsage(path string) (vfs.DiskUsage, error) {
	if err := fs.inj.MaybeError(OpGetDiskUsage, path); err != nil {
		return vfs.DiskUsage{}, err
	}
	return fs.fs.GetDiskUsage(path)
}

// PathBase implements FS.PathBase.
func (fs *FS) PathBase(p string) string {
	return fs.fs.PathBase(p)
}

// PathDir implements FS.PathDir.
func (fs *FS) PathDir(p string) string {
	return fs.fs.PathDir(p)
}

// PathJoin implements FS.PathJoin.
func (fs *FS) PathJoin(elem ...string) string {
	return fs.fs.PathJoin(elem...)
}

// Remove implements FS.Remove.
func (fs *FS) Remove(name string) error {
	if _, err := fs.fs.Stat(name); oserror.IsNotExist(err) {
		return nil
	}

	if err := fs.inj.MaybeError(OpRemove, name); err != nil {
		return err
	}
	return fs.fs.Remove(name)
}

// RemoveAll implements FS.RemoveAll.
func (fs *FS) RemoveAll(fullname string) error {
	if err := fs.inj.MaybeError(OpRemoveAll, fullname); err != nil {
		return err
	}
	return fs.fs.RemoveAll(fullname)
}

// Rename implements FS.Rename.
func (fs *FS) Rename(oldname, newname string) error {
	if err := fs.inj.MaybeError(OpRename, oldname); err != nil {
		return err
	}
	return fs.fs.Rename(oldname, newname)
}

// ReuseForWrite implements FS.ReuseForWrite.
func (fs *FS) ReuseForWrite(oldname, newname string) (vfs.File, error) {
	if err := fs.inj.MaybeError(OpReuseForRewrite, oldname); err != nil {
		return nil, err
	}
	return fs.fs.ReuseForWrite(oldname, newname)
}

// MkdirAll implements FS.MkdirAll.
func (fs *FS) MkdirAll(dir string, perm os.FileMode) error {
	if err := fs.inj.MaybeError(OpMkdirAll, dir); err != nil {
		return err
	}
	return fs.fs.MkdirAll(dir, perm)
}

// Lock implements FS.Lock.
func (fs *FS) Lock(name string) (io.Closer, error) {
	if err := fs.inj.MaybeError(OpLock, name); err != nil {
		return nil, err
	}
	return fs.fs.Lock(name)
}

// List implements FS.List.
func (fs *FS) List(dir string) ([]string, error) {
	if err := fs.inj.MaybeError(OpList, dir); err != nil {
		return nil, err
	}
	return fs.fs.List(dir)
}

// Stat implements FS.Stat.
func (fs *FS) Stat(name string) (os.FileInfo, error) {
	if err := fs.inj.MaybeError(OpStat, name); err != nil {
		return nil, err
	}
	return fs.fs.Stat(name)
}

// errorFile implements vfs.File. The interface is implemented on the pointer
// type to allow pointer equality comparisons.
type errorFile struct {
	path string
	file vfs.File
	inj  Injector
}

func (f *errorFile) Close() error {
	// We don't inject errors during close as those calls should never fail in
	// practice.
	return f.file.Close()
}

func (f *errorFile) Read(p []byte) (int, error) {
	if err := f.inj.MaybeError(OpFileRead, f.path); err != nil {
		return 0, err
	}
	return f.file.Read(p)
}

func (f *errorFile) ReadAt(p []byte, off int64) (int, error) {
	if err := f.inj.MaybeError(OpFileReadAt, f.path); err != nil {
		return 0, err
	}
	return f.file.ReadAt(p, off)
}

func (f *errorFile) Write(p []byte) (int, error) {
	if err := f.inj.MaybeError(OpFileWrite, f.path); err != nil {
		return 0, err
	}
	return f.file.Write(p)
}

func (f *errorFile) WriteAt(p []byte, ofs int64) (int, error) {
	if err := f.inj.MaybeError(OpFileWriteAt, f.path); err != nil {
		return 0, err
	}
	return f.file.WriteAt(p, ofs)
}

func (f *errorFile) Stat() (os.FileInfo, error) {
	if err := f.inj.MaybeError(OpFileStat, f.path); err != nil {
		return nil, err
	}
	return f.file.Stat()
}

func (f *errorFile) Prefetch(offset, length int64) error {
	// TODO(radu): Consider error injection.
	return f.file.Prefetch(offset, length)
}

func (f *errorFile) Preallocate(offset, length int64) error {
	if err := f.inj.MaybeError(OpFilePreallocate, f.path); err != nil {
		return err
	}
	return f.file.Preallocate(offset, length)
}

func (f *errorFile) Sync() error {
	if err := f.inj.MaybeError(OpFileSync, f.path); err != nil {
		return err
	}
	return f.file.Sync()
}

func (f *errorFile) SyncData() error {
	// TODO(jackson): Consider error injection.
	return f.file.SyncData()
}

func (f *errorFile) SyncTo(length int64) (fullSync bool, err error) {
	// TODO(jackson): Consider error injection.
	return f.file.SyncTo(length)
}

func (f *errorFile) Fd() uintptr {
	return f.file.Fd()
}
