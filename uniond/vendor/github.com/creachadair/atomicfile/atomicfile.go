// Package atomicfile implements all-or-nothing file replacement by staging
// output to a temporary file adjacent to the target, and renaming over the
// target when the temporary is closed.
//
// If (and only if) the implementation of rename is atomic the replacement is
// also atomic. Since IEEE Std 1003.1 requires rename to be atomic, this is
// ordinarily true on POSIX-compatible filesystems.
package atomicfile

import (
	"errors"
	"io"
	"os"
	"path/filepath"
)

// New constructs a new writable File with the given mode that, when
// successfully closed will be renamed to target.
func New(target string, mode os.FileMode) (*File, error) {
	dir, name := filepath.Split(target)
	f, err := os.CreateTemp(filepath.Clean(dir), "aftmp."+name)
	if err != nil {
		return nil, err
	} else if err := f.Chmod(mode); err != nil {
		f.Close()
		os.Remove(f.Name())
		return nil, err
	}
	return &File{
		tmp:    f,
		target: target,
	}, nil
}

// Tx runs f with a file constructed by New.  If f reports an error, the file
// is automatically cancelled and Tx returns the error from f. Otherwise, Tx
// returns the error from calling Close on the file.
func Tx(target string, mode os.FileMode, f func(*File) error) error {
	tmp, err := New(target, mode)
	if err != nil {
		return err
	}
	defer tmp.Cancel()
	if err := f(tmp); err != nil {
		return err
	}
	return tmp.Close()
}

// WriteData copies data to the specified target path via a File.
func WriteData(target string, data []byte, mode os.FileMode) error {
	return Tx(target, mode, func(f *File) error {
		_, err := f.Write(data)
		return err
	})
}

// WriteAll copies all the data from r to the specified target path via a File.
// It reports the total number of bytes copied.
func WriteAll(target string, r io.Reader, mode os.FileMode) (int64, error) {
	var nw int64
	err := Tx(target, mode, func(f *File) error {
		var err error
		nw, err = f.tmp.ReadFrom(r)
		return err
	})
	return nw, err
}

// A File is a writable temporary file that will be renamed to a target path
// when successfully closed.
type File struct {
	tmp    *os.File
	target string
}

// Close closes the temporary associated with f and renames it to the
// designated target file. If closing the temporary fails, or if the rename
// fails, the temporary file is unlinked before Close returns.
func (f *File) Close() error {
	if f.tmp == nil {
		return errors.New("file is already closed")
	}
	name := f.tmp.Name()
	if err := f.tmp.Close(); err != nil {
		os.Remove(name) // best-effort
		return err
	}
	if err := os.Rename(name, f.target); err != nil {
		os.Remove(name) // best-effort
		return err
	}
	f.tmp = nil // rename succeeded
	return nil
}

// Cancel closes the temporary associated with f and discards it.
// It is safe to call Cancel even if f.Close has already succeeded.
func (f *File) Cancel() {
	// Clean up the temp file (only) if a rename has not yet occurred, or it failed.
	// The check averts an A-B-A conflict during the window after renaming.
	if tmp := f.tmp; tmp != nil {
		f.tmp = nil
		tmp.Close()
		os.Remove(tmp.Name())
	}
}

// Write writes data to f, satisfying io.Writer.
func (f *File) Write(data []byte) (int, error) { return f.tmp.Write(data) }

// ReadFrom implements the io.ReaderFrom interface to the underlying temporary.
func (f *File) ReadFrom(r io.Reader) (int64, error) { return f.tmp.ReadFrom(r) }
