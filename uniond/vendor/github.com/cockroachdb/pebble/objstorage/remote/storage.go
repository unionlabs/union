// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package remote

import (
	"context"
	"io"
)

// Locator is an opaque string identifying a remote.Storage implementation.
//
// The Locator must not contain secrets (like authentication keys). Locators are
// stored on disk in the shared object catalog and are passed around as part of
// RemoteObjectBacking; they can also appear in error messages.
type Locator string

// StorageFactory is used to return Storage implementations based on locators. A
// Pebble store that uses shared storage is configured with a StorageFactory.
type StorageFactory interface {
	CreateStorage(locator Locator) (Storage, error)
}

// SharedLevelsStart denotes the highest (i.e. lowest numbered) level that will
// have sstables shared across Pebble instances when doing skip-shared
// iteration (see db.ScanInternal) or shared file ingestion (see
// db.IngestAndExcise).
const SharedLevelsStart = 5

// CreateOnSharedStrategy specifies what table files should be created on shared
// storage. For use with CreateOnShared in options.
type CreateOnSharedStrategy int

const (
	// CreateOnSharedNone denotes no files being created on shared storage.
	CreateOnSharedNone CreateOnSharedStrategy = iota
	// CreateOnSharedLower denotes the creation of files in lower levels of the
	// LSM (specifically, L5 and L6 as they're below SharedLevelsStart) on
	// shared storage, and higher levels on local storage.
	CreateOnSharedLower
	// CreateOnSharedAll denotes the creation of all sstables on shared storage.
	CreateOnSharedAll
)

// ShouldCreateShared returns whether new table files at the specified level
// should be created on shared storage.
func ShouldCreateShared(strategy CreateOnSharedStrategy, level int) bool {
	switch strategy {
	case CreateOnSharedAll:
		return true
	case CreateOnSharedNone:
		return false
	case CreateOnSharedLower:
		return level >= SharedLevelsStart
	default:
		panic("unexpected CreateOnSharedStrategy value")
	}
}

// Storage is an interface for a blob storage driver. This is lower-level
// than an FS-like interface, however FS/File-like abstractions can be built on
// top of these methods.
//
// TODO(bilal): Consider pushing shared file obsoletion as well as path
// generation behind this interface.
type Storage interface {
	io.Closer

	// ReadObject returns an ObjectReader that can be used to perform reads on an
	// object, along with the total size of the object.
	ReadObject(ctx context.Context, objName string) (_ ObjectReader, objSize int64, _ error)

	// CreateObject returns a writer for the object at the request name. A new
	// empty object is created if CreateObject is called on an existing object.
	//
	// A Writer *must* be closed via either Close, and if closing returns a
	// non-nil error, that error should be handled or reported to the user -- an
	// implementation may buffer written data until Close and only then return
	// an error, or Write may return an opaque io.EOF with the underlying cause
	// returned by the subsequent Close().
	//
	// TODO(radu): if we encounter some unrelated error while writing to the
	// WriteCloser, we'd want to abort the whole thing rather than letting Close
	// finalize the upload.
	CreateObject(objName string) (io.WriteCloser, error)

	// List enumerates files within the supplied prefix, returning a list of
	// objects within that prefix. If delimiter is non-empty, names which have the
	// same prefix, prior to the delimiter but after the prefix, are grouped into a
	// single result which is that prefix. The order that results are returned is
	// undefined. If a prefix is specified, the prefix is trimmed from the result
	// list.
	//
	// An example would be, if the storage contains objects a, b/4, b/5 and b/6,
	// these would be the return values:
	//   List("", "") -> ["a", "b/4", "b/5", "b/6"]
	//   List("", "/") -> ["a", "b"]
	//   List("b", "/") -> ["4", "5", "6"]
	//   List("b", "") -> ["/4", "/5", "/6"]
	List(prefix, delimiter string) ([]string, error)

	// Delete removes the named object from the store.
	Delete(objName string) error

	// Size returns the length of the named object in bytesWritten.
	Size(objName string) (int64, error)

	// IsNotExistError returns true if the given error (returned by a method in
	// this interface) indicates that the object does not exist.
	IsNotExistError(err error) bool
}

// ObjectReader is used to perform reads on an object.
type ObjectReader interface {
	// ReadAt reads len(p) bytes into p starting at offset off.
	//
	// Does not return partial results; if offset + len(p) is past the end of the
	// object, an error is returned.
	//
	// Clients of ReadAt can execute parallel ReadAt calls on the same
	// ObjectReader.
	ReadAt(ctx context.Context, p []byte, offset int64) error

	Close() error
}
