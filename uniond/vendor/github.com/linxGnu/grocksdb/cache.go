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

// NewHyperClockCache creates a new hyper clock cache.
func NewHyperClockCache(capacity, estimatedEntryCharge int) *Cache {
	cCache := C.rocksdb_cache_create_hyper_clock(C.size_t(capacity), C.size_t(estimatedEntryCharge))
	return newNativeCache(cCache)
}

// NewHyperClockCacheWithOpts creates a hyper clock cache with predefined options.
func NewHyperClockCacheWithOpts(opt *HyperClockCacheOptions) *Cache {
	cCache := C.rocksdb_cache_create_hyper_clock_opts(opt.c)
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

// TODO: try to re-enable later, along with next release of RocksDB

// // GetTableAddressCount returns the number of ways the hash function is divided for addressing
// // entries. Zero means "not supported." This is used for inspecting the load
// // factor, along with GetOccupancyCount().
// func (c *Cache) GetTableAddressCount() int {
// 	return int(rocksdb_cache_get_table_address_count(c.c))
// }

// // GetOccupancyCount returns the number of entries currently tracked in the table. SIZE_MAX
// // means "not supported." This is used for inspecting the load factor, along
// // with GetTableAddressCount().
// func (c *Cache) GetOccupancyCount() int {
// 	return int(rocksdb_cache_get_occupancy_count(c.c))
// }

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

// HyperClockCacheOptions are options for HyperClockCache.
//
// HyperClockCache is a lock-free Cache alternative for RocksDB block cache
// that offers much improved CPU efficiency vs. LRUCache under high parallel
// load or high contention, with some caveats:
// * Not a general Cache implementation: can only be used for
// BlockBasedTableOptions::block_cache, which RocksDB uses in a way that is
// compatible with HyperClockCache.
// * Requires an extra tuning parameter: see estimated_entry_charge below.
// Similarly, substantially changing the capacity with SetCapacity could
// harm efficiency.
// * SecondaryCache is not yet supported.
// * Cache priorities are less aggressively enforced, which could cause
// cache dilution from long range scans (unless they use fill_cache=false).
// * Can be worse for small caches, because if almost all of a cache shard is
// pinned (more likely with non-partitioned filters), then CLOCK eviction
// becomes very CPU intensive.
//
// See internal cache/clock_cache.h for full description.
type HyperClockCacheOptions struct {
	c *C.rocksdb_hyper_clock_cache_options_t
}

// NewHyperClockCacheOptions creates new options for hyper clock cache.
func NewHyperClockCacheOptions(capacity, estimatedEntryCharge int) *HyperClockCacheOptions {
	return &HyperClockCacheOptions{
		c: C.rocksdb_hyper_clock_cache_options_create(C.size_t(capacity), C.size_t(estimatedEntryCharge)),
	}
}

// SetCapacity sets the capacity of the cache.
func (h *HyperClockCacheOptions) SetCapacity(capacity int) {
	C.rocksdb_hyper_clock_cache_options_set_capacity(h.c, C.size_t(capacity))
}

// SetEstimatedEntryCharge sets the estimated average `charge` associated with cache entries.
//
// This is a critical configuration parameter for good performance from the hyper
// cache, because having a table size that is fixed at creation time greatly
// reduces the required synchronization between threads.
// * If the estimate is substantially too low (e.g. less than half the true
// average) then metadata space overhead with be substantially higher (e.g.
// 200 bytes per entry rather than 100). With kFullChargeCacheMetadata, this
// can slightly reduce cache hit rates, and slightly reduce access times due
// to the larger working memory size.
// * If the estimate is substantially too high (e.g. 25% higher than the true
// average) then there might not be sufficient slots in the hash table for
// both efficient operation and capacity utilization (hit rate). The hyper
// cache will evict entries to prevent load factors that could dramatically
// affect lookup times, instead letting the hit rate suffer by not utilizing
// the full capacity.
//
// A reasonable choice is the larger of block_size and metadata_block_size.
// When WriteBufferManager (and similar) charge memory usage to the block
// cache, this can lead to the same effect as estimate being too low, which
// is better than the opposite. Therefore, the general recommendation is to
// assume that other memory charged to block cache could be negligible, and
// ignore it in making the estimate.
//
// The best parameter choice based on a cache in use is given by
// GetUsage() / GetOccupancyCount(), ignoring metadata overheads such as
// with kDontChargeCacheMetadata. More precisely with
// kFullChargeCacheMetadata is (GetUsage() - 64 * GetTableAddressCount()) /
// GetOccupancyCount(). However, when the average value size might vary
// (e.g. balance between metadata and data blocks in cache), it is better
// to estimate toward the lower side than the higher side.
func (h *HyperClockCacheOptions) SetEstimatedEntryCharge(v int) {
	C.rocksdb_hyper_clock_cache_options_set_estimated_entry_charge(h.c, C.size_t(v))
}

// SetCapacity sets number of shards used for this cache.
func (h *HyperClockCacheOptions) SetNumShardBits(n int) {
	C.rocksdb_hyper_clock_cache_options_set_num_shard_bits(h.c, C.int(n))
}

// SetMemoryAllocator for this cache.
func (h *HyperClockCacheOptions) SetMemoryAllocator(m *MemoryAllocator) {
	C.rocksdb_hyper_clock_cache_options_set_memory_allocator(h.c, m.c)
}

// Destroy the options.
func (h *HyperClockCacheOptions) Destroy() {
	C.rocksdb_hyper_clock_cache_options_destroy(h.c)
	h.c = nil
}
