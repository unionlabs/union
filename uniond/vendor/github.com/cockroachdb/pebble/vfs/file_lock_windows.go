// Copyright 2013 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build windows
// +build windows

package vfs

import (
	"io"

	"golang.org/x/sys/windows"
)

// lockCloser hides all of an windows.Handle's methods, except for Close.
type lockCloser struct {
	fd windows.Handle
}

func (l lockCloser) Close() error {
	return windows.Close(l.fd)
}

// Lock locks the given file. On Windows, Locking will fail if the file is
// already open by the current process.
func (defaultFS) Lock(name string) (io.Closer, error) {
	p, err := windows.UTF16PtrFromString(name)
	if err != nil {
		return nil, err
	}
	fd, err := windows.CreateFile(p,
		windows.GENERIC_READ|windows.GENERIC_WRITE,
		0, nil, windows.CREATE_ALWAYS,
		windows.FILE_ATTRIBUTE_NORMAL,
		0,
	)
	if err != nil {
		return nil, err
	}
	return lockCloser{fd: fd}, nil
}
