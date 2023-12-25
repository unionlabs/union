package grocksdb

// #include "rocksdb/c.h"
import "C"
import "unsafe"

// Snapshot provides a consistent view of read operations in a DB.
type Snapshot struct {
	c *C.rocksdb_snapshot_t
}

// NewNativeSnapshot creates a Snapshot object.
func newNativeSnapshot(c *C.rocksdb_snapshot_t) *Snapshot {
	return &Snapshot{c: c}
}

// Destroy deallocates the Snapshot object.
func (snapshot *Snapshot) Destroy() {
	C.rocksdb_free(unsafe.Pointer(snapshot.c))
	snapshot.c = nil
}
