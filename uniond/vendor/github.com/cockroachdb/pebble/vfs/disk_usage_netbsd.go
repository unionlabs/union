// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build netbsd
// +build netbsd

package vfs

import "golang.org/x/sys/unix"

func (defaultFS) GetDiskUsage(path string) (DiskUsage, error) {
	stat := unix.Statvfs_t{}
	if err := unix.Statvfs(path, &stat); err != nil {
		return DiskUsage{}, err
	}

	freeBytes := uint64(stat.Bsize) * uint64(stat.Bfree)
	availBytes := uint64(stat.Bsize) * uint64(stat.Bavail)
	totalBytes := uint64(stat.Bsize) * uint64(stat.Blocks)
	return DiskUsage{
		AvailBytes: availBytes,
		TotalBytes: totalBytes,
		UsedBytes:  totalBytes - freeBytes,
	}, nil
}
