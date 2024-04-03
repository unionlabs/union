package grocksdb

// #include "rocksdb/c.h"
import "C"

// StatisticsLevel can be used to reduce statistics overhead by skipping certain
// types of stats in the stats collection process.
type StatisticsLevel int

const (
	// StatisticsLevelDisableAll disable all metrics.
	StatisticsLevelDisableAll StatisticsLevel = iota

	// StatisticsLevelExceptHistogramOrTimers disable timer stats and skip histogram stats.
	StatisticsLevelExceptHistogramOrTimers

	// StatisticsLevelExceptTimers skip timer stats
	StatisticsLevelExceptTimers

	// StatisticsLevelExceptDetailedTimers collect all stats except time inside mutex lock AND time spent on
	// compression.
	StatisticsLevelExceptDetailedTimers

	// StatisticsLevelExceptTimeForMutex collect all stats except the counters requiring to get time inside the
	// mutex lock.
	StatisticsLevelExceptTimeForMutex

	// StatisticsLevelAll collect all stats including measuring duration of mutex operations.
	// If getting time is expensive on the platform to run it can
	// reduce scalability to more threads especially for writes.
	StatisticsLevelAll
)

const (
	// StatisticsLevelExceptTickers disable tickers.
	StatisticsLevelExceptTickers = StatisticsLevelDisableAll
)

type TickerType uint32

const (
	// total block cache misses
	// REQUIRES: BLOCK_CACHE_MISS == BLOCK_CACHE_INDEX_MISS +
	//                               BLOCK_CACHE_FILTER_MISS +
	//                               BLOCK_CACHE_DATA_MISS;
	TickerType_BLOCK_CACHE_MISS TickerType = iota
	// total block cache hit
	// REQUIRES: BLOCK_CACHE_HIT == BLOCK_CACHE_INDEX_HIT +
	//                              BLOCK_CACHE_FILTER_HIT +
	//                              BLOCK_CACHE_DATA_HIT;
	TickerType_BLOCK_CACHE_HIT
	// # of blocks added to block cache.
	TickerType_BLOCK_CACHE_ADD
	// # of failures when adding blocks to block cache.
	TickerType_BLOCK_CACHE_ADD_FAILURES
	// # of times cache miss when accessing index block from block cache.
	TickerType_BLOCK_CACHE_INDEX_MISS
	// # of times cache hit when accessing index block from block cache.
	TickerType_BLOCK_CACHE_INDEX_HIT
	// # of index blocks added to block cache.
	TickerType_BLOCK_CACHE_INDEX_ADD
	// # of bytes of index blocks inserted into cache
	TickerType_BLOCK_CACHE_INDEX_BYTES_INSERT
	// # of times cache miss when accessing filter block from block cache.
	TickerType_BLOCK_CACHE_FILTER_MISS
	// # of times cache hit when accessing filter block from block cache.
	TickerType_BLOCK_CACHE_FILTER_HIT
	// # of filter blocks added to block cache.
	TickerType_BLOCK_CACHE_FILTER_ADD
	// # of bytes of bloom filter blocks inserted into cache
	TickerType_BLOCK_CACHE_FILTER_BYTES_INSERT
	// # of times cache miss when accessing data block from block cache.
	TickerType_BLOCK_CACHE_DATA_MISS
	// # of times cache hit when accessing data block from block cache.
	TickerType_BLOCK_CACHE_DATA_HIT
	// # of data blocks added to block cache.
	TickerType_BLOCK_CACHE_DATA_ADD
	// # of bytes of data blocks inserted into cache
	TickerType_BLOCK_CACHE_DATA_BYTES_INSERT
	// # of bytes read from cache.
	TickerType_BLOCK_CACHE_BYTES_READ
	// # of bytes written into cache.
	TickerType_BLOCK_CACHE_BYTES_WRITE

	// # of times bloom filter has avoided file reads i.e. negatives.
	TickerType_BLOOM_FILTER_USEFUL
	// # of times bloom FullFilter has not avoided the reads.
	TickerType_BLOOM_FILTER_FULL_POSITIVE
	// # of times bloom FullFilter has not avoided the reads and data actually
	// exist.
	TickerType_BLOOM_FILTER_FULL_TRUE_POSITIVE

	// # persistent cache hit
	TickerType_PERSISTENT_CACHE_HIT
	// # persistent cache miss
	TickerType_PERSISTENT_CACHE_MISS

	// # total simulation block cache hits
	TickerType_SIM_BLOCK_CACHE_HIT
	// # total simulation block cache misses
	TickerType_SIM_BLOCK_CACHE_MISS

	// # of memtable hits.
	TickerType_MEMTABLE_HIT
	// # of memtable misses.
	TickerType_MEMTABLE_MISS

	// # of Get() queries served by L0
	TickerType_GET_HIT_L0
	// # of Get() queries served by L1
	TickerType_GET_HIT_L1
	// # of Get() queries served by L2 and up
	TickerType_GET_HIT_L2_AND_UP

	/**
	 * COMPACTION_KEY_DROP_* count the reasons for key drop during compaction
	 * There are 4 reasons currently.
	 */
	TickerType_COMPACTION_KEY_DROP_NEWER_ENTRY // key was written with a newer value.
	// Also includes keys dropped for range del.
	TickerType_COMPACTION_KEY_DROP_OBSOLETE       // The key is obsolete.
	TickerType_COMPACTION_KEY_DROP_RANGE_DEL      // key was covered by a range tombstone.
	TickerType_COMPACTION_KEY_DROP_USER           // user compaction function has dropped the key.
	TickerType_COMPACTION_RANGE_DEL_DROP_OBSOLETE // all keys in range were deleted.
	// Deletions obsoleted before bottom level due to file gap optimization.
	TickerType_COMPACTION_OPTIMIZED_DEL_DROP_OBSOLETE
	// If a compaction was canceled in sfm to prevent ENOSPC
	TickerType_COMPACTION_CANCELLED

	// Number of keys written to the database via the Put and Write call's
	TickerType_NUMBER_KEYS_WRITTEN
	// Number of Keys read
	TickerType_NUMBER_KEYS_READ
	// Number keys updated if inplace update is enabled
	TickerType_NUMBER_KEYS_UPDATED
	// The number of uncompressed bytes issued by DB::Put() DB::Delete()
	// DB::Merge() and DB::Write().
	TickerType_BYTES_WRITTEN
	// The number of uncompressed bytes read from DB::Get().  It could be
	// either from memtables cache or table files.
	// For the number of logical bytes read from DB::MultiGet()
	// please use NUMBER_MULTIGET_BYTES_READ.
	TickerType_BYTES_READ
	// The number of calls to seek/next/prev
	TickerType_NUMBER_DB_SEEK
	TickerType_NUMBER_DB_NEXT
	TickerType_NUMBER_DB_PREV
	// The number of calls to seek/next/prev that returned data
	TickerType_NUMBER_DB_SEEK_FOUND
	TickerType_NUMBER_DB_NEXT_FOUND
	TickerType_NUMBER_DB_PREV_FOUND
	// The number of uncompressed bytes read from an iterator.
	// Includes size of key and value.
	TickerType_ITER_BYTES_READ
	TickerType_NO_FILE_OPENS
	TickerType_NO_FILE_ERRORS
	// Writer has to wait for compaction or flush to finish.
	TickerType_STALL_MICROS
	// The wait time for db mutex.
	// Disabled by default. To enable it set stats level to kAll
	TickerType_DB_MUTEX_WAIT_MICROS

	// Number of MultiGet calls keys read and bytes read
	TickerType_NUMBER_MULTIGET_CALLS
	TickerType_NUMBER_MULTIGET_KEYS_READ
	TickerType_NUMBER_MULTIGET_BYTES_READ

	TickerType_NUMBER_MERGE_FAILURES

	// Prefix filter stats when used for point lookups (Get / MultiGet).
	// (For prefix filter stats on iterators see *_LEVEL_SEEK_*.)
	// Checked: filter was queried
	TickerType_BLOOM_FILTER_PREFIX_CHECKED
	// Useful: filter returned false so prevented accessing data+index blocks
	TickerType_BLOOM_FILTER_PREFIX_USEFUL
	// True positive: found a key matching the point query. When another key
	// with the same prefix matches it is considered a false positive by
	// these statistics even though the filter returned a true positive.
	TickerType_BLOOM_FILTER_PREFIX_TRUE_POSITIVE

	// Number of times we had to reseek inside an iteration to skip
	// over large number of keys with same userkey.
	TickerType_NUMBER_OF_RESEEKS_IN_ITERATION

	// Record the number of calls to GetUpdatesSince. Useful to keep track of
	// transaction log iterator refreshes
	TickerType_GET_UPDATES_SINCE_CALLS
	TickerType_WAL_FILE_SYNCED // Number of times WAL sync is done
	TickerType_WAL_FILE_BYTES  // Number of bytes written to WAL

	// Writes can be processed by requesting thread or by the thread at the
	// head of the writers queue.
	TickerType_WRITE_DONE_BY_SELF
	TickerType_WRITE_DONE_BY_OTHER // Equivalent to writes done for others
	TickerType_WRITE_WITH_WAL      // Number of Write calls that request WAL
	TickerType_COMPACT_READ_BYTES  // Bytes read during compaction
	TickerType_COMPACT_WRITE_BYTES // Bytes written during compaction
	TickerType_FLUSH_WRITE_BYTES   // Bytes written during flush

	// Compaction read and write statistics broken down by CompactionReason
	TickerType_COMPACT_READ_BYTES_MARKED
	TickerType_COMPACT_READ_BYTES_PERIODIC
	TickerType_COMPACT_READ_BYTES_TTL
	TickerType_COMPACT_WRITE_BYTES_MARKED
	TickerType_COMPACT_WRITE_BYTES_PERIODIC
	TickerType_COMPACT_WRITE_BYTES_TTL

	// Number of table's properties loaded directly from file without creating
	// table reader object.
	TickerType_NUMBER_DIRECT_LOAD_TABLE_PROPERTIES
	TickerType_NUMBER_SUPERVERSION_ACQUIRES
	TickerType_NUMBER_SUPERVERSION_RELEASES
	TickerType_NUMBER_SUPERVERSION_CLEANUPS

	// # of compressions/decompressions executed
	TickerType_NUMBER_BLOCK_COMPRESSED
	TickerType_NUMBER_BLOCK_DECOMPRESSED

	// DEPRECATED / unused (see NUMBER_BLOCK_COMPRESSION_*)
	TickerType_NUMBER_BLOCK_NOT_COMPRESSED
	TickerType_MERGE_OPERATION_TOTAL_TIME
	TickerType_FILTER_OPERATION_TOTAL_TIME

	// Row cache.
	TickerType_ROW_CACHE_HIT
	TickerType_ROW_CACHE_MISS

	// Read amplification statistics.
	// Read amplification can be calculated using this formula
	// (READ_AMP_TOTAL_READ_BYTES / READ_AMP_ESTIMATE_USEFUL_BYTES)
	//
	// REQUIRES: ReadOptions::read_amp_bytes_per_bit to be enabled
	TickerType_READ_AMP_ESTIMATE_USEFUL_BYTES // Estimate of total bytes actually used.
	TickerType_READ_AMP_TOTAL_READ_BYTES      // Total size of loaded data blocks.

	// Number of refill intervals where rate limiter's bytes are fully consumed.
	TickerType_NUMBER_RATE_LIMITER_DRAINS

	// Number of internal keys skipped by Iterator
	TickerType_NUMBER_ITER_SKIP

	// BlobDB specific stats
	// # of Put/PutTTL/PutUntil to BlobDB. Only applicable to legacy BlobDB.
	TickerType_BLOB_DB_NUM_PUT
	// # of Write to BlobDB. Only applicable to legacy BlobDB.
	TickerType_BLOB_DB_NUM_WRITE
	// # of Get to BlobDB. Only applicable to legacy BlobDB.
	TickerType_BLOB_DB_NUM_GET
	// # of MultiGet to BlobDB. Only applicable to legacy BlobDB.
	TickerType_BLOB_DB_NUM_MULTIGET
	// # of Seek/SeekToFirst/SeekToLast/SeekForPrev to BlobDB iterator. Only
	// applicable to legacy BlobDB.
	TickerType_BLOB_DB_NUM_SEEK
	// # of Next to BlobDB iterator. Only applicable to legacy BlobDB.
	TickerType_BLOB_DB_NUM_NEXT
	// # of Prev to BlobDB iterator. Only applicable to legacy BlobDB.
	TickerType_BLOB_DB_NUM_PREV
	// # of keys written to BlobDB. Only applicable to legacy BlobDB.
	TickerType_BLOB_DB_NUM_KEYS_WRITTEN
	// # of keys read from BlobDB. Only applicable to legacy BlobDB.
	TickerType_BLOB_DB_NUM_KEYS_READ
	// # of bytes (key + value) written to BlobDB. Only applicable to legacy
	// BlobDB.
	TickerType_BLOB_DB_BYTES_WRITTEN
	// # of bytes (keys + value) read from BlobDB. Only applicable to legacy
	// BlobDB.
	TickerType_BLOB_DB_BYTES_READ
	// # of keys written by BlobDB as non-TTL inlined value. Only applicable to
	// legacy BlobDB.
	TickerType_BLOB_DB_WRITE_INLINED
	// # of keys written by BlobDB as TTL inlined value. Only applicable to legacy
	// BlobDB.
	TickerType_BLOB_DB_WRITE_INLINED_TTL
	// # of keys written by BlobDB as non-TTL blob value. Only applicable to
	// legacy BlobDB.
	TickerType_BLOB_DB_WRITE_BLOB
	// # of keys written by BlobDB as TTL blob value. Only applicable to legacy
	// BlobDB.
	TickerType_BLOB_DB_WRITE_BLOB_TTL
	// # of bytes written to blob file.
	TickerType_BLOB_DB_BLOB_FILE_BYTES_WRITTEN
	// # of bytes read from blob file.
	TickerType_BLOB_DB_BLOB_FILE_BYTES_READ
	// # of times a blob files being synced.
	TickerType_BLOB_DB_BLOB_FILE_SYNCED
	// # of blob index evicted from base DB by BlobDB compaction filter because
	// of expiration. Only applicable to legacy BlobDB.
	TickerType_BLOB_DB_BLOB_INDEX_EXPIRED_COUNT
	// size of blob index evicted from base DB by BlobDB compaction filter
	// because of expiration. Only applicable to legacy BlobDB.
	TickerType_BLOB_DB_BLOB_INDEX_EXPIRED_SIZE
	// # of blob index evicted from base DB by BlobDB compaction filter because
	// of corresponding file deleted. Only applicable to legacy BlobDB.
	TickerType_BLOB_DB_BLOB_INDEX_EVICTED_COUNT
	// size of blob index evicted from base DB by BlobDB compaction filter
	// because of corresponding file deleted. Only applicable to legacy BlobDB.
	TickerType_BLOB_DB_BLOB_INDEX_EVICTED_SIZE
	// # of blob files that were obsoleted by garbage collection. Only applicable
	// to legacy BlobDB.
	TickerType_BLOB_DB_GC_NUM_FILES
	// # of blob files generated by garbage collection. Only applicable to legacy
	// BlobDB.
	TickerType_BLOB_DB_GC_NUM_NEW_FILES
	// # of BlobDB garbage collection failures. Only applicable to legacy BlobDB.
	TickerType_BLOB_DB_GC_FAILURES
	// # of keys relocated to new blob file by garbage collection.
	TickerType_BLOB_DB_GC_NUM_KEYS_RELOCATED
	// # of bytes relocated to new blob file by garbage collection.
	TickerType_BLOB_DB_GC_BYTES_RELOCATED
	// # of blob files evicted because of BlobDB is full. Only applicable to
	// legacy BlobDB.
	TickerType_BLOB_DB_FIFO_NUM_FILES_EVICTED
	// # of keys in the blob files evicted because of BlobDB is full. Only
	// applicable to legacy BlobDB.
	TickerType_BLOB_DB_FIFO_NUM_KEYS_EVICTED
	// # of bytes in the blob files evicted because of BlobDB is full. Only
	// applicable to legacy BlobDB.
	TickerType_BLOB_DB_FIFO_BYTES_EVICTED

	// These counters indicate a performance issue in WritePrepared transactions.
	// We should not seem them ticking them much.
	// # of times prepare_mutex_ is acquired in the fast path.
	TickerType_TXN_PREPARE_MUTEX_OVERHEAD
	// # of times old_commit_map_mutex_ is acquired in the fast path.
	TickerType_TXN_OLD_COMMIT_MAP_MUTEX_OVERHEAD
	// # of times we checked a batch for duplicate keys.
	TickerType_TXN_DUPLICATE_KEY_OVERHEAD
	// # of times snapshot_mutex_ is acquired in the fast path.
	TickerType_TXN_SNAPSHOT_MUTEX_OVERHEAD
	// # of times ::Get returned TryAgain due to expired snapshot seq
	TickerType_TXN_GET_TRY_AGAIN

	// Number of keys actually found in MultiGet calls (vs number requested by
	// caller)
	// NUMBER_MULTIGET_KEYS_READ gives the number requested by caller
	TickerType_NUMBER_MULTIGET_KEYS_FOUND

	TickerType_NO_ITERATOR_CREATED // number of iterators created
	TickerType_NO_ITERATOR_DELETED // number of iterators deleted

	TickerType_BLOCK_CACHE_COMPRESSION_DICT_MISS
	TickerType_BLOCK_CACHE_COMPRESSION_DICT_HIT
	TickerType_BLOCK_CACHE_COMPRESSION_DICT_ADD
	TickerType_BLOCK_CACHE_COMPRESSION_DICT_BYTES_INSERT

	// # of blocks redundantly inserted into block cache.
	// REQUIRES: BLOCK_CACHE_ADD_REDUNDANT <= BLOCK_CACHE_ADD
	TickerType_BLOCK_CACHE_ADD_REDUNDANT
	// # of index blocks redundantly inserted into block cache.
	// REQUIRES: BLOCK_CACHE_INDEX_ADD_REDUNDANT <= BLOCK_CACHE_INDEX_ADD
	TickerType_BLOCK_CACHE_INDEX_ADD_REDUNDANT
	// # of filter blocks redundantly inserted into block cache.
	// REQUIRES: BLOCK_CACHE_FILTER_ADD_REDUNDANT <= BLOCK_CACHE_FILTER_ADD
	TickerType_BLOCK_CACHE_FILTER_ADD_REDUNDANT
	// # of data blocks redundantly inserted into block cache.
	// REQUIRES: BLOCK_CACHE_DATA_ADD_REDUNDANT <= BLOCK_CACHE_DATA_ADD
	TickerType_BLOCK_CACHE_DATA_ADD_REDUNDANT
	// # of dict blocks redundantly inserted into block cache.
	// REQUIRES: BLOCK_CACHE_COMPRESSION_DICT_ADD_REDUNDANT
	//           <= BLOCK_CACHE_COMPRESSION_DICT_ADD
	TickerType_BLOCK_CACHE_COMPRESSION_DICT_ADD_REDUNDANT

	// # of files marked as trash by sst file manager and will be deleted
	// later by background thread.
	TickerType_FILES_MARKED_TRASH
	// # of trash files deleted by the background thread from the trash queue.
	TickerType_FILES_DELETED_FROM_TRASH_QUEUE
	// # of files deleted immediately by sst file manager through delete
	// scheduler.
	TickerType_FILES_DELETED_IMMEDIATELY

	// The counters for error handler not that bg_io_error is the subset of
	// bg_error and bg_retryable_io_error is the subset of bg_io_error.
	// The misspelled versions are deprecated and only kept for compatibility.
	// TODO: remove the misspelled tickers in the next major release.
	TickerType_ERROR_HANDLER_BG_ERROR_COUNT
	TickerType_ERROR_HANDLER_BG_ERROR_COUNT_MISSPELLED
	TickerType_ERROR_HANDLER_BG_IO_ERROR_COUNT
	TickerType_ERROR_HANDLER_BG_IO_ERROR_COUNT_MISSPELLED
	TickerType_ERROR_HANDLER_BG_RETRYABLE_IO_ERROR_COUNT
	TickerType_ERROR_HANDLER_BG_RETRYABLE_IO_ERROR_COUNT_MISSPELLED
	TickerType_ERROR_HANDLER_AUTORESUME_COUNT
	TickerType_ERROR_HANDLER_AUTORESUME_RETRY_TOTAL_COUNT
	TickerType_ERROR_HANDLER_AUTORESUME_SUCCESS_COUNT

	// Statistics for memtable garbage collection:
	// Raw bytes of data (payload) present on memtable at flush time.
	TickerType_MEMTABLE_PAYLOAD_BYTES_AT_FLUSH
	// Outdated bytes of data present on memtable at flush time.
	TickerType_MEMTABLE_GARBAGE_BYTES_AT_FLUSH

	// Secondary cache statistics
	TickerType_SECONDARY_CACHE_HITS

	// Bytes read by `VerifyChecksum()` and `VerifyFileChecksums()` APIs.
	TickerType_VERIFY_CHECKSUM_READ_BYTES

	// Bytes read/written while creating backups
	TickerType_BACKUP_READ_BYTES
	TickerType_BACKUP_WRITE_BYTES

	// Remote compaction read/write statistics
	TickerType_REMOTE_COMPACT_READ_BYTES
	TickerType_REMOTE_COMPACT_WRITE_BYTES

	// Tiered storage related statistics
	TickerType_HOT_FILE_READ_BYTES
	TickerType_WARM_FILE_READ_BYTES
	TickerType_COLD_FILE_READ_BYTES
	TickerType_HOT_FILE_READ_COUNT
	TickerType_WARM_FILE_READ_COUNT
	TickerType_COLD_FILE_READ_COUNT

	// Last level and non-last level read statistics
	TickerType_LAST_LEVEL_READ_BYTES
	TickerType_LAST_LEVEL_READ_COUNT
	TickerType_NON_LAST_LEVEL_READ_BYTES
	TickerType_NON_LAST_LEVEL_READ_COUNT

	// Statistics on iterator Seek() (and variants) for each sorted run. I.e. a
	// single user Seek() can result in many sorted run Seek()s.
	// The stats are split between last level and non-last level.
	// Filtered: a filter such as prefix Bloom filter indicate the Seek() would
	// not find anything relevant so avoided a likely access to data+index
	// blocks.
	TickerType_LAST_LEVEL_SEEK_FILTERED
	// Filter match: a filter such as prefix Bloom filter was queried but did
	// not filter out the seek.
	TickerType_LAST_LEVEL_SEEK_FILTER_MATCH
	// At least one data block was accessed for a Seek() (or variant) on a
	// sorted run.
	TickerType_LAST_LEVEL_SEEK_DATA
	// At least one value() was accessed for the seek (suggesting it was useful)
	// and no filter such as prefix Bloom was queried.
	TickerType_LAST_LEVEL_SEEK_DATA_USEFUL_NO_FILTER
	// At least one value() was accessed for the seek (suggesting it was useful)
	// after querying a filter such as prefix Bloom.
	TickerType_LAST_LEVEL_SEEK_DATA_USEFUL_FILTER_MATCH
	// The same set of stats but for non-last level seeks.
	TickerType_NON_LAST_LEVEL_SEEK_FILTERED
	TickerType_NON_LAST_LEVEL_SEEK_FILTER_MATCH
	TickerType_NON_LAST_LEVEL_SEEK_DATA
	TickerType_NON_LAST_LEVEL_SEEK_DATA_USEFUL_NO_FILTER
	TickerType_NON_LAST_LEVEL_SEEK_DATA_USEFUL_FILTER_MATCH

	// Number of block checksum verifications
	TickerType_BLOCK_CHECKSUM_COMPUTE_COUNT
	// Number of times RocksDB detected a corruption while verifying a block
	// checksum. RocksDB does not remember corruptions that happened during user
	// reads so the same block corruption may be detected multiple times.
	TickerType_BLOCK_CHECKSUM_MISMATCH_COUNT

	TickerType_MULTIGET_COROUTINE_COUNT

	// Integrated BlobDB specific stats
	// # of times cache miss when accessing blob from blob cache.
	TickerType_BLOB_DB_CACHE_MISS
	// # of times cache hit when accessing blob from blob cache.
	TickerType_BLOB_DB_CACHE_HIT
	// # of data blocks added to blob cache.
	TickerType_BLOB_DB_CACHE_ADD
	// # of failures when adding blobs to blob cache.
	TickerType_BLOB_DB_CACHE_ADD_FAILURES
	// # of bytes read from blob cache.
	TickerType_BLOB_DB_CACHE_BYTES_READ
	// # of bytes written into blob cache.
	TickerType_BLOB_DB_CACHE_BYTES_WRITE

	// Time spent in the ReadAsync file system call
	TickerType_READ_ASYNC_MICROS
	// Number of errors returned to the async read callback
	TickerType_ASYNC_READ_ERROR_COUNT

	// Fine grained secondary cache stats
	TickerType_SECONDARY_CACHE_FILTER_HITS
	TickerType_SECONDARY_CACHE_INDEX_HITS
	TickerType_SECONDARY_CACHE_DATA_HITS

	// Number of lookup into the prefetched tail (see
	// `TABLE_OPEN_PREFETCH_TAIL_READ_BYTES`)
	// that can't find its data for table open
	TickerType_TABLE_OPEN_PREFETCH_TAIL_MISS
	// Number of lookup into the prefetched tail (see
	// `TABLE_OPEN_PREFETCH_TAIL_READ_BYTES`)
	// that finds its data for table open
	TickerType_TABLE_OPEN_PREFETCH_TAIL_HIT

	// Statistics on the filtering by user-defined timestamps
	// # of times timestamps are checked on accessing the table
	TickerType_TIMESTAMP_FILTER_TABLE_CHECKED
	// # of times timestamps can successfully help skip the table access
	TickerType_TIMESTAMP_FILTER_TABLE_FILTERED

	// Number of input bytes (uncompressed) to compression for SST blocks that
	// are stored compressed.
	TickerType_BYTES_COMPRESSED_FROM
	// Number of output bytes (compressed) from compression for SST blocks that
	// are stored compressed.
	TickerType_BYTES_COMPRESSED_TO
	// Number of uncompressed bytes for SST blocks that are stored uncompressed
	// because compression type is kNoCompression or some error case caused
	// compression not to run or produce an output. Index blocks are only counted
	// if enable_index_compression is true.
	TickerType_BYTES_COMPRESSION_BYPASSED
	// Number of input bytes (uncompressed) to compression for SST blocks that
	// are stored uncompressed because the compression result was rejected
	// either because the ratio was not acceptable (see
	// CompressionOptions::max_compressed_bytes_per_kb) or found invalid by the
	// `verify_compression` option.
	TickerType_BYTES_COMPRESSION_REJECTED

	// Like BYTES_COMPRESSION_BYPASSED but counting number of blocks
	TickerType_NUMBER_BLOCK_COMPRESSION_BYPASSED
	// Like BYTES_COMPRESSION_REJECTED but counting number of blocks
	TickerType_NUMBER_BLOCK_COMPRESSION_REJECTED

	// Number of input bytes (compressed) to decompression in reading compressed
	// SST blocks from storage.
	TickerType_BYTES_DECOMPRESSED_FROM
	// Number of output bytes (uncompressed) from decompression in reading
	// compressed SST blocks from storage.
	TickerType_BYTES_DECOMPRESSED_TO

	// Number of times readahead is trimmed during scans when
	// ReadOptions.auto_readahead_size is set.
	TickerType_READAHEAD_TRIMMED
)

type HistogramType uint32

const (
	HistogramType_DB_GET HistogramType = iota
	HistogramType_DB_WRITE
	HistogramType_COMPACTION_TIME
	HistogramType_COMPACTION_CPU_TIME
	HistogramType_SUBCOMPACTION_SETUP_TIME
	HistogramType_TABLE_SYNC_MICROS
	HistogramType_COMPACTION_OUTFILE_SYNC_MICROS
	HistogramType_WAL_FILE_SYNC_MICROS
	HistogramType_MANIFEST_FILE_SYNC_MICROS
	// TIME SPENT IN IO DURING TABLE OPEN
	HistogramType_TABLE_OPEN_IO_MICROS
	HistogramType_DB_MULTIGET
	HistogramType_READ_BLOCK_COMPACTION_MICROS
	HistogramType_READ_BLOCK_GET_MICROS
	HistogramType_WRITE_RAW_BLOCK_MICROS
	HistogramType_NUM_FILES_IN_SINGLE_COMPACTION
	HistogramType_DB_SEEK
	HistogramType_WRITE_STALL
	// Time spent in reading block-based or plain SST table
	HistogramType_SST_READ_MICROS
	// Time spent in reading SST table (currently only block-based table) or blob
	// file corresponding to `Env::IOActivity`
	HistogramType_FILE_READ_FLUSH_MICROS
	HistogramType_FILE_READ_COMPACTION_MICROS
	HistogramType_FILE_READ_DB_OPEN_MICROS
	// The following `FILE_READ_*` require stats level greater than
	// `StatsLevel::kExceptDetailedTimers`
	HistogramType_FILE_READ_GET_MICROS
	HistogramType_FILE_READ_MULTIGET_MICROS
	HistogramType_FILE_READ_DB_ITERATOR_MICROS
	HistogramType_FILE_READ_VERIFY_DB_CHECKSUM_MICROS
	HistogramType_FILE_READ_VERIFY_FILE_CHECKSUMS_MICROS

	// The number of subcompactions actually scheduled during a compaction
	HistogramType_NUM_SUBCOMPACTIONS_SCHEDULED
	// Value size distribution in each operation
	HistogramType_BYTES_PER_READ
	HistogramType_BYTES_PER_WRITE
	HistogramType_BYTES_PER_MULTIGET

	HistogramType_BYTES_COMPRESSED   // DEPRECATED / unused (see BYTES_COMPRESSED_{FROMTO})
	HistogramType_BYTES_DECOMPRESSED // DEPRECATED / unused (see BYTES_DECOMPRESSED_{FROMTO})
	HistogramType_COMPRESSION_TIMES_NANOS
	HistogramType_DECOMPRESSION_TIMES_NANOS
	// Number of merge operands passed to the merge operator in user read
	// requests.
	HistogramType_READ_NUM_MERGE_OPERANDS

	// BlobDB specific stats
	// Size of keys written to BlobDB. Only applicable to legacy BlobDB.
	HistogramType_BLOB_DB_KEY_SIZE
	// Size of values written to BlobDB. Only applicable to legacy BlobDB.
	HistogramType_BLOB_DB_VALUE_SIZE
	// BlobDB Put/PutWithTTL/PutUntil/Write latency. Only applicable to legacy
	// BlobDB.
	HistogramType_BLOB_DB_WRITE_MICROS
	// BlobDB Get latency. Only applicable to legacy BlobDB.
	HistogramType_BLOB_DB_GET_MICROS
	// BlobDB MultiGet latency. Only applicable to legacy BlobDB.
	HistogramType_BLOB_DB_MULTIGET_MICROS
	// BlobDB Seek/SeekToFirst/SeekToLast/SeekForPrev latency. Only applicable to
	// legacy BlobDB.
	HistogramType_BLOB_DB_SEEK_MICROS
	// BlobDB Next latency. Only applicable to legacy BlobDB.
	HistogramType_BLOB_DB_NEXT_MICROS
	// BlobDB Prev latency. Only applicable to legacy BlobDB.
	HistogramType_BLOB_DB_PREV_MICROS
	// Blob file write latency.
	HistogramType_BLOB_DB_BLOB_FILE_WRITE_MICROS
	// Blob file read latency.
	HistogramType_BLOB_DB_BLOB_FILE_READ_MICROS
	// Blob file sync latency.
	HistogramType_BLOB_DB_BLOB_FILE_SYNC_MICROS
	// BlobDB compression time.
	HistogramType_BLOB_DB_COMPRESSION_MICROS
	// BlobDB decompression time.
	HistogramType_BLOB_DB_DECOMPRESSION_MICROS
	// Time spent flushing memtable to disk
	HistogramType_FLUSH_TIME
	HistogramType_SST_BATCH_SIZE

	// MultiGet stats logged per level
	// Num of index and filter blocks read from file system per level.
	HistogramType_NUM_INDEX_AND_FILTER_BLOCKS_READ_PER_LEVEL
	// Num of sst files read from file system per level.
	HistogramType_NUM_SST_READ_PER_LEVEL

	// Error handler statistics
	HistogramType_ERROR_HANDLER_AUTORESUME_RETRY_COUNT

	// Stats related to asynchronous read requests.
	HistogramType_ASYNC_READ_BYTES
	HistogramType_POLL_WAIT_MICROS

	// Number of prefetched bytes discarded by RocksDB.
	HistogramType_PREFETCHED_BYTES_DISCARDED

	// Number of IOs issued in parallel in a MultiGet batch
	HistogramType_MULTIGET_IO_BATCH_SIZE

	// Number of levels requiring IO for MultiGet
	HistogramType_NUM_LEVEL_READ_PER_MULTIGET

	// Wait time for aborting async read in FilePrefetchBuffer destructor
	HistogramType_ASYNC_PREFETCH_ABORT_MICROS

	// Number of bytes read for RocksDB's prefetching contents (as opposed to file
	// system's prefetch) from the end of SST table during block based table open
	HistogramType_TABLE_OPEN_PREFETCH_TAIL_READ_BYTES
)

// HistogramData histogram metrics.
type HistogramData struct {
	Median  float64
	P95     float64
	P99     float64
	Average float64
	StdDev  float64
	Max     float64
	Min     float64
	Count   uint64
	Sum     uint64
}
