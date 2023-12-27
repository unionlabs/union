// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build openbsd
// +build openbsd

package vfs

import "golang.org/x/sys/unix"

func (defaultFS) GetDiskUsage(path string) (DiskUsage, error) {
	stat := unix.Statfs_t{}
	if err := unix.Statfs(path, &stat); err != nil {
		return DiskUsage{}, err
	}

	freeBytes := uint64(stat.F_bsize) * uint64(stat.F_bfree)
	availBytes := uint64(stat.F_bsize) * uint64(stat.F_bavail)
	totalBytes := uint64(stat.F_bsize) * uint64(stat.F_blocks)
	return DiskUsage{
		AvailBytes: availBytes,
		TotalBytes: totalBytes,
		UsedBytes:  totalBytes - freeBytes,
	}, nil
}
