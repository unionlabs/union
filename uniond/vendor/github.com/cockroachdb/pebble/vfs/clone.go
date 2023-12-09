// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package vfs

import (
	"io"
	"sort"

	"github.com/cockroachdb/errors/oserror"
)

type cloneOpts struct {
	skip    func(string) bool
	sync    bool
	tryLink bool
}

// A CloneOption configures the behavior of Clone.
type CloneOption func(*cloneOpts)

// CloneSkip configures Clone to skip files for which the provided function
// returns true when passed the file's path.
func CloneSkip(fn func(string) bool) CloneOption {
	return func(co *cloneOpts) { co.skip = fn }
}

// CloneSync configures Clone to sync files and directories.
var CloneSync CloneOption = func(o *cloneOpts) { o.sync = true }

// CloneTryLink configures Clone to link files to the destination if the source and
// destination filesystems are the same. If the source and destination
// filesystems are not the same or the filesystem does not support linking, then
// Clone falls back to copying.
var CloneTryLink CloneOption = func(o *cloneOpts) { o.tryLink = true }

// Clone recursively copies a directory structure from srcFS to dstFS. srcPath
// specifies the path in srcFS to copy from and must be compatible with the
// srcFS path format. dstDir is the target directory in dstFS and must be
// compatible with the dstFS path format. Returns (true,nil) on a successful
// copy, (false,nil) if srcPath does not exist, and (false,err) if an error
// occurred.
func Clone(srcFS, dstFS FS, srcPath, dstPath string, opts ...CloneOption) (bool, error) {
	var o cloneOpts
	for _, opt := range opts {
		opt(&o)
	}

	srcFile, err := srcFS.Open(srcPath)
	if err != nil {
		if oserror.IsNotExist(err) {
			// Ignore non-existent errors. Those will translate into non-existent
			// files in the destination filesystem.
			return false, nil
		}
		return false, err
	}
	defer srcFile.Close()

	stat, err := srcFile.Stat()
	if err != nil {
		return false, err
	}

	if stat.IsDir() {
		if err := dstFS.MkdirAll(dstPath, 0755); err != nil {
			return false, err
		}
		list, err := srcFS.List(srcPath)
		if err != nil {
			return false, err
		}
		// Sort the paths so we get deterministic test output.
		sort.Strings(list)
		for _, name := range list {
			if o.skip != nil && o.skip(srcFS.PathJoin(srcPath, name)) {
				continue
			}
			_, err := Clone(srcFS, dstFS, srcFS.PathJoin(srcPath, name), dstFS.PathJoin(dstPath, name), opts...)
			if err != nil {
				return false, err
			}
		}

		if o.sync {
			dir, err := dstFS.OpenDir(dstPath)
			if err != nil {
				return false, err
			}
			if err := dir.Sync(); err != nil {
				return false, err
			}
			if err := dir.Close(); err != nil {
				return false, err
			}
		}

		return true, nil
	}

	// If the source and destination filesystems are the same and the user
	// specified they'd prefer to link if possible, try to use a hardlink,
	// falling back to copying if it fails.
	if srcFS == dstFS && o.tryLink {
		if err := LinkOrCopy(srcFS, srcPath, dstPath); oserror.IsNotExist(err) {
			// Clone's semantics are such that it returns (false,nil) if the
			// source does not exist.
			return false, nil
		} else if err != nil {
			return false, err
		} else {
			return true, nil
		}
	}

	data, err := io.ReadAll(srcFile)
	if err != nil {
		return false, err
	}
	dstFile, err := dstFS.Create(dstPath)
	if err != nil {
		return false, err
	}
	if _, err = dstFile.Write(data); err != nil {
		return false, err
	}
	if o.sync {
		if err := dstFile.Sync(); err != nil {
			return false, err
		}
	}
	if err := dstFile.Close(); err != nil {
		return false, err
	}
	return true, nil
}
