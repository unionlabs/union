// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build linux
// +build linux

package vfs

import "golang.org/x/sys/unix"

// Calls Fadvise with FADV_RANDOM to disable readahead on a file descriptor.
func fadviseRandom(f uintptr) error {
	return unix.Fadvise(int(f), 0, 0, unix.FADV_RANDOM)
}

// Calls Fadvise with FADV_SEQUENTIAL to enable readahead on a file descriptor.
func fadviseSequential(f uintptr) error {
	return unix.Fadvise(int(f), 0, 0, unix.FADV_SEQUENTIAL)
}
