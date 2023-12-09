package grocksdb

// #include "rocksdb/c.h"
// #include "grocksdb.h"
import "C"

// IndexType specifies the index type that will be used for this table.
type IndexType uint

const (
	// KBinarySearchIndexType a space efficient index block that is optimized for
	// binary-search-based index.
	KBinarySearchIndexType IndexType = 0x00

	// KHashSearchIndexType the hash index, if enabled, will do the hash lookup when
	// `Options.prefix_extractor` is provided.
	KHashSearchIndexType IndexType = 0x01

	// KTwoLevelIndexSearchIndexType a two-level index implementation. Both levels are binary search indexes.
	KTwoLevelIndexSearchIndexType IndexType = 0x02

	// KBinarySearchWithFirstKey like KBinarySearchIndexType, but index also contains
	// first key of each block.
	//
	// This allows iterators to defer reading the block until it's actually
	// needed. May significantly reduce read amplification of short range scans.
	// Without it, iterator seek usually reads one block from each level-0 file
	// and from each level, which may be expensive.
	// Works best in combination with:
	//  - IndexShorteningMode::kNoShortening,
	//  - custom FlushBlockPolicy to cut blocks at some meaningful boundaries,
	//    e.g. when prefix changes.
	// Makes the index significantly bigger (2x or more), especially when keys
	// are long.
	//
	// IO errors are not handled correctly in this mode right now: if an error
	// happens when lazily reading a block in value(), value() returns empty
	// slice, and you need to call Valid()/status() afterwards.
	KBinarySearchWithFirstKey IndexType = 0x03
)

// DataBlockIndexType specifies index type that will be used for the data block.
type DataBlockIndexType uint

const (
	// KDataBlockIndexTypeBinarySearch is traditional block type
	KDataBlockIndexTypeBinarySearch DataBlockIndexType = 0
	// KDataBlockIndexTypeBinarySearchAndHash additional hash index
	KDataBlockIndexTypeBinarySearchAndHash DataBlockIndexType = 1
)

// BlockBasedTableOptions represents block-based table options.
type BlockBasedTableOptions struct {
	c *C.rocksdb_block_based_table_options_t

	// Hold references for GC.
	cache     *Cache
	compCache *Cache

	// We keep these so we can free their memory in Destroy.
	cFp *C.rocksdb_filterpolicy_t
}

// NewDefaultBlockBasedTableOptions creates a default BlockBasedTableOptions object.
func NewDefaultBlockBasedTableOptions() *BlockBasedTableOptions {
	return newNativeBlockBasedTableOptions(C.rocksdb_block_based_options_create())
}

// NewNativeBlockBasedTableOptions creates a BlockBasedTableOptions object.
func newNativeBlockBasedTableOptions(c *C.rocksdb_block_based_table_options_t) *BlockBasedTableOptions {
	return &BlockBasedTableOptions{c: c}
}

// Destroy deallocates the BlockBasedTableOptions object.
func (opts *BlockBasedTableOptions) Destroy() {
	C.rocksdb_block_based_options_destroy(opts.c)
	opts.c = nil
	opts.cache = nil
	opts.compCache = nil
}

// SetChecksum sets checksum types.
//
//	enum ChecksumType : char {
//	  kNoChecksum = 0x0,
//	  kCRC32c = 0x1,
//	  kxxHash = 0x2,
//	  kxxHash64 = 0x3,
//	  kXXH3 = 0x4,  // Supported since RocksDB 6.27
//	};
func (opts *BlockBasedTableOptions) SetChecksum(csType int8) {
	C.rocksdb_block_based_options_set_checksum(opts.c, C.char(csType))
}

// SetCacheIndexAndFilterBlocks is indicating if we'd put index/filter blocks to the block cache.
// If not specified, each "table reader" object will pre-load index/filter
// block during table initialization.
// Default: false
func (opts *BlockBasedTableOptions) SetCacheIndexAndFilterBlocks(value bool) {
	C.rocksdb_block_based_options_set_cache_index_and_filter_blocks(opts.c, boolToChar(value))
}

// SetPinL0FilterAndIndexBlocksInCache sets cache_index_and_filter_blocks.
// If is true and the below is true (hash_index_allow_collision), then
// filter and index blocks are stored in the cache, but a reference is
// held in the "table reader" object so the blocks are pinned and only
// evicted from cache when the table reader is freed.
func (opts *BlockBasedTableOptions) SetPinL0FilterAndIndexBlocksInCache(value bool) {
	C.rocksdb_block_based_options_set_pin_l0_filter_and_index_blocks_in_cache(opts.c, boolToChar(value))
}

// SetBlockSize sets the approximate size of user data packed per block.
// Note that the block size specified here corresponds opts uncompressed data.
// The actual size of the unit read from disk may be smaller if
// compression is enabled. This parameter can be changed dynamically.
// Default: 4K
func (opts *BlockBasedTableOptions) SetBlockSize(blockSize int) {
	C.rocksdb_block_based_options_set_block_size(opts.c, C.size_t(blockSize))
}

// SetBlockSizeDeviation sets the block size deviation.
// This is used opts close a block before it reaches the configured
// 'block_size'. If the percentage of free space in the current block is less
// than this specified number and adding a new record opts the block will
// exceed the configured block size, then this block will be closed and the
// new record will be written opts the next block.
// Default: 10
func (opts *BlockBasedTableOptions) SetBlockSizeDeviation(blockSizeDeviation int) {
	C.rocksdb_block_based_options_set_block_size_deviation(opts.c, C.int(blockSizeDeviation))
}

// SetBlockRestartInterval sets the number of keys between
// restart points for delta encoding of keys.
// This parameter can be changed dynamically. Most clients should
// leave this parameter alone.
// Default: 16
func (opts *BlockBasedTableOptions) SetBlockRestartInterval(blockRestartInterval int) {
	C.rocksdb_block_based_options_set_block_restart_interval(opts.c, C.int(blockRestartInterval))
}

// SetFilterPolicy sets the filter policy opts reduce disk reads.
// Many applications will benefit from passing the result of
// NewBloomFilterPolicy() here.
//
// Note: this op is `move`, fp is no longer usable.
//
// Default: nil
func (opts *BlockBasedTableOptions) SetFilterPolicy(fp *NativeFilterPolicy) {
	opts.cFp = fp.c
	fp.c = nil
	C.rocksdb_block_based_options_set_filter_policy(opts.c, opts.cFp)
}

// SetNoBlockCache specify whether block cache should be used or not.
// Default: false
func (opts *BlockBasedTableOptions) SetNoBlockCache(value bool) {
	C.rocksdb_block_based_options_set_no_block_cache(opts.c, boolToChar(value))
}

// SetBlockCache sets the control over blocks (user data is stored in a set of blocks, and
// a block is the unit of reading from disk).
//
// If set, use the specified cache for blocks.
// If nil, rocksdb will auoptsmatically create and use an 8MB internal cache.
// Default: nil
func (opts *BlockBasedTableOptions) SetBlockCache(cache *Cache) {
	opts.cache = cache
	C.rocksdb_block_based_options_set_block_cache(opts.c, cache.c)
}

// SetWholeKeyFiltering specify if whole keys in the filter (not just prefixes)
// should be placed.
// This must generally be true for gets opts be efficient.
// Default: true
func (opts *BlockBasedTableOptions) SetWholeKeyFiltering(value bool) {
	C.rocksdb_block_based_options_set_whole_key_filtering(opts.c, boolToChar(value))
}

// SetIndexType sets the index type used for this table.
// kBinarySearch:
// A space efficient index block that is optimized for
// binary-search-based index.
//
// kHashSearch:
// The hash index, if enabled, will do the hash lookup when
// `Options.prefix_extractor` is provided.
//
// kTwoLevelIndexSearch:
// A two-level index implementation. Both levels are binary search indexes.
// Default: kBinarySearch
func (opts *BlockBasedTableOptions) SetIndexType(value IndexType) {
	C.rocksdb_block_based_options_set_index_type(opts.c, C.int(value))
}

// SetDataBlockIndexType sets data block index type
func (opts *BlockBasedTableOptions) SetDataBlockIndexType(value DataBlockIndexType) {
	C.rocksdb_block_based_options_set_data_block_index_type(opts.c, C.int(value))
}

// SetDataBlockHashRatio is valid only when data_block_hash_index_type is
// KDataBlockIndexTypeBinarySearchAndHash.
//
// Default value: 0.75
func (opts *BlockBasedTableOptions) SetDataBlockHashRatio(value float64) {
	C.rocksdb_block_based_options_set_data_block_hash_ratio(opts.c, C.double(value))
}

// SetIndexBlockRestartInterval same as block_restart_interval but used for the index block.
func (opts *BlockBasedTableOptions) SetIndexBlockRestartInterval(value int) {
	C.rocksdb_block_based_options_set_index_block_restart_interval(opts.c, C.int(value))
}

// SetMetadataBlockSize sets block size for partitioned metadata. Currently applied to indexes when
// kTwoLevelIndexSearch is used and to filters when partition_filters is used.
// Note: Since in the current implementation the filters and index partitions
// are aligned, an index/filter block is created when either index or filter
// block size reaches the specified limit.
// Note: this limit is currently applied to only index blocks; a filter
// partition is cut right after an index block is cut.
func (opts *BlockBasedTableOptions) SetMetadataBlockSize(value uint64) {
	C.rocksdb_block_based_options_set_metadata_block_size(opts.c, C.uint64_t(value))
}

// SetPartitionFilters use partitioned full filters for each SST file. This option is
// incompatible with block-based filters.
//
// Note: currently this option requires kTwoLevelIndexSearch to be set as
// well.
func (opts *BlockBasedTableOptions) SetPartitionFilters(value bool) {
	C.rocksdb_block_based_options_set_partition_filters(opts.c, boolToChar(value))
}

// SetOptimizeFiltersForMemory to generate Bloom/Ribbon filters that minimize memory
// internal fragmentation.
//
// When false, malloc_usable_size is not available, or format_version < 5,
// filters are generated without regard to internal fragmentation when
// loaded into memory (historical behavior). When true (and
// malloc_usable_size is available and format_version >= 5), then
// filters are generated to "round up" and "round down" their sizes to
// minimize internal fragmentation when loaded into memory, assuming the
// reading DB has the same memory allocation characteristics as the
// generating DB. This option does not break forward or backward
// compatibility.
//
// While individual filters will vary in bits/key and false positive rate
// when setting is true, the implementation attempts to maintain a weighted
// average FP rate for filters consistent with this option set to false.
//
// With Jemalloc for example, this setting is expected to save about 10% of
// the memory footprint and block cache charge of filters, while increasing
// disk usage of filters by about 1-2% due to encoding efficiency losses
// with variance in bits/key.
//
// NOTE: Because some memory counted by block cache might be unmapped pages
// within internal fragmentation, this option can increase observed RSS
// memory usage. With cache_index_and_filter_blocks=true, this option makes
// the block cache better at using space it is allowed. (These issues
// should not arise with partitioned filters.)
//
// NOTE: Do not set to true if you do not trust malloc_usable_size. With
// this option, RocksDB might access an allocated memory object beyond its
// original size if malloc_usable_size says it is safe to do so. While this
// can be considered bad practice, it should not produce undefined behavior
// unless malloc_usable_size is buggy or broken.
//
// Default: false
func (opts *BlockBasedTableOptions) SetOptimizeFiltersForMemory(value bool) {
	C.rocksdb_block_based_options_set_optimize_filters_for_memory(opts.c, boolToChar(value))
}

// SetUseDeltaEncoding uses delta encoding to compress keys in blocks.
// ReadOptions::pin_data requires this option to be disabled.
//
// Default: true
func (opts *BlockBasedTableOptions) SetUseDeltaEncoding(value bool) {
	C.rocksdb_block_based_options_set_use_delta_encoding(opts.c, boolToChar(value))
}

// SetFormatVersion set format version. We currently have five options:
// 0 -- This version is currently written out by all RocksDB's versions by
// default.  Can be read by really old RocksDB's. Doesn't support changing
// checksum (default is CRC32).
// 1 -- Can be read by RocksDB's versions since 3.0. Supports non-default
// checksum, like xxHash. It is written by RocksDB when
// BlockBasedTableOptions::checksum is something other than kCRC32c. (version
// 0 is silently upconverted)
// 2 -- Can be read by RocksDB's versions since 3.10. Changes the way we
// encode compressed blocks with LZ4, BZip2 and Zlib compression. If you
// don't plan to run RocksDB before version 3.10, you should probably use
// this.
// 3 -- Can be read by RocksDB's versions since 5.15. Changes the way we
// encode the keys in index blocks. If you don't plan to run RocksDB before
// version 5.15, you should probably use this.
// This option only affects newly written tables. When reading existing
// tables, the information about version is read from the footer.
// 4 -- Can be read by RocksDB's versions since 5.16. Changes the way we
// encode the values in index blocks. If you don't plan to run RocksDB before
// version 5.16 and you are using index_block_restart_interval > 1, you should
// probably use this as it would reduce the index size.
// This option only affects newly written tables. When reading existing
// tables, the information about version is read from the footer.
func (opts *BlockBasedTableOptions) SetFormatVersion(value int) {
	C.rocksdb_block_based_options_set_format_version(opts.c, C.int(value))
}

// SetCacheIndexAndFilterBlocksWithHighPriority if cache_index_and_filter_blocks is enabled,
// cache index and filter blocks with high priority. If set to true, depending on implementation of
// block cache, index and filter blocks may be less likely to be evicted
// than data blocks.
//
// Default: true.
func (opts *BlockBasedTableOptions) SetCacheIndexAndFilterBlocksWithHighPriority(value bool) {
	C.rocksdb_block_based_options_set_cache_index_and_filter_blocks_with_high_priority(opts.c, boolToChar(value))
}

// SetPinTopLevelIndexAndFilter if cache_index_and_filter_blocks is true and the below is true, then
// the top-level index of partitioned filter and index blocks are stored in
// the cache, but a reference is held in the "table reader" object so the
// blocks are pinned and only evicted from cache when the table reader is
// freed. This is not limited to l0 in LSM tree.
//
// Default: true.
func (opts *BlockBasedTableOptions) SetPinTopLevelIndexAndFilter(value bool) {
	C.rocksdb_block_based_options_set_pin_top_level_index_and_filter(opts.c, boolToChar(value))
}
