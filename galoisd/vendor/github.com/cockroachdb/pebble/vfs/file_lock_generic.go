// Copyright 2012 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build !darwin && !dragonfly && !freebsd && !linux && !netbsd && !openbsd && !solaris && !windows
// +build !darwin,!dragonfly,!freebsd,!linux,!netbsd,!openbsd,!solaris,!windows

package vfs

import (
	"io"
	"runtime"

	"github.com/cockroachdb/errors"
)

func (defFS) Lock(name string) (io.Closer, error) {
	return nil, errors.Errorf("pebble: file locking is not implemented on %s/%s",
		errors.Safe(runtime.GOOS), errors.Safe(runtime.GOARCH))
}
