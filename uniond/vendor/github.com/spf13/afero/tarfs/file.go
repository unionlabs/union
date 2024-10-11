package tarfs

import (
	"archive/tar"
	"bytes"
	"os"
	"path/filepath"
	"sort"
	"syscall"

	"github.com/spf13/afero"
)

type File struct {
	h      *tar.Header
	data   *bytes.Reader
	closed bool
	fs     *Fs
}

func (f *File) Close() error {
	if f.closed {
		return afero.ErrFileClosed
	}

	f.closed = true
	f.h = nil
	f.data = nil
	f.fs = nil

	return nil
}

func (f *File) Read(p []byte) (n int, err error) {
	if f.closed {
		return 0, afero.ErrFileClosed
	}

	if f.h.Typeflag == tar.TypeDir {
		return 0, syscall.EISDIR
	}

	return f.data.Read(p)
}

func (f *File) ReadAt(p []byte, off int64) (n int, err error) {
	if f.closed {
		return 0, afero.ErrFileClosed
	}

	if f.h.Typeflag == tar.TypeDir {
		return 0, syscall.EISDIR
	}

	return f.data.ReadAt(p, off)
}

func (f *File) Seek(offset int64, whence int) (int64, error) {
	if f.closed {
		return 0, afero.ErrFileClosed
	}

	if f.h.Typeflag == tar.TypeDir {
		return 0, syscall.EISDIR
	}

	return f.data.Seek(offset, whence)
}

func (f *File) Write(p []byte) (n int, err error) { return 0, syscall.EROFS }

func (f *File) WriteAt(p []byte, off int64) (n int, err error) { return 0, syscall.EROFS }

func (f *File) Name() string {
	return filepath.Join(splitpath(f.h.Name))
}

func (f *File) getDirectoryNames() ([]string, error) {
	d, ok := f.fs.files[f.Name()]
	if !ok {
		return nil, &os.PathError{Op: "readdir", Path: f.Name(), Err: syscall.ENOENT}
	}

	var names []string
	for n := range d {
		names = append(names, n)
	}
	sort.Strings(names)

	return names, nil
}

func (f *File) Readdir(count int) ([]os.FileInfo, error) {
	if f.closed {
		return nil, afero.ErrFileClosed
	}

	if !f.h.FileInfo().IsDir() {
		return nil, syscall.ENOTDIR
	}

	names, err := f.getDirectoryNames()
	if err != nil {
		return nil, err
	}

	d := f.fs.files[f.Name()]
	var fi []os.FileInfo
	for _, n := range names {
		if n == "" {
			continue
		}

		f := d[n]
		fi = append(fi, f.h.FileInfo())
		if count > 0 && len(fi) >= count {
			break
		}
	}

	return fi, nil
}

func (f *File) Readdirnames(n int) ([]string, error) {
	fi, err := f.Readdir(n)
	if err != nil {
		return nil, err
	}

	var names []string
	for _, f := range fi {
		names = append(names, f.Name())
	}

	return names, nil
}

func (f *File) Stat() (os.FileInfo, error) { return f.h.FileInfo(), nil }

func (f *File) Sync() error { return nil }

func (f *File) Truncate(size int64) error { return syscall.EROFS }

func (f *File) WriteString(s string) (ret int, err error) { return 0, syscall.EROFS }
