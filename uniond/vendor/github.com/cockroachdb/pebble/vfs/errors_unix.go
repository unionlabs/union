// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build darwin || dragonfly || freebsd || linux || openbsd || netbsd
// +build darwin dragonfly freebsd linux openbsd netbsd

package vfs

import (
	"github.com/cockroachdb/errors"
	"golang.org/x/sys/unix"
)

var errNotEmpty = unix.ENOTEMPTY

// IsNoSpaceError returns true if the given error indicates that the disk is
// out of space.
func IsNoSpaceError(err error) bool {
	return errors.Is(err, unix.ENOSPC)
}
