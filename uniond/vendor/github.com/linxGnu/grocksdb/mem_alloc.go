package grocksdb

// #include <stdlib.h>
// #include "rocksdb/c.h"
import "C"

// MemoryAllocator wraps memory allocator for rocksdb.
type MemoryAllocator struct {
	c *C.rocksdb_memory_allocator_t
}

// Destroy this mem allocator.
func (m *MemoryAllocator) Destroy() {
	C.rocksdb_memory_allocator_destroy(m.c)
}
