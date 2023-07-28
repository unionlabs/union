package gorocksdb

// #include "rocksdb/c.h"
import "C"

// Snapshot provides a consistent view of read operations in a DB.
type Snapshot struct {
	c *C.rocksdb_snapshot_t
}

// NewNativeSnapshot creates a Snapshot object.
func NewNativeSnapshot(c *C.rocksdb_snapshot_t) *Snapshot {
	return &Snapshot{c}
}
