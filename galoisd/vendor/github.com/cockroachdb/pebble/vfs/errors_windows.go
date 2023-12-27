// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build windows
// +build windows

package vfs

import (
	"github.com/cockroachdb/errors"
	"golang.org/x/sys/windows"
)

var errNotEmpty = windows.ERROR_DIR_NOT_EMPTY

// IsNoSpaceError returns true if the given error indicates that the disk is
// out of space.
func IsNoSpaceError(err error) bool {
	return errors.Is(err, windows.ERROR_DISK_FULL) ||
		errors.Is(err, windows.ERROR_HANDLE_DISK_FULL)
}
