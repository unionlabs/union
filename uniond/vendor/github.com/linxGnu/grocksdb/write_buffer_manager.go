package grocksdb

// #include "rocksdb/c.h"
import "C"

type WriteBufferManager struct {
	p *C.rocksdb_write_buffer_manager_t
}

// NewWriteBufferManager creates new WriteBufferManager object.
//
// @bufferSize: bufferSize = 0 indicates no limit. Memory won't be capped.
// memory_usage() won't be valid and ShouldFlush() will always return true.
//
// @allowStall: if set true, it will enable stalling of writes when
// memory_usage() exceeds buffer_size. It will wait for flush to complete and
// memory usage to drop down.
func NewWriteBufferManager(bufferSize int, allowStall bool) *WriteBufferManager {
	return &WriteBufferManager{
		p: C.rocksdb_write_buffer_manager_create(C.size_t(bufferSize), C.bool(allowStall)),
	}
}

// NewWriteBufferManagerWithCache similars to NewWriteBufferManager, we'll put dummy entries in the cache and
// cost the memory allocated to the cache. It can be used even if bufferSize = 0.
func NewWriteBufferManagerWithCache(bufferSize int, cache *Cache, allowStall bool) *WriteBufferManager {
	return &WriteBufferManager{
		p: C.rocksdb_write_buffer_manager_create_with_cache(C.size_t(bufferSize), cache.c, C.bool(allowStall)),
	}
}

// Enabled returns true if buffer_limit is passed to limit the total memory usage and
// is greater than 0.
func (w *WriteBufferManager) Enabled() bool {
	return bool(C.rocksdb_write_buffer_manager_enabled(w.p))
}

// CostToCache returns true if pointer to cache is passed.
func (w *WriteBufferManager) CostToCache() bool {
	return bool(C.rocksdb_write_buffer_manager_cost_to_cache(w.p))
}

// MemoryUsage returns the total memory used by memtables.
// Only valid if enabled().
func (w *WriteBufferManager) MemoryUsage() int {
	return int(C.rocksdb_write_buffer_manager_memory_usage(w.p))
}

// MemtableMemoryUsage returns the total memory used by active memtables.
func (w *WriteBufferManager) MemtableMemoryUsage() int {
	return int(C.rocksdb_write_buffer_manager_mutable_memtable_memory_usage(w.p))
}

// DummyEntriesInCacheUsage returns number of dummy entries in cache.
func (w *WriteBufferManager) DummyEntriesInCacheUsage() int {
	return int(C.rocksdb_write_buffer_manager_dummy_entries_in_cache_usage(w.p))
}

// BufferSize returns the buffer size.
func (w *WriteBufferManager) BufferSize() int {
	return int(C.rocksdb_write_buffer_manager_buffer_size(w.p))
}

// SetBufferSize sets buffer size.
func (w *WriteBufferManager) SetBufferSize(size int) {
	C.rocksdb_write_buffer_manager_set_buffer_size(w.p, C.size_t(size))
}

// ToggleAllowStall sets allow stall option.
func (w *WriteBufferManager) ToggleAllowStall(v bool) {
	C.rocksdb_write_buffer_manager_set_allow_stall(w.p, C.bool(v))
}

// Destroy the object.
func (w *WriteBufferManager) Destroy() {
	C.rocksdb_write_buffer_manager_destroy(w.p)
	w.p = nil
}
