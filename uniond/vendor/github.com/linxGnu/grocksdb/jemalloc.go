//go:build !testing && jemalloc

package grocksdb

// #include "rocksdb/c.h"
import "C"

// CreateJemallocNodumpAllocator generates memory allocator which allocates through
// Jemalloc and utilize MADV_DONTDUMP through madvise to exclude cache items from core dump.
// Applications can use the allocator with block cache to exclude block cache
// usage from core dump.
//
// Implementation details:
// The JemallocNodumpAllocator creates a dedicated jemalloc arena, and all
// allocations of the JemallocNodumpAllocator are through the same arena.
// The memory allocator hooks memory allocation of the arena, and calls
// madvise() with MADV_DONTDUMP flag to exclude the piece of memory from
// core dump. Side benefit of using single arena would be reduction of jemalloc
// metadata for some workloads.
//
// To mitigate mutex contention for using one single arena, jemalloc tcache
// (thread-local cache) is enabled to cache unused allocations for future use.
// The tcache normally incurs 0.5M extra memory usage per-thread. The usage
// can be reduced by limiting allocation sizes to cache.
func CreateJemallocNodumpAllocator() (*MemoryAllocator, error) {
	var cErr *C.char

	c := C.rocksdb_jemalloc_nodump_allocator_create(&cErr)

	// check error
	if err := fromCError(cErr); err != nil {
		return nil, err
	}

	return &MemoryAllocator{
		c: c,
	}, nil
}
