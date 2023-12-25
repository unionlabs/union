package grocksdb

// #include <stdlib.h>
// #include "rocksdb/c.h"
import "C"

// ColumnFamilyHandle represents a handle to a ColumnFamily.
type ColumnFamilyHandle struct {
	c *C.rocksdb_column_family_handle_t
}

// NewNativeColumnFamilyHandle creates a ColumnFamilyHandle object.
func newNativeColumnFamilyHandle(c *C.rocksdb_column_family_handle_t) *ColumnFamilyHandle {
	return &ColumnFamilyHandle{c: c}
}

// ID returned id of Column family.
func (h *ColumnFamilyHandle) ID() uint32 {
	return uint32(C.rocksdb_column_family_handle_get_id(h.c))
}

// Name returned name of Column family.
func (h *ColumnFamilyHandle) Name() string {
	var len C.size_t
	cValue := C.rocksdb_column_family_handle_get_name(h.c, &len)
	return toString(cValue, C.int(len))
}

// Destroy calls the destructor of the underlying column family handle.
func (h *ColumnFamilyHandle) Destroy() {
	C.rocksdb_column_family_handle_destroy(h.c)
	h.c = nil
}

// ColumnFamilyHandles represents collection of multiple column family handle.
type ColumnFamilyHandles []*ColumnFamilyHandle

func (cfs ColumnFamilyHandles) toCSlice() columnFamilySlice {
	cCFs := make(columnFamilySlice, len(cfs))
	for i, cf := range cfs {
		cCFs[i] = cf.c
	}
	return cCFs
}
