// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package remote

import (
	"context"
	"io"
	"os"
	"path"

	"github.com/cockroachdb/pebble/vfs"
)

// NewLocalFS returns a vfs-backed implementation of the remote.Storage
// interface (for testing). All objects will be stored at the directory
// dirname.
func NewLocalFS(dirname string, fs vfs.FS) Storage {
	store := &localFSStore{
		dirname: dirname,
		vfs:     fs,
	}
	return store
}

// localFSStore is a vfs-backed implementation of the remote.Storage
// interface (for testing).
type localFSStore struct {
	dirname string
	vfs     vfs.FS
}

var _ Storage = (*localFSStore)(nil)

// Close is part of the remote.Storage interface.
func (s *localFSStore) Close() error {
	*s = localFSStore{}
	return nil
}

// ReadObject is part of the remote.Storage interface.
func (s *localFSStore) ReadObject(
	ctx context.Context, objName string,
) (_ ObjectReader, objSize int64, _ error) {
	f, err := s.vfs.Open(path.Join(s.dirname, objName))
	if err != nil {
		return nil, 0, err
	}
	stat, err := f.Stat()
	if err != nil {
		return nil, 0, err
	}

	return &localFSReader{f}, stat.Size(), nil
}

type localFSReader struct {
	file vfs.File
}

var _ ObjectReader = (*localFSReader)(nil)

// ReadAt is part of the shared.ObjectReader interface.
func (r *localFSReader) ReadAt(_ context.Context, p []byte, offset int64) error {
	n, err := r.file.ReadAt(p, offset)
	// https://pkg.go.dev/io#ReaderAt
	if err == io.EOF && n == len(p) {
		return nil
	}
	return err
}

// Close is part of the shared.ObjectReader interface.
func (r *localFSReader) Close() error {
	r.file.Close()
	r.file = nil
	return nil
}

// CreateObject is part of the remote.Storage interface.
func (s *localFSStore) CreateObject(objName string) (io.WriteCloser, error) {
	file, err := s.vfs.Create(path.Join(s.dirname, objName))
	return file, err
}

// List is part of the remote.Storage interface.
func (s *localFSStore) List(prefix, delimiter string) ([]string, error) {
	// TODO(josh): For the intended use case of localfs.go of running 'pebble bench',
	// List can always return <nil, nil>, since this indicates a file has only one ref,
	// and since `pebble bench` implies running in a single-pebble-instance context.
	// https://github.com/cockroachdb/pebble/blob/a9a079d4fb6bf4a9ebc52e4d83a76ad4cbf676cb/objstorage/objstorageprovider/shared.go#L292
	return nil, nil
}

// Delete is part of the remote.Storage interface.
func (s *localFSStore) Delete(objName string) error {
	return s.vfs.Remove(path.Join(s.dirname, objName))
}

// Size is part of the remote.Storage interface.
func (s *localFSStore) Size(objName string) (int64, error) {
	f, err := s.vfs.Open(path.Join(s.dirname, objName))
	if err != nil {
		return 0, err
	}
	defer f.Close()
	stat, err := f.Stat()
	if err != nil {
		return 0, err
	}
	return stat.Size(), nil
}

// IsNotExistError is part of the remote.Storage interface.
func (s *localFSStore) IsNotExistError(err error) bool {
	return err == os.ErrNotExist
}
