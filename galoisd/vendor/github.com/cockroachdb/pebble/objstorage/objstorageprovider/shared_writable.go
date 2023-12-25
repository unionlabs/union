// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package objstorageprovider

import (
	"io"

	"github.com/cockroachdb/pebble/objstorage"
)

// NewRemoteWritable creates an objstorage.Writable out of an io.WriteCloser.
func NewRemoteWritable(obj io.WriteCloser) objstorage.Writable {
	return &sharedWritable{storageWriter: obj}
}

// sharedWritable is a very simple implementation of Writable on top of the
// WriteCloser returned by remote.Storage.CreateObject.
type sharedWritable struct {
	// Either both p and meta must be unset / zero values, or both must be set.
	// The case where both are unset is true only in tests.
	p             *provider
	meta          objstorage.ObjectMetadata
	storageWriter io.WriteCloser
}

var _ objstorage.Writable = (*sharedWritable)(nil)

// Write is part of the Writable interface.
func (w *sharedWritable) Write(p []byte) error {
	_, err := w.storageWriter.Write(p)
	return err
}

// Finish is part of the Writable interface.
func (w *sharedWritable) Finish() error {
	err := w.storageWriter.Close()
	w.storageWriter = nil
	if err != nil {
		w.Abort()
		return err
	}

	// Create the marker object.
	if w.p != nil {
		if err := w.p.sharedCreateRef(w.meta); err != nil {
			w.Abort()
			return err
		}
	}
	return nil
}

// Abort is part of the Writable interface.
func (w *sharedWritable) Abort() {
	if w.storageWriter != nil {
		_ = w.storageWriter.Close()
		w.storageWriter = nil
	}
	if w.p != nil {
		w.p.removeMetadata(w.meta.DiskFileNum)
	}
	// TODO(radu): delete the object if it was created.
}
