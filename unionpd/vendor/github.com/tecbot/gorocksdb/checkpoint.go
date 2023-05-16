package gorocksdb

// #include <stdlib.h>
// #include "rocksdb/c.h"
import "C"

import (
	"errors"
	"unsafe"
)

// Checkpoint provides Checkpoint functionality.
// Checkpoints provide persistent snapshots of RocksDB databases.
type Checkpoint struct {
	c *C.rocksdb_checkpoint_t
}

// NewNativeCheckpoint creates a new checkpoint.
func NewNativeCheckpoint(c *C.rocksdb_checkpoint_t) *Checkpoint {
	return &Checkpoint{c}
}

// CreateCheckpoint builds an openable snapshot of RocksDB on the same disk, which
// accepts an output directory on the same disk, and under the directory
// (1) hard-linked SST files pointing to existing live SST files
// SST files will be copied if output directory is on a different filesystem
// (2) a copied manifest files and other files
// The directory should not already exist and will be created by this API.
// The directory will be an absolute path
// log_size_for_flush: if the total log file size is equal or larger than
// this value, then a flush is triggered for all the column families. The
// default value is 0, which means flush is always triggered. If you move
// away from the default, the checkpoint may not contain up-to-date data
// if WAL writing is not always enabled.
// Flush will always trigger if it is 2PC.
func (checkpoint *Checkpoint) CreateCheckpoint(checkpoint_dir string, log_size_for_flush uint64) error {
	var (
		cErr *C.char
	)

	cDir := C.CString(checkpoint_dir)
	defer C.free(unsafe.Pointer(cDir))

	C.rocksdb_checkpoint_create(checkpoint.c, cDir, C.uint64_t(log_size_for_flush), &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return errors.New(C.GoString(cErr))
	}
	return nil
}

// Destroy deallocates the Checkpoint object.
func (checkpoint *Checkpoint) Destroy() {
	C.rocksdb_checkpoint_object_destroy(checkpoint.c)
	checkpoint.c = nil
}
