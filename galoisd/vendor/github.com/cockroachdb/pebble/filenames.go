// Copyright 2012 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"fmt"

	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/vfs"
)

type fileType = base.FileType

// FileNum is an identifier for a file within a database.
type FileNum = base.FileNum

const (
	fileTypeLog      = base.FileTypeLog
	fileTypeLock     = base.FileTypeLock
	fileTypeTable    = base.FileTypeTable
	fileTypeManifest = base.FileTypeManifest
	fileTypeCurrent  = base.FileTypeCurrent
	fileTypeOptions  = base.FileTypeOptions
	fileTypeTemp     = base.FileTypeTemp
	fileTypeOldTemp  = base.FileTypeOldTemp
)

// setCurrentFile sets the CURRENT file to point to the manifest with
// provided file number.
//
// NB: This is a low-level routine and typically not what you want to
// use. Newer versions of Pebble running newer format major versions do
// not use the CURRENT file. See setCurrentFunc in version_set.go.
func setCurrentFile(dirname string, fs vfs.FS, fileNum base.DiskFileNum) error {
	newFilename := base.MakeFilepath(fs, dirname, fileTypeCurrent, fileNum)
	oldFilename := base.MakeFilepath(fs, dirname, fileTypeTemp, fileNum)
	fs.Remove(oldFilename)
	f, err := fs.Create(oldFilename)
	if err != nil {
		return err
	}
	if _, err := fmt.Fprintf(f, "MANIFEST-%s\n", fileNum); err != nil {
		return err
	}
	if err := f.Sync(); err != nil {
		return err
	}
	if err := f.Close(); err != nil {
		return err
	}
	return fs.Rename(oldFilename, newFilename)
}
