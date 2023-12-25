package grocksdb

// #include <stdlib.h>
// #include "rocksdb/c.h"
import "C"

// MemoryUsage contains memory usage statistics provided by RocksDB
type MemoryUsage struct {
	// MemTableTotal estimates memory usage of all mem-tables
	MemTableTotal uint64
	// MemTableUnflushed estimates memory usage of unflushed mem-tables
	MemTableUnflushed uint64
	// MemTableReadersTotal memory usage of table readers (indexes and bloom filters)
	MemTableReadersTotal uint64
	// CacheTotal memory usage of cache
	CacheTotal uint64
}

// GetApproximateMemoryUsageByType returns summary
// memory usage stats for given databases and caches.
func GetApproximateMemoryUsageByType(dbs []*DB, caches []*Cache) (result *MemoryUsage, err error) {
	// register memory consumers
	consumers := C.rocksdb_memory_consumers_create()

	for _, db := range dbs {
		if db != nil {
			C.rocksdb_memory_consumers_add_db(consumers, db.c)
		}
	}

	for _, cache := range caches {
		if cache != nil {
			C.rocksdb_memory_consumers_add_cache(consumers, cache.c)
		}
	}

	// obtain memory usage stats
	var cErr *C.char
	memoryUsage := C.rocksdb_approximate_memory_usage_create(consumers, &cErr)
	if err = fromCError(cErr); err == nil {
		result = &MemoryUsage{
			MemTableTotal:        uint64(C.rocksdb_approximate_memory_usage_get_mem_table_total(memoryUsage)),
			MemTableUnflushed:    uint64(C.rocksdb_approximate_memory_usage_get_mem_table_unflushed(memoryUsage)),
			MemTableReadersTotal: uint64(C.rocksdb_approximate_memory_usage_get_mem_table_readers_total(memoryUsage)),
			CacheTotal:           uint64(C.rocksdb_approximate_memory_usage_get_cache_total(memoryUsage)),
		}
	}

	C.rocksdb_approximate_memory_usage_destroy(memoryUsage)
	C.rocksdb_memory_consumers_destroy(consumers)

	return
}
