// Copyright 2012 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package vfs // import "github.com/cockroachdb/pebble/vfs"

import (
	"bytes"
	"fmt"
	"io"
	"os"
	"path"
	"sort"
	"strings"
	"sync"
	"sync/atomic"
	"syscall"
	"time"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/errors/oserror"
	"github.com/cockroachdb/pebble/internal/invariants"
)

const sep = "/"

// NewMem returns a new memory-backed FS implementation.
func NewMem() *MemFS {
	return &MemFS{
		root: newRootMemNode(),
	}
}

// NewStrictMem returns a "strict" memory-backed FS implementation. The behaviour is strict wrt
// needing a Sync() call on files or directories for the state changes to be finalized. Any
// changes that are not finalized are visible to reads until MemFS.ResetToSyncedState() is called,
// at which point they are discarded and no longer visible.
//
// Expected usage:
//
//	strictFS := NewStrictMem()
//	db := Open(..., &Options{FS: strictFS})
//	// Do and commit various operations.
//	...
//	// Prevent any more changes to finalized state.
//	strictFS.SetIgnoreSyncs(true)
//	// This will finish any ongoing background flushes, compactions but none of these writes will
//	// be finalized since syncs are being ignored.
//	db.Close()
//	// Discard unsynced state.
//	strictFS.ResetToSyncedState()
//	// Allow changes to finalized state.
//	strictFS.SetIgnoreSyncs(false)
//	// Open the DB. This DB should have the same state as if the earlier strictFS operations and
//	// db.Close() were not called.
//	db := Open(..., &Options{FS: strictFS})
func NewStrictMem() *MemFS {
	return &MemFS{
		root:   newRootMemNode(),
		strict: true,
	}
}

// NewMemFile returns a memory-backed File implementation. The memory-backed
// file takes ownership of data.
func NewMemFile(data []byte) File {
	n := &memNode{}
	n.refs.Store(1)
	n.mu.data = data
	n.mu.modTime = time.Now()
	return &memFile{
		n:    n,
		read: true,
	}
}

// MemFS implements FS.
type MemFS struct {
	mu   sync.Mutex
	root *memNode

	// lockFiles holds a map of open file locks. Presence in this map indicates
	// a file lock is currently held. Keys are strings holding the path of the
	// locked file. The stored value is untyped and  unused; only presence of
	// the key within the map is significant.
	lockedFiles sync.Map
	strict      bool
	ignoreSyncs bool
	// Windows has peculiar semantics with respect to hard links and deleting
	// open files. In tests meant to exercise this behavior, this flag can be
	// set to error if removing an open file.
	windowsSemantics bool
}

var _ FS = &MemFS{}

// UseWindowsSemantics configures whether the MemFS implements Windows-style
// semantics, in particular with respect to whether any of an open file's links
// may be removed. Windows semantics default to off.
func (y *MemFS) UseWindowsSemantics(windowsSemantics bool) {
	y.mu.Lock()
	defer y.mu.Unlock()
	y.windowsSemantics = windowsSemantics
}

// String dumps the contents of the MemFS.
func (y *MemFS) String() string {
	y.mu.Lock()
	defer y.mu.Unlock()

	s := new(bytes.Buffer)
	y.root.dump(s, 0)
	return s.String()
}

// SetIgnoreSyncs sets the MemFS.ignoreSyncs field. See the usage comment with NewStrictMem() for
// details.
func (y *MemFS) SetIgnoreSyncs(ignoreSyncs bool) {
	y.mu.Lock()
	if !y.strict {
		// noop
		return
	}
	y.ignoreSyncs = ignoreSyncs
	y.mu.Unlock()
}

// ResetToSyncedState discards state in the FS that is not synced. See the usage comment with
// NewStrictMem() for details.
func (y *MemFS) ResetToSyncedState() {
	if !y.strict {
		// noop
		return
	}
	y.mu.Lock()
	y.root.resetToSyncedState()
	y.mu.Unlock()
}

// walk walks the directory tree for the fullname, calling f at each step. If
// f returns an error, the walk will be aborted and return that same error.
//
// Each walk is atomic: y's mutex is held for the entire operation, including
// all calls to f.
//
// dir is the directory at that step, frag is the name fragment, and final is
// whether it is the final step. For example, walking "/foo/bar/x" will result
// in 3 calls to f:
//   - "/", "foo", false
//   - "/foo/", "bar", false
//   - "/foo/bar/", "x", true
//
// Similarly, walking "/y/z/", with a trailing slash, will result in 3 calls to f:
//   - "/", "y", false
//   - "/y/", "z", false
//   - "/y/z/", "", true
func (y *MemFS) walk(fullname string, f func(dir *memNode, frag string, final bool) error) error {
	y.mu.Lock()
	defer y.mu.Unlock()

	// For memfs, the current working directory is the same as the root directory,
	// so we strip off any leading "/"s to make fullname a relative path, and
	// the walk starts at y.root.
	for len(fullname) > 0 && fullname[0] == sep[0] {
		fullname = fullname[1:]
	}
	dir := y.root

	for {
		frag, remaining := fullname, ""
		i := strings.IndexRune(fullname, rune(sep[0]))
		final := i < 0
		if !final {
			frag, remaining = fullname[:i], fullname[i+1:]
			for len(remaining) > 0 && remaining[0] == sep[0] {
				remaining = remaining[1:]
			}
		}
		if err := f(dir, frag, final); err != nil {
			return err
		}
		if final {
			break
		}
		child := dir.children[frag]
		if child == nil {
			return &os.PathError{
				Op:   "open",
				Path: fullname,
				Err:  oserror.ErrNotExist,
			}
		}
		if !child.isDir {
			return &os.PathError{
				Op:   "open",
				Path: fullname,
				Err:  errors.New("not a directory"),
			}
		}
		dir, fullname = child, remaining
	}
	return nil
}

// Create implements FS.Create.
func (y *MemFS) Create(fullname string) (File, error) {
	var ret *memFile
	err := y.walk(fullname, func(dir *memNode, frag string, final bool) error {
		if final {
			if frag == "" {
				return errors.New("pebble/vfs: empty file name")
			}
			n := &memNode{name: frag}
			dir.children[frag] = n
			ret = &memFile{
				n:     n,
				fs:    y,
				read:  true,
				write: true,
			}
		}
		return nil
	})
	if err != nil {
		return nil, err
	}
	ret.n.refs.Add(1)
	return ret, nil
}

// Link implements FS.Link.
func (y *MemFS) Link(oldname, newname string) error {
	var n *memNode
	err := y.walk(oldname, func(dir *memNode, frag string, final bool) error {
		if final {
			if frag == "" {
				return errors.New("pebble/vfs: empty file name")
			}
			n = dir.children[frag]
		}
		return nil
	})
	if err != nil {
		return err
	}
	if n == nil {
		return &os.LinkError{
			Op:  "link",
			Old: oldname,
			New: newname,
			Err: oserror.ErrNotExist,
		}
	}
	return y.walk(newname, func(dir *memNode, frag string, final bool) error {
		if final {
			if frag == "" {
				return errors.New("pebble/vfs: empty file name")
			}
			if _, ok := dir.children[frag]; ok {
				return &os.LinkError{
					Op:  "link",
					Old: oldname,
					New: newname,
					Err: oserror.ErrExist,
				}
			}
			dir.children[frag] = n
		}
		return nil
	})
}

func (y *MemFS) open(fullname string, openForWrite bool) (File, error) {
	var ret *memFile
	err := y.walk(fullname, func(dir *memNode, frag string, final bool) error {
		if final {
			if frag == "" {
				ret = &memFile{
					n:  dir,
					fs: y,
				}
				return nil
			}
			if n := dir.children[frag]; n != nil {
				ret = &memFile{
					n:     n,
					fs:    y,
					read:  true,
					write: openForWrite,
				}
			}
		}
		return nil
	})
	if err != nil {
		return nil, err
	}
	if ret == nil {
		return nil, &os.PathError{
			Op:   "open",
			Path: fullname,
			Err:  oserror.ErrNotExist,
		}
	}
	ret.n.refs.Add(1)
	return ret, nil
}

// Open implements FS.Open.
func (y *MemFS) Open(fullname string, opts ...OpenOption) (File, error) {
	return y.open(fullname, false /* openForWrite */)
}

// OpenReadWrite implements FS.OpenReadWrite.
func (y *MemFS) OpenReadWrite(fullname string, opts ...OpenOption) (File, error) {
	f, err := y.open(fullname, true /* openForWrite */)
	pathErr, ok := err.(*os.PathError)
	if ok && pathErr.Err == oserror.ErrNotExist {
		return y.Create(fullname)
	}
	return f, err
}

// OpenDir implements FS.OpenDir.
func (y *MemFS) OpenDir(fullname string) (File, error) {
	return y.open(fullname, false /* openForWrite */)
}

// Remove implements FS.Remove.
func (y *MemFS) Remove(fullname string) error {
	return y.walk(fullname, func(dir *memNode, frag string, final bool) error {
		if final {
			if frag == "" {
				return errors.New("pebble/vfs: empty file name")
			}
			child, ok := dir.children[frag]
			if !ok {
				return oserror.ErrNotExist
			}
			if y.windowsSemantics {
				// Disallow removal of open files/directories which implements
				// Windows semantics. This ensures that we don't regress in the
				// ordering of operations and try to remove a file while it is
				// still open.
				if n := child.refs.Load(); n > 0 {
					return oserror.ErrInvalid
				}
			}
			if len(child.children) > 0 {
				return errNotEmpty
			}
			delete(dir.children, frag)
		}
		return nil
	})
}

// RemoveAll implements FS.RemoveAll.
func (y *MemFS) RemoveAll(fullname string) error {
	err := y.walk(fullname, func(dir *memNode, frag string, final bool) error {
		if final {
			if frag == "" {
				return errors.New("pebble/vfs: empty file name")
			}
			_, ok := dir.children[frag]
			if !ok {
				return nil
			}
			delete(dir.children, frag)
		}
		return nil
	})
	// Match os.RemoveAll which returns a nil error even if the parent
	// directories don't exist.
	if oserror.IsNotExist(err) {
		err = nil
	}
	return err
}

// Rename implements FS.Rename.
func (y *MemFS) Rename(oldname, newname string) error {
	var n *memNode
	err := y.walk(oldname, func(dir *memNode, frag string, final bool) error {
		if final {
			if frag == "" {
				return errors.New("pebble/vfs: empty file name")
			}
			n = dir.children[frag]
			delete(dir.children, frag)
		}
		return nil
	})
	if err != nil {
		return err
	}
	if n == nil {
		return &os.PathError{
			Op:   "open",
			Path: oldname,
			Err:  oserror.ErrNotExist,
		}
	}
	return y.walk(newname, func(dir *memNode, frag string, final bool) error {
		if final {
			if frag == "" {
				return errors.New("pebble/vfs: empty file name")
			}
			dir.children[frag] = n
			n.name = frag
		}
		return nil
	})
}

// ReuseForWrite implements FS.ReuseForWrite.
func (y *MemFS) ReuseForWrite(oldname, newname string) (File, error) {
	if err := y.Rename(oldname, newname); err != nil {
		return nil, err
	}
	f, err := y.Open(newname)
	if err != nil {
		return nil, err
	}
	y.mu.Lock()
	defer y.mu.Unlock()

	mf := f.(*memFile)
	mf.read = false
	mf.write = true
	return f, nil
}

// MkdirAll implements FS.MkdirAll.
func (y *MemFS) MkdirAll(dirname string, perm os.FileMode) error {
	return y.walk(dirname, func(dir *memNode, frag string, final bool) error {
		if frag == "" {
			if final {
				return nil
			}
			return errors.New("pebble/vfs: empty file name")
		}
		child := dir.children[frag]
		if child == nil {
			dir.children[frag] = &memNode{
				name:     frag,
				children: make(map[string]*memNode),
				isDir:    true,
			}
			return nil
		}
		if !child.isDir {
			return &os.PathError{
				Op:   "open",
				Path: dirname,
				Err:  errors.New("not a directory"),
			}
		}
		return nil
	})
}

// Lock implements FS.Lock.
func (y *MemFS) Lock(fullname string) (io.Closer, error) {
	// FS.Lock excludes other processes, but other processes cannot see this
	// process' memory. However some uses (eg, Cockroach tests) may open and
	// close the same MemFS-backed database multiple times. We want mutual
	// exclusion in this case too. See cockroachdb/cockroach#110645.
	_, loaded := y.lockedFiles.Swap(fullname, nil /* the value itself is insignificant */)
	if loaded {
		// This file lock has already been acquired. On unix, this results in
		// either EACCES or EAGAIN so we mimic.
		return nil, syscall.EAGAIN
	}
	// Otherwise, we successfully acquired the lock. Locks are visible in the
	// parent directory listing, and they also must be created under an existent
	// directory. Create the path so that we have the normal detection of
	// non-existent directory paths, and make the lock visible when listing
	// directory entries.
	f, err := y.Create(fullname)
	if err != nil {
		// "Release" the lock since we failed.
		y.lockedFiles.Delete(fullname)
		return nil, err
	}
	return &memFileLock{
		y:        y,
		f:        f,
		fullname: fullname,
	}, nil
}

// List implements FS.List.
func (y *MemFS) List(dirname string) ([]string, error) {
	if !strings.HasSuffix(dirname, sep) {
		dirname += sep
	}
	var ret []string
	err := y.walk(dirname, func(dir *memNode, frag string, final bool) error {
		if final {
			if frag != "" {
				panic("unreachable")
			}
			ret = make([]string, 0, len(dir.children))
			for s := range dir.children {
				ret = append(ret, s)
			}
		}
		return nil
	})
	return ret, err
}

// Stat implements FS.Stat.
func (y *MemFS) Stat(name string) (os.FileInfo, error) {
	f, err := y.Open(name)
	if err != nil {
		if pe, ok := err.(*os.PathError); ok {
			pe.Op = "stat"
		}
		return nil, err
	}
	defer f.Close()
	return f.Stat()
}

// PathBase implements FS.PathBase.
func (*MemFS) PathBase(p string) string {
	// Note that MemFS uses forward slashes for its separator, hence the use of
	// path.Base, not filepath.Base.
	return path.Base(p)
}

// PathJoin implements FS.PathJoin.
func (*MemFS) PathJoin(elem ...string) string {
	// Note that MemFS uses forward slashes for its separator, hence the use of
	// path.Join, not filepath.Join.
	return path.Join(elem...)
}

// PathDir implements FS.PathDir.
func (*MemFS) PathDir(p string) string {
	// Note that MemFS uses forward slashes for its separator, hence the use of
	// path.Dir, not filepath.Dir.
	return path.Dir(p)
}

// GetDiskUsage implements FS.GetDiskUsage.
func (*MemFS) GetDiskUsage(string) (DiskUsage, error) {
	return DiskUsage{}, ErrUnsupported
}

// memNode holds a file's data or a directory's children, and implements os.FileInfo.
type memNode struct {
	name  string
	isDir bool
	refs  atomic.Int32

	// Mutable state.
	// - For a file: data, syncedDate, modTime: A file is only being mutated by a single goroutine,
	//   but there can be concurrent readers e.g. DB.Checkpoint() which can read WAL or MANIFEST
	//   files that are being written to. Additionally Sync() calls can be concurrent with writing.
	// - For a directory: children and syncedChildren. Concurrent writes are possible, and
	//   these are protected using MemFS.mu.
	mu struct {
		sync.Mutex
		data       []byte
		syncedData []byte
		modTime    time.Time
	}

	children       map[string]*memNode
	syncedChildren map[string]*memNode
}

func newRootMemNode() *memNode {
	return &memNode{
		name:     "/", // set the name to match what file systems do
		children: make(map[string]*memNode),
		isDir:    true,
	}
}

func (f *memNode) IsDir() bool {
	return f.isDir
}

func (f *memNode) ModTime() time.Time {
	f.mu.Lock()
	defer f.mu.Unlock()
	return f.mu.modTime
}

func (f *memNode) Mode() os.FileMode {
	if f.isDir {
		return os.ModeDir | 0755
	}
	return 0755
}

func (f *memNode) Name() string {
	return f.name
}

func (f *memNode) Size() int64 {
	f.mu.Lock()
	defer f.mu.Unlock()
	return int64(len(f.mu.data))
}

func (f *memNode) Sys() interface{} {
	return nil
}

func (f *memNode) dump(w *bytes.Buffer, level int) {
	if f.isDir {
		w.WriteString("          ")
	} else {
		f.mu.Lock()
		fmt.Fprintf(w, "%8d  ", len(f.mu.data))
		f.mu.Unlock()
	}
	for i := 0; i < level; i++ {
		w.WriteString("  ")
	}
	w.WriteString(f.name)
	if !f.isDir {
		w.WriteByte('\n')
		return
	}
	if level > 0 { // deal with the fact that the root's name is already "/"
		w.WriteByte(sep[0])
	}
	w.WriteByte('\n')
	names := make([]string, 0, len(f.children))
	for name := range f.children {
		names = append(names, name)
	}
	sort.Strings(names)
	for _, name := range names {
		f.children[name].dump(w, level+1)
	}
}

func (f *memNode) resetToSyncedState() {
	if f.isDir {
		f.children = make(map[string]*memNode)
		for k, v := range f.syncedChildren {
			f.children[k] = v
		}
		for _, v := range f.children {
			v.resetToSyncedState()
		}
	} else {
		f.mu.Lock()
		f.mu.data = append([]byte(nil), f.mu.syncedData...)
		f.mu.Unlock()
	}
}

// memFile is a reader or writer of a node's data, and implements File.
type memFile struct {
	n           *memNode
	fs          *MemFS // nil for a standalone memFile
	rpos        int
	wpos        int
	read, write bool
}

var _ File = (*memFile)(nil)

func (f *memFile) Close() error {
	if n := f.n.refs.Add(-1); n < 0 {
		panic(fmt.Sprintf("pebble: close of unopened file: %d", n))
	}
	f.n = nil
	return nil
}

func (f *memFile) Read(p []byte) (int, error) {
	if !f.read {
		return 0, errors.New("pebble/vfs: file was not opened for reading")
	}
	if f.n.isDir {
		return 0, errors.New("pebble/vfs: cannot read a directory")
	}
	f.n.mu.Lock()
	defer f.n.mu.Unlock()
	if f.rpos >= len(f.n.mu.data) {
		return 0, io.EOF
	}
	n := copy(p, f.n.mu.data[f.rpos:])
	f.rpos += n
	return n, nil
}

func (f *memFile) ReadAt(p []byte, off int64) (int, error) {
	if !f.read {
		return 0, errors.New("pebble/vfs: file was not opened for reading")
	}
	if f.n.isDir {
		return 0, errors.New("pebble/vfs: cannot read a directory")
	}
	f.n.mu.Lock()
	defer f.n.mu.Unlock()
	if off >= int64(len(f.n.mu.data)) {
		return 0, io.EOF
	}
	n := copy(p, f.n.mu.data[off:])
	if n < len(p) {
		return n, io.EOF
	}
	return n, nil
}

func (f *memFile) Write(p []byte) (int, error) {
	if !f.write {
		return 0, errors.New("pebble/vfs: file was not created for writing")
	}
	if f.n.isDir {
		return 0, errors.New("pebble/vfs: cannot write a directory")
	}
	f.n.mu.Lock()
	defer f.n.mu.Unlock()
	f.n.mu.modTime = time.Now()
	if f.wpos+len(p) <= len(f.n.mu.data) {
		n := copy(f.n.mu.data[f.wpos:f.wpos+len(p)], p)
		if n != len(p) {
			panic("stuff")
		}
	} else {
		f.n.mu.data = append(f.n.mu.data[:f.wpos], p...)
	}
	f.wpos += len(p)

	if invariants.Enabled {
		// Mutate the input buffer to flush out bugs in Pebble which expect the
		// input buffer to be unmodified.
		for i := range p {
			p[i] ^= 0xff
		}
	}
	return len(p), nil
}

func (f *memFile) WriteAt(p []byte, ofs int64) (int, error) {
	if !f.write {
		return 0, errors.New("pebble/vfs: file was not created for writing")
	}
	if f.n.isDir {
		return 0, errors.New("pebble/vfs: cannot write a directory")
	}
	f.n.mu.Lock()
	defer f.n.mu.Unlock()
	f.n.mu.modTime = time.Now()

	for len(f.n.mu.data) < int(ofs)+len(p) {
		f.n.mu.data = append(f.n.mu.data, 0)
	}

	n := copy(f.n.mu.data[int(ofs):int(ofs)+len(p)], p)
	if n != len(p) {
		panic("stuff")
	}

	return len(p), nil
}

func (f *memFile) Prefetch(offset int64, length int64) error { return nil }
func (f *memFile) Preallocate(offset, length int64) error    { return nil }

func (f *memFile) Stat() (os.FileInfo, error) {
	return f.n, nil
}

func (f *memFile) Sync() error {
	if f.fs != nil && f.fs.strict {
		f.fs.mu.Lock()
		defer f.fs.mu.Unlock()
		if f.fs.ignoreSyncs {
			return nil
		}
		if f.n.isDir {
			f.n.syncedChildren = make(map[string]*memNode)
			for k, v := range f.n.children {
				f.n.syncedChildren[k] = v
			}
		} else {
			f.n.mu.Lock()
			f.n.mu.syncedData = append([]byte(nil), f.n.mu.data...)
			f.n.mu.Unlock()
		}
	}
	return nil
}

func (f *memFile) SyncData() error {
	return f.Sync()
}

func (f *memFile) SyncTo(length int64) (fullSync bool, err error) {
	// NB: This SyncTo implementation lies, with its return values claiming it
	// synced the data up to `length`. When fullSync=false, SyncTo provides no
	// durability guarantees, so this can help surface bugs where we improperly
	// rely on SyncTo providing durability.
	return false, nil
}

func (f *memFile) Fd() uintptr {
	return InvalidFd
}

// Flush is a no-op and present only to prevent buffering at higher levels
// (e.g. it prevents sstable.Writer from using a bufio.Writer).
func (f *memFile) Flush() error {
	return nil
}

type memFileLock struct {
	y        *MemFS
	f        File
	fullname string
}

func (l *memFileLock) Close() error {
	if l.y == nil {
		return nil
	}
	l.y.lockedFiles.Delete(l.fullname)
	l.y = nil
	return l.f.Close()
}
