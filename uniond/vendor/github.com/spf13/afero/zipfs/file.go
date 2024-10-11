package zipfs

import (
	"archive/zip"
	"io"
	"os"
	"path/filepath"
	"syscall"

	"github.com/spf13/afero"
)

type File struct {
	fs            *Fs
	zipfile       *zip.File
	reader        io.ReadCloser
	offset        int64
	isdir, closed bool
	buf           []byte
}

func (f *File) fillBuffer(offset int64) (err error) {
	if f.reader == nil {
		if f.reader, err = f.zipfile.Open(); err != nil {
			return
		}
	}
	if offset > int64(f.zipfile.UncompressedSize64) {
		offset = int64(f.zipfile.UncompressedSize64)
		err = io.EOF
	}
	if len(f.buf) >= int(offset) {
		return
	}
	buf := make([]byte, int(offset)-len(f.buf))
	if n, readErr := io.ReadFull(f.reader, buf); n > 0 {
		f.buf = append(f.buf, buf[:n]...)
	} else if readErr != nil {
		err = readErr
	}
	return
}

func (f *File) Close() (err error) {
	f.zipfile = nil
	f.closed = true
	f.buf = nil
	if f.reader != nil {
		err = f.reader.Close()
		f.reader = nil
	}
	return
}

func (f *File) Read(p []byte) (n int, err error) {
	if f.isdir {
		return 0, syscall.EISDIR
	}
	if f.closed {
		return 0, afero.ErrFileClosed
	}
	err = f.fillBuffer(f.offset + int64(len(p)))
	n = copy(p, f.buf[f.offset:])
	f.offset += int64(n)
	return
}

func (f *File) ReadAt(p []byte, off int64) (n int, err error) {
	if f.isdir {
		return 0, syscall.EISDIR
	}
	if f.closed {
		return 0, afero.ErrFileClosed
	}
	err = f.fillBuffer(off + int64(len(p)))
	n = copy(p, f.buf[int(off):])
	return
}

func (f *File) Seek(offset int64, whence int) (int64, error) {
	if f.isdir {
		return 0, syscall.EISDIR
	}
	if f.closed {
		return 0, afero.ErrFileClosed
	}
	switch whence {
	case io.SeekStart:
	case io.SeekCurrent:
		offset += f.offset
	case io.SeekEnd:
		offset += int64(f.zipfile.UncompressedSize64)
	default:
		return 0, syscall.EINVAL
	}
	if offset < 0 || offset > int64(f.zipfile.UncompressedSize64) {
		return 0, afero.ErrOutOfRange
	}
	f.offset = offset
	return offset, nil
}

func (f *File) Write(p []byte) (n int, err error) { return 0, syscall.EPERM }

func (f *File) WriteAt(p []byte, off int64) (n int, err error) { return 0, syscall.EPERM }

func (f *File) Name() string {
	if f.zipfile == nil {
		return string(filepath.Separator)
	}
	return filepath.Join(splitpath(f.zipfile.Name))
}

func (f *File) getDirEntries() (map[string]*zip.File, error) {
	if !f.isdir {
		return nil, syscall.ENOTDIR
	}
	name := f.Name()
	entries, ok := f.fs.files[name]
	if !ok {
		return nil, &os.PathError{Op: "readdir", Path: name, Err: syscall.ENOENT}
	}
	return entries, nil
}

func (f *File) Readdir(count int) (fi []os.FileInfo, err error) {
	zipfiles, err := f.getDirEntries()
	if err != nil {
		return nil, err
	}
	for _, zipfile := range zipfiles {
		fi = append(fi, zipfile.FileInfo())
		if count > 0 && len(fi) >= count {
			break
		}
	}
	return
}

func (f *File) Readdirnames(count int) (names []string, err error) {
	zipfiles, err := f.getDirEntries()
	if err != nil {
		return nil, err
	}
	for filename := range zipfiles {
		names = append(names, filename)
		if count > 0 && len(names) >= count {
			break
		}
	}
	return
}

func (f *File) Stat() (os.FileInfo, error) {
	if f.zipfile == nil {
		return &pseudoRoot{}, nil
	}
	return f.zipfile.FileInfo(), nil
}

func (f *File) Sync() error { return nil }

func (f *File) Truncate(size int64) error { return syscall.EPERM }

func (f *File) WriteString(s string) (ret int, err error) { return 0, syscall.EPERM }
