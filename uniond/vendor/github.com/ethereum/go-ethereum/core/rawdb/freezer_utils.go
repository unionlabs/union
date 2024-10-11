// Copyright 2022 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package rawdb

import (
	"io"
	"os"
	"path/filepath"
)

// copyFrom copies data from 'srcPath' at offset 'offset' into 'destPath'.
// The 'destPath' is created if it doesn't exist, otherwise it is overwritten.
// Before the copy is executed, there is a callback can be registered to
// manipulate the dest file.
// It is perfectly valid to have destPath == srcPath.
func copyFrom(srcPath, destPath string, offset uint64, before func(f *os.File) error) error {
	// Create a temp file in the same dir where we want it to wind up
	f, err := os.CreateTemp(filepath.Dir(destPath), "*")
	if err != nil {
		return err
	}
	fname := f.Name()

	// Clean up the leftover file
	defer func() {
		if f != nil {
			f.Close()
		}
		os.Remove(fname)
	}()
	// Apply the given function if it's not nil before we copy
	// the content from the src.
	if before != nil {
		if err := before(f); err != nil {
			return err
		}
	}
	// Open the source file
	src, err := os.Open(srcPath)
	if err != nil {
		return err
	}
	if _, err = src.Seek(int64(offset), 0); err != nil {
		src.Close()
		return err
	}
	// io.Copy uses 32K buffer internally.
	_, err = io.Copy(f, src)
	if err != nil {
		src.Close()
		return err
	}
	// Rename the temporary file to the specified dest name.
	// src may be same as dest, so needs to be closed before
	// we do the final move.
	src.Close()

	if err := f.Close(); err != nil {
		return err
	}
	f = nil

	if err := os.Rename(fname, destPath); err != nil {
		return err
	}
	return nil
}

// openFreezerFileForAppend opens a freezer table file and seeks to the end
func openFreezerFileForAppend(filename string) (*os.File, error) {
	// Open the file without the O_APPEND flag
	// because it has differing behaviour during Truncate operations
	// on different OS's
	file, err := os.OpenFile(filename, os.O_RDWR|os.O_CREATE, 0644)
	if err != nil {
		return nil, err
	}
	// Seek to end for append
	if _, err = file.Seek(0, io.SeekEnd); err != nil {
		return nil, err
	}
	return file, nil
}

// openFreezerFileForReadOnly opens a freezer table file for read only access
func openFreezerFileForReadOnly(filename string) (*os.File, error) {
	return os.OpenFile(filename, os.O_RDONLY, 0644)
}

// openFreezerFileTruncated opens a freezer table making sure it is truncated
func openFreezerFileTruncated(filename string) (*os.File, error) {
	return os.OpenFile(filename, os.O_RDWR|os.O_CREATE|os.O_TRUNC, 0644)
}

// truncateFreezerFile resizes a freezer table file and seeks to the end
func truncateFreezerFile(file *os.File, size int64) error {
	if err := file.Truncate(size); err != nil {
		return err
	}
	// Seek to end for append
	if _, err := file.Seek(0, io.SeekEnd); err != nil {
		return err
	}
	return nil
}

// grow prepares the slice space for new item, and doubles the slice capacity
// if space is not enough.
func grow(buf []byte, n int) []byte {
	if cap(buf)-len(buf) < n {
		newcap := 2 * cap(buf)
		if newcap-len(buf) < n {
			newcap = len(buf) + n
		}
		nbuf := make([]byte, len(buf), newcap)
		copy(nbuf, buf)
		buf = nbuf
	}
	buf = buf[:len(buf)+n]
	return buf
}
