package grocksdb

// #include "rocksdb/c.h"
import "C"

// Cache is a cache used to store data read from data in memory.
type Cache struct {
	c *C.rocksdb_cache_t
}

// NewLRUCache creates a new LRU Cache object with the capacity given.
func NewLRUCache(capacity uint64) *Cache {
	cCache := C.rocksdb_cache_create_lru(C.size_t(capacity))
	return newNativeCache(cCache)
}

// NewLRUCacheWithOptions creates a new LRU Cache from options.
func NewLRUCacheWithOptions(opt *LRUCacheOptions) *Cache {
	cCache := C.rocksdb_cache_create_lru_opts(opt.c)
	return newNativeCache(cCache)
}

// NewNativeCache creates a Cache object.
func newNativeCache(c *C.rocksdb_cache_t) *Cache {
	return &Cache{c: c}
}

// GetUsage returns the Cache memory usage.
func (c *Cache) GetUsage() uint64 {
	return uint64(C.rocksdb_cache_get_usage(c.c))
}

// GetPinnedUsage returns the Cache pinned memory usage.
func (c *Cache) GetPinnedUsage() uint64 {
	return uint64(C.rocksdb_cache_get_pinned_usage(c.c))
}

// SetCapacity sets capacity of the cache.
func (c *Cache) SetCapacity(value uint64) {
	C.rocksdb_cache_set_capacity(c.c, C.size_t(value))
}

// GetCapacity returns capacity of the cache.
func (c *Cache) GetCapacity() uint64 {
	return uint64(C.rocksdb_cache_get_capacity(c.c))
}

// Disowndata call this on shutdown if you want to speed it up. Cache will disown
// any underlying data and will not free it on delete. This call will leak
// memory - call this only if you're shutting down the process.
// Any attempts of using cache after this call will fail terribly.
// Always delete the DB object before calling this method!
func (c *Cache) DisownData() {
	C.rocksdb_cache_disown_data(c.c)
}

// Destroy deallocates the Cache object.
func (c *Cache) Destroy() {
	C.rocksdb_cache_destroy(c.c)
	c.c = nil
}

// LRUCacheOptions are options for LRU Cache.
type LRUCacheOptions struct {
	c *C.rocksdb_lru_cache_options_t
}

// NewLRUCacheOptions creates lru cache options.
func NewLRUCacheOptions() *LRUCacheOptions {
	return &LRUCacheOptions{c: C.rocksdb_lru_cache_options_create()}
}

// Destroy lru cache options.
func (l *LRUCacheOptions) Destroy() {
	C.rocksdb_lru_cache_options_destroy(l.c)
	l.c = nil
}

// SetCapacity sets capacity for this lru cache.
func (l *LRUCacheOptions) SetCapacity(s uint) {
	C.rocksdb_lru_cache_options_set_capacity(l.c, C.size_t(s))
}

// SetCapacity sets number of shards used for this lru cache.
func (l *LRUCacheOptions) SetNumShardBits(n int) {
	C.rocksdb_lru_cache_options_set_num_shard_bits(l.c, C.int(n))
}

// SetMemoryAllocator for this lru cache.
func (l *LRUCacheOptions) SetMemoryAllocator(m *MemoryAllocator) {
	C.rocksdb_lru_cache_options_set_memory_allocator(l.c, m.c)
}
