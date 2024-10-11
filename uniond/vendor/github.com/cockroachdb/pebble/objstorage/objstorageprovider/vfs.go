// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package objstorageprovider

import (
	"context"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/cockroachdb/pebble/vfs"
)

func (p *provider) vfsPath(fileType base.FileType, fileNum base.DiskFileNum) string {
	return base.MakeFilepath(p.st.FS, p.st.FSDirName, fileType, fileNum)
}

func (p *provider) vfsOpenForReading(
	ctx context.Context,
	fileType base.FileType,
	fileNum base.DiskFileNum,
	opts objstorage.OpenOptions,
) (objstorage.Readable, error) {
	filename := p.vfsPath(fileType, fileNum)
	file, err := p.st.FS.Open(filename, vfs.RandomReadsOption)
	if err != nil {
		if opts.MustExist {
			base.MustExist(p.st.FS, filename, p.st.Logger, err)
		}
		return nil, err
	}
	readaheadConfig := DefaultReadaheadConfig
	if f := p.st.Local.ReadaheadConfigFn; f != nil {
		readaheadConfig = f()
	}
	return newFileReadable(file, p.st.FS, readaheadConfig, filename)
}

func (p *provider) vfsCreate(
	_ context.Context, fileType base.FileType, fileNum base.DiskFileNum,
) (objstorage.Writable, objstorage.ObjectMetadata, error) {
	filename := p.vfsPath(fileType, fileNum)
	file, err := p.st.FS.Create(filename)
	if err != nil {
		return nil, objstorage.ObjectMetadata{}, err
	}
	file = vfs.NewSyncingFile(file, vfs.SyncingFileOptions{
		NoSyncOnClose: p.st.NoSyncOnClose,
		BytesPerSync:  p.st.BytesPerSync,
	})
	meta := objstorage.ObjectMetadata{
		DiskFileNum: fileNum,
		FileType:    fileType,
	}
	return newFileBufferedWritable(file), meta, nil
}

func (p *provider) vfsRemove(fileType base.FileType, fileNum base.DiskFileNum) error {
	return p.st.FSCleaner.Clean(p.st.FS, fileType, p.vfsPath(fileType, fileNum))
}

// vfsInit finds any local FS objects.
func (p *provider) vfsInit() error {
	listing := p.st.FSDirInitialListing
	if listing == nil {
		var err error
		listing, err = p.st.FS.List(p.st.FSDirName)
		if err != nil {
			return errors.Wrapf(err, "pebble: could not list store directory")
		}
	}

	for _, filename := range listing {
		fileType, fileNum, ok := base.ParseFilename(p.st.FS, filename)
		if ok && fileType == base.FileTypeTable {
			o := objstorage.ObjectMetadata{
				FileType:    fileType,
				DiskFileNum: fileNum,
			}
			p.mu.knownObjects[o.DiskFileNum] = o
		}
	}
	return nil
}

func (p *provider) vfsSync() error {
	p.mu.Lock()
	shouldSync := p.mu.localObjectsChanged
	p.mu.localObjectsChanged = false
	p.mu.Unlock()

	if !shouldSync {
		return nil
	}
	if err := p.fsDir.Sync(); err != nil {
		p.mu.Lock()
		defer p.mu.Unlock()
		p.mu.localObjectsChanged = true
		return err
	}
	return nil
}

func (p *provider) vfsSize(fileType base.FileType, fileNum base.DiskFileNum) (int64, error) {
	filename := p.vfsPath(fileType, fileNum)
	stat, err := p.st.FS.Stat(filename)
	if err != nil {
		return 0, err
	}
	return stat.Size(), nil
}
