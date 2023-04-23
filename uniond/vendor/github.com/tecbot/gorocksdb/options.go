package gorocksdb

// #include "rocksdb/c.h"
// #include "gorocksdb.h"
import "C"
import (
	"errors"
	"unsafe"
)

// CompressionType specifies the block compression.
// DB contents are stored in a set of blocks, each of which holds a
// sequence of key,value pairs. Each block may be compressed before
// being stored in a file. The following enum describes which
// compression method (if any) is used to compress a block.
type CompressionType uint

// Compression types.
const (
	NoCompression     = CompressionType(C.rocksdb_no_compression)
	SnappyCompression = CompressionType(C.rocksdb_snappy_compression)
	ZLibCompression   = CompressionType(C.rocksdb_zlib_compression)
	Bz2Compression    = CompressionType(C.rocksdb_bz2_compression)
	LZ4Compression    = CompressionType(C.rocksdb_lz4_compression)
	LZ4HCCompression  = CompressionType(C.rocksdb_lz4hc_compression)
	XpressCompression = CompressionType(C.rocksdb_xpress_compression)
	ZSTDCompression   = CompressionType(C.rocksdb_zstd_compression)
)

// CompactionStyle specifies the compaction style.
type CompactionStyle uint

// Compaction styles.
const (
	LevelCompactionStyle     = CompactionStyle(C.rocksdb_level_compaction)
	UniversalCompactionStyle = CompactionStyle(C.rocksdb_universal_compaction)
	FIFOCompactionStyle      = CompactionStyle(C.rocksdb_fifo_compaction)
)

// CompactionAccessPattern specifies the access patern in compaction.
type CompactionAccessPattern uint

// Access patterns for compaction.
const (
	NoneCompactionAccessPattern       = CompactionAccessPattern(0)
	NormalCompactionAccessPattern     = CompactionAccessPattern(1)
	SequentialCompactionAccessPattern = CompactionAccessPattern(2)
	WillneedCompactionAccessPattern   = CompactionAccessPattern(3)
)

// InfoLogLevel describes the log level.
type InfoLogLevel uint

// Log leves.
const (
	DebugInfoLogLevel = InfoLogLevel(0)
	InfoInfoLogLevel  = InfoLogLevel(1)
	WarnInfoLogLevel  = InfoLogLevel(2)
	ErrorInfoLogLevel = InfoLogLevel(3)
	FatalInfoLogLevel = InfoLogLevel(4)
)

type WALRecoveryMode int

const (
	TolerateCorruptedTailRecordsRecovery = WALRecoveryMode(0)
	AbsoluteConsistencyRecovery          = WALRecoveryMode(1)
	PointInTimeRecovery                  = WALRecoveryMode(2)
	SkipAnyCorruptedRecordsRecovery      = WALRecoveryMode(3)
)

// Options represent all of the available options when opening a database with Open.
type Options struct {
	c *C.rocksdb_options_t

	// Hold references for GC.
	env  *Env
	bbto *BlockBasedTableOptions

	// We keep these so we can free their memory in Destroy.
	ccmp *C.rocksdb_comparator_t
	cmo  *C.rocksdb_mergeoperator_t
	cst  *C.rocksdb_slicetransform_t
	ccf  *C.rocksdb_compactionfilter_t
}

// NewDefaultOptions creates the default Options.
func NewDefaultOptions() *Options {
	return NewNativeOptions(C.rocksdb_options_create())
}

// NewNativeOptions creates a Options object.
func NewNativeOptions(c *C.rocksdb_options_t) *Options {
	return &Options{c: c}
}

// GetOptionsFromString creates a Options object from existing opt and string.
// If base is nil, a default opt create by NewDefaultOptions will be used as base opt.
func GetOptionsFromString(base *Options, optStr string) (*Options, error) {
	if base == nil {
		base = NewDefaultOptions()
		defer base.Destroy()
	}

	var (
		cErr    *C.char
		cOptStr = C.CString(optStr)
	)
	defer C.free(unsafe.Pointer(cOptStr))

	newOpt := NewDefaultOptions()
	C.rocksdb_get_options_from_string(base.c, cOptStr, newOpt.c, &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return nil, errors.New(C.GoString(cErr))
	}

	return newOpt, nil
}

// -------------------
// Parameters that affect behavior

// SetCompactionFilter sets the specified compaction filter
// which will be applied on compactions.
// Default: nil
func (opts *Options) SetCompactionFilter(value CompactionFilter) {
	if nc, ok := value.(nativeCompactionFilter); ok {
		opts.ccf = nc.c
	} else {
		idx := registerCompactionFilter(value)
		opts.ccf = C.gorocksdb_compactionfilter_create(C.uintptr_t(idx))
	}
	C.rocksdb_options_set_compaction_filter(opts.c, opts.ccf)
}

// SetComparator sets the comparator which define the order of keys in the table.
// Default: a comparator that uses lexicographic byte-wise ordering
func (opts *Options) SetComparator(value Comparator) {
	if nc, ok := value.(nativeComparator); ok {
		opts.ccmp = nc.c
	} else {
		idx := registerComperator(value)
		opts.ccmp = C.gorocksdb_comparator_create(C.uintptr_t(idx))
	}
	C.rocksdb_options_set_comparator(opts.c, opts.ccmp)
}

// SetMergeOperator sets the merge operator which will be called
// if a merge operations are used.
// Default: nil
func (opts *Options) SetMergeOperator(value MergeOperator) {
	if nmo, ok := value.(nativeMergeOperator); ok {
		opts.cmo = nmo.c
	} else {
		idx := registerMergeOperator(value)
		opts.cmo = C.gorocksdb_mergeoperator_create(C.uintptr_t(idx))
	}
	C.rocksdb_options_set_merge_operator(opts.c, opts.cmo)
}

// A single CompactionFilter instance to call into during compaction.
// Allows an application to modify/delete a key-value during background
// compaction.
//
// If the client requires a new compaction filter to be used for different
// compaction runs, it can specify compaction_filter_factory instead of this
// option. The client should specify only one of the two.
// compaction_filter takes precedence over compaction_filter_factory if
// client specifies both.
//
// If multithreaded compaction is being used, the supplied CompactionFilter
// instance may be used from different threads concurrently and so should be
// thread-safe.
//
// Default: nil
// TODO: implement in C
//func (opts *Options) SetCompactionFilter(value *CompactionFilter) {
//	C.rocksdb_options_set_compaction_filter(opts.c, value.filter)
//}

// This is a factory that provides compaction filter objects which allow
// an application to modify/delete a key-value during background compaction.
//
// A new filter will be created on each compaction run.  If multithreaded
// compaction is being used, each created CompactionFilter will only be used
// from a single thread and so does not need to be thread-safe.
//
// Default: a factory that doesn't provide any object
// std::shared_ptr<CompactionFilterFactory> compaction_filter_factory;
// TODO: implement in C and Go

// Version TWO of the compaction_filter_factory
// It supports rolling compaction
//
// Default: a factory that doesn't provide any object
// std::shared_ptr<CompactionFilterFactoryV2> compaction_filter_factory_v2;
// TODO: implement in C and Go

// SetCreateIfMissing specifies whether the database
// should be created if it is missing.
// Default: false
func (opts *Options) SetCreateIfMissing(value bool) {
	C.rocksdb_options_set_create_if_missing(opts.c, boolToChar(value))
}

// SetErrorIfExists specifies whether an error should be raised
// if the database already exists.
// Default: false
func (opts *Options) SetErrorIfExists(value bool) {
	C.rocksdb_options_set_error_if_exists(opts.c, boolToChar(value))
}

// SetParanoidChecks enable/disable paranoid checks.
//
// If true, the implementation will do aggressive checking of the
// data it is processing and will stop early if it detects any
// errors. This may have unforeseen ramifications: for example, a
// corruption of one DB entry may cause a large number of entries to
// become unreadable or for the entire DB to become unopenable.
// If any of the  writes to the database fails (Put, Delete, Merge, Write),
// the database will switch to read-only mode and fail all other
// Write operations.
// Default: false
func (opts *Options) SetParanoidChecks(value bool) {
	C.rocksdb_options_set_paranoid_checks(opts.c, boolToChar(value))
}

// SetDBPaths sets the DBPaths of the options.
//
// A list of paths where SST files can be put into, with its target size.
// Newer data is placed into paths specified earlier in the vector while
// older data gradually moves to paths specified later in the vector.
//
// For example, you have a flash device with 10GB allocated for the DB,
// as well as a hard drive of 2TB, you should config it to be:
//   [{"/flash_path", 10GB}, {"/hard_drive", 2TB}]
//
// The system will try to guarantee data under each path is close to but
// not larger than the target size. But current and future file sizes used
// by determining where to place a file are based on best-effort estimation,
// which means there is a chance that the actual size under the directory
// is slightly more than target size under some workloads. User should give
// some buffer room for those cases.
//
// If none of the paths has sufficient room to place a file, the file will
// be placed to the last path anyway, despite to the target size.
//
// Placing newer data to earlier paths is also best-efforts. User should
// expect user files to be placed in higher levels in some extreme cases.
//
// If left empty, only one path will be used, which is db_name passed when
// opening the DB.
// Default: empty
func (opts *Options) SetDBPaths(dbpaths []*DBPath) {
	l := len(dbpaths)
	cDbpaths := make([]*C.rocksdb_dbpath_t, l)
	for i, v := range dbpaths {
		cDbpaths[i] = v.c
	}

	C.rocksdb_options_set_db_paths(opts.c, &cDbpaths[0], C.size_t(l))
}

// SetEnv sets the specified object to interact with the environment,
// e.g. to read/write files, schedule background work, etc.
// Default: DefaultEnv
func (opts *Options) SetEnv(value *Env) {
	opts.env = value

	C.rocksdb_options_set_env(opts.c, value.c)
}

// SetInfoLogLevel sets the info log level.
// Default: InfoInfoLogLevel
func (opts *Options) SetInfoLogLevel(value InfoLogLevel) {
	C.rocksdb_options_set_info_log_level(opts.c, C.int(value))
}

// IncreaseParallelism sets the parallelism.
//
// By default, RocksDB uses only one background thread for flush and
// compaction. Calling this function will set it up such that total of
// `total_threads` is used. Good value for `total_threads` is the number of
// cores. You almost definitely want to call this function if your system is
// bottlenecked by RocksDB.
func (opts *Options) IncreaseParallelism(total_threads int) {
	C.rocksdb_options_increase_parallelism(opts.c, C.int(total_threads))
}

// OptimizeForPointLookup optimize the DB for point lookups.
//
// Use this if you don't need to keep the data sorted, i.e. you'll never use
// an iterator, only Put() and Get() API calls
//
// If you use this with rocksdb >= 5.0.2, you must call `SetAllowConcurrentMemtableWrites(false)`
// to avoid an assertion error immediately on opening the db.
func (opts *Options) OptimizeForPointLookup(block_cache_size_mb uint64) {
	C.rocksdb_options_optimize_for_point_lookup(opts.c, C.uint64_t(block_cache_size_mb))
}

// Set whether to allow concurrent memtable writes. Conccurent writes are
// not supported by all memtable factories (currently only SkipList memtables).
// As of rocksdb 5.0.2 you must call `SetAllowConcurrentMemtableWrites(false)`
// if you use `OptimizeForPointLookup`.
func (opts *Options) SetAllowConcurrentMemtableWrites(allow bool) {
	C.rocksdb_options_set_allow_concurrent_memtable_write(opts.c, boolToChar(allow))
}

// OptimizeLevelStyleCompaction optimize the DB for leveld compaction.
//
// Default values for some parameters in ColumnFamilyOptions are not
// optimized for heavy workloads and big datasets, which means you might
// observe write stalls under some conditions. As a starting point for tuning
// RocksDB options, use the following two functions:
// * OptimizeLevelStyleCompaction -- optimizes level style compaction
// * OptimizeUniversalStyleCompaction -- optimizes universal style compaction
// Universal style compaction is focused on reducing Write Amplification
// Factor for big data sets, but increases Space Amplification. You can learn
// more about the different styles here:
// https://github.com/facebook/rocksdb/wiki/Rocksdb-Architecture-Guide
// Make sure to also call IncreaseParallelism(), which will provide the
// biggest performance gains.
// Note: we might use more memory than memtable_memory_budget during high
// write rate period
func (opts *Options) OptimizeLevelStyleCompaction(memtable_memory_budget uint64) {
	C.rocksdb_options_optimize_level_style_compaction(opts.c, C.uint64_t(memtable_memory_budget))
}

// OptimizeUniversalStyleCompaction optimize the DB for universal compaction.
// See note on OptimizeLevelStyleCompaction.
func (opts *Options) OptimizeUniversalStyleCompaction(memtable_memory_budget uint64) {
	C.rocksdb_options_optimize_universal_style_compaction(opts.c, C.uint64_t(memtable_memory_budget))
}

// SetWriteBufferSize sets the amount of data to build up in memory
// (backed by an unsorted log on disk) before converting to a sorted on-disk file.
//
// Larger values increase performance, especially during bulk loads.
// Up to max_write_buffer_number write buffers may be held in memory
// at the same time,
// so you may wish to adjust this parameter to control memory usage.
// Also, a larger write buffer will result in a longer recovery time
// the next time the database is opened.
// Default: 64MB
func (opts *Options) SetWriteBufferSize(value int) {
	C.rocksdb_options_set_write_buffer_size(opts.c, C.size_t(value))
}

// SetMaxWriteBufferNumber sets the maximum number of write buffers
// that are built up in memory.
//
// The default is 2, so that when 1 write buffer is being flushed to
// storage, new writes can continue to the other write buffer.
// Default: 2
func (opts *Options) SetMaxWriteBufferNumber(value int) {
	C.rocksdb_options_set_max_write_buffer_number(opts.c, C.int(value))
}

// SetMinWriteBufferNumberToMerge sets the minimum number of write buffers
// that will be merged together before writing to storage.
//
// If set to 1, then all write buffers are flushed to L0 as individual files
// and this increases read amplification because a get request has to check
// in all of these files. Also, an in-memory merge may result in writing lesser
// data to storage if there are duplicate records in each of these
// individual write buffers.
// Default: 1
func (opts *Options) SetMinWriteBufferNumberToMerge(value int) {
	C.rocksdb_options_set_min_write_buffer_number_to_merge(opts.c, C.int(value))
}

// SetMaxOpenFiles sets the number of open files that can be used by the DB.
//
// You may need to increase this if your database has a large working set
// (budget one open file per 2MB of working set).
// Default: 1000
func (opts *Options) SetMaxOpenFiles(value int) {
	C.rocksdb_options_set_max_open_files(opts.c, C.int(value))
}

// SetMaxFileOpeningThreads sets the maximum number of file opening threads.
// If max_open_files is -1, DB will open all files on DB::Open(). You can
// use this option to increase the number of threads used to open the files.
// Default: 16
func (opts *Options) SetMaxFileOpeningThreads(value int) {
	C.rocksdb_options_set_max_file_opening_threads(opts.c, C.int(value))
}

// SetMaxTotalWalSize sets the maximum total wal size in bytes.
// Once write-ahead logs exceed this size, we will start forcing the flush of
// column families whose memtables are backed by the oldest live WAL file
// (i.e. the ones that are causing all the space amplification). If set to 0
// (default), we will dynamically choose the WAL size limit to be
// [sum of all write_buffer_size * max_write_buffer_number] * 4
// Default: 0
func (opts *Options) SetMaxTotalWalSize(value uint64) {
	C.rocksdb_options_set_max_total_wal_size(opts.c, C.uint64_t(value))
}

// SetCompression sets the compression algorithm.
// Default: SnappyCompression, which gives lightweight but fast
// compression.
func (opts *Options) SetCompression(value CompressionType) {
	C.rocksdb_options_set_compression(opts.c, C.int(value))
}

// SetCompressionPerLevel sets different compression algorithm per level.
//
// Different levels can have different compression policies. There
// are cases where most lower levels would like to quick compression
// algorithm while the higher levels (which have more data) use
// compression algorithms that have better compression but could
// be slower. This array should have an entry for
// each level of the database. This array overrides the
// value specified in the previous field 'compression'.
func (opts *Options) SetCompressionPerLevel(value []CompressionType) {
	cLevels := make([]C.int, len(value))
	for i, v := range value {
		cLevels[i] = C.int(v)
	}

	C.rocksdb_options_set_compression_per_level(opts.c, &cLevels[0], C.size_t(len(value)))
}

// SetMinLevelToCompress sets the start level to use compression.
func (opts *Options) SetMinLevelToCompress(value int) {
	C.rocksdb_options_set_min_level_to_compress(opts.c, C.int(value))
}

// SetCompressionOptions sets different options for compression algorithms.
// Default: nil
func (opts *Options) SetCompressionOptions(value *CompressionOptions) {
	C.rocksdb_options_set_compression_options(opts.c, C.int(value.WindowBits), C.int(value.Level), C.int(value.Strategy), C.int(value.MaxDictBytes))
}

// SetPrefixExtractor sets the prefic extractor.
//
// If set, use the specified function to determine the
// prefixes for keys. These prefixes will be placed in the filter.
// Depending on the workload, this can reduce the number of read-IOP
// cost for scans when a prefix is passed via ReadOptions to
// db.NewIterator().
// Default: nil
func (opts *Options) SetPrefixExtractor(value SliceTransform) {
	if nst, ok := value.(nativeSliceTransform); ok {
		opts.cst = nst.c
	} else {
		idx := registerSliceTransform(value)
		opts.cst = C.gorocksdb_slicetransform_create(C.uintptr_t(idx))
	}
	C.rocksdb_options_set_prefix_extractor(opts.c, opts.cst)
}

// SetNumLevels sets the number of levels for this database.
// Default: 7
func (opts *Options) SetNumLevels(value int) {
	C.rocksdb_options_set_num_levels(opts.c, C.int(value))
}

// SetLevel0FileNumCompactionTrigger sets the number of files
// to trigger level-0 compaction.
//
// A value <0 means that level-0 compaction will not be
// triggered by number of files at all.
// Default: 4
func (opts *Options) SetLevel0FileNumCompactionTrigger(value int) {
	C.rocksdb_options_set_level0_file_num_compaction_trigger(opts.c, C.int(value))
}

// SetLevel0SlowdownWritesTrigger sets the soft limit on number of level-0 files.
//
// We start slowing down writes at this point.
// A value <0 means that no writing slow down will be triggered by
// number of files in level-0.
// Default: 8
func (opts *Options) SetLevel0SlowdownWritesTrigger(value int) {
	C.rocksdb_options_set_level0_slowdown_writes_trigger(opts.c, C.int(value))
}

// SetLevel0StopWritesTrigger sets the maximum number of level-0 files.
// We stop writes at this point.
// Default: 12
func (opts *Options) SetLevel0StopWritesTrigger(value int) {
	C.rocksdb_options_set_level0_stop_writes_trigger(opts.c, C.int(value))
}

// SetMaxMemCompactionLevel sets the maximum level
// to which a new compacted memtable is pushed if it does not create overlap.
//
// We try to push to level 2 to avoid the
// relatively expensive level 0=>1 compactions and to avoid some
// expensive manifest file operations. We do not push all the way to
// the largest level since that can generate a lot of wasted disk
// space if the same key space is being repeatedly overwritten.
// Default: 2
func (opts *Options) SetMaxMemCompactionLevel(value int) {
	C.rocksdb_options_set_max_mem_compaction_level(opts.c, C.int(value))
}

// SetTargetFileSizeBase sets the target file size for compaction.
//
// Target file size is per-file size for level-1.
// Target file size for level L can be calculated by
// target_file_size_base * (target_file_size_multiplier ^ (L-1))
//
// For example, if target_file_size_base is 2MB and
// target_file_size_multiplier is 10, then each file on level-1 will
// be 2MB, and each file on level 2 will be 20MB,
// and each file on level-3 will be 200MB.
// Default: 2MB
func (opts *Options) SetTargetFileSizeBase(value uint64) {
	C.rocksdb_options_set_target_file_size_base(opts.c, C.uint64_t(value))
}

// SetTargetFileSizeMultiplier sets the target file size multiplier for compaction.
// Default: 1
func (opts *Options) SetTargetFileSizeMultiplier(value int) {
	C.rocksdb_options_set_target_file_size_multiplier(opts.c, C.int(value))
}

// SetMaxBytesForLevelBase sets the maximum total data size for a level.
//
// It is the max total for level-1.
// Maximum number of bytes for level L can be calculated as
// (max_bytes_for_level_base) * (max_bytes_for_level_multiplier ^ (L-1))
//
// For example, if max_bytes_for_level_base is 20MB, and if
// max_bytes_for_level_multiplier is 10, total data size for level-1
// will be 20MB, total file size for level-2 will be 200MB,
// and total file size for level-3 will be 2GB.
// Default: 10MB
func (opts *Options) SetMaxBytesForLevelBase(value uint64) {
	C.rocksdb_options_set_max_bytes_for_level_base(opts.c, C.uint64_t(value))
}

// SetMaxBytesForLevelMultiplier sets the max Bytes for level multiplier.
// Default: 10
func (opts *Options) SetMaxBytesForLevelMultiplier(value float64) {
	C.rocksdb_options_set_max_bytes_for_level_multiplier(opts.c, C.double(value))
}

// SetLevelCompactiondynamiclevelbytes specifies whether to pick
// target size of each level dynamically.
//
// We will pick a base level b >= 1. L0 will be directly merged into level b,
// instead of always into level 1. Level 1 to b-1 need to be empty.
// We try to pick b and its target size so that
// 1. target size is in the range of
//   (max_bytes_for_level_base / max_bytes_for_level_multiplier,
//    max_bytes_for_level_base]
// 2. target size of the last level (level num_levels-1) equals to extra size
//    of the level.
// At the same time max_bytes_for_level_multiplier and
// max_bytes_for_level_multiplier_additional are still satisfied.
//
// With this option on, from an empty DB, we make last level the base level,
// which means merging L0 data into the last level, until it exceeds
// max_bytes_for_level_base. And then we make the second last level to be
// base level, to start to merge L0 data to second last level, with its
// target size to be 1/max_bytes_for_level_multiplier of the last level's
// extra size. After the data accumulates more so that we need to move the
// base level to the third last one, and so on.
//
// For example, assume max_bytes_for_level_multiplier=10, num_levels=6,
// and max_bytes_for_level_base=10MB.
// Target sizes of level 1 to 5 starts with:
// [- - - - 10MB]
// with base level is level. Target sizes of level 1 to 4 are not applicable
// because they will not be used.
// Until the size of Level 5 grows to more than 10MB, say 11MB, we make
// base target to level 4 and now the targets looks like:
// [- - - 1.1MB 11MB]
// While data are accumulated, size targets are tuned based on actual data
// of level 5. When level 5 has 50MB of data, the target is like:
// [- - - 5MB 50MB]
// Until level 5's actual size is more than 100MB, say 101MB. Now if we keep
// level 4 to be the base level, its target size needs to be 10.1MB, which
// doesn't satisfy the target size range. So now we make level 3 the target
// size and the target sizes of the levels look like:
// [- - 1.01MB 10.1MB 101MB]
// In the same way, while level 5 further grows, all levels' targets grow,
// like
// [- - 5MB 50MB 500MB]
// Until level 5 exceeds 1000MB and becomes 1001MB, we make level 2 the
// base level and make levels' target sizes like this:
// [- 1.001MB 10.01MB 100.1MB 1001MB]
// and go on...
//
// By doing it, we give max_bytes_for_level_multiplier a priority against
// max_bytes_for_level_base, for a more predictable LSM tree shape. It is
// useful to limit worse case space amplification.
//
// max_bytes_for_level_multiplier_additional is ignored with this flag on.
//
// Turning this feature on or off for an existing DB can cause unexpected
// LSM tree structure so it's not recommended.
//
// Default: false
func (opts *Options) SetLevelCompactionDynamicLevelBytes(value bool) {
	C.rocksdb_options_set_level_compaction_dynamic_level_bytes(opts.c, boolToChar(value))
}

// SetMaxCompactionBytes sets the maximum number of bytes in all compacted files.
// We try to limit number of bytes in one compaction to be lower than this
// threshold. But it's not guaranteed.
// Value 0 will be sanitized.
// Default: result.target_file_size_base * 25
func (opts *Options) SetMaxCompactionBytes(value uint64) {
	C.rocksdb_options_set_max_compaction_bytes(opts.c, C.uint64_t(value))
}

// SetSoftPendingCompactionBytesLimit sets the threshold at which
// all writes will be slowed down to at least delayed_write_rate if estimated
// bytes needed to be compaction exceed this threshold.
//
// Default: 64GB
func (opts *Options) SetSoftPendingCompactionBytesLimit(value uint64) {
	C.rocksdb_options_set_soft_pending_compaction_bytes_limit(opts.c, C.size_t(value))
}

// SetHardPendingCompactionBytesLimit sets the bytes threshold at which
// all writes are stopped if estimated bytes needed to be compaction exceed
// this threshold.
//
// Default: 256GB
func (opts *Options) SetHardPendingCompactionBytesLimit(value uint64) {
	C.rocksdb_options_set_hard_pending_compaction_bytes_limit(opts.c, C.size_t(value))
}

// SetMaxBytesForLevelMultiplierAdditional sets different max-size multipliers
// for different levels.
//
// These are multiplied by max_bytes_for_level_multiplier to arrive
// at the max-size of each level.
// Default: 1 for each level
func (opts *Options) SetMaxBytesForLevelMultiplierAdditional(value []int) {
	cLevels := make([]C.int, len(value))
	for i, v := range value {
		cLevels[i] = C.int(v)
	}

	C.rocksdb_options_set_max_bytes_for_level_multiplier_additional(opts.c, &cLevels[0], C.size_t(len(value)))
}

// SetUseFsync enable/disable fsync.
//
// If true, then every store to stable storage will issue a fsync.
// If false, then every store to stable storage will issue a fdatasync.
// This parameter should be set to true while storing data to
// filesystem like ext3 that can lose files after a reboot.
// Default: false
func (opts *Options) SetUseFsync(value bool) {
	C.rocksdb_options_set_use_fsync(opts.c, C.int(btoi(value)))
}

// SetDbLogDir specifies the absolute info LOG dir.
//
// If it is empty, the log files will be in the same dir as data.
// If it is non empty, the log files will be in the specified dir,
// and the db data dir's absolute path will be used as the log file
// name's prefix.
// Default: empty
func (opts *Options) SetDbLogDir(value string) {
	cvalue := C.CString(value)
	defer C.free(unsafe.Pointer(cvalue))
	C.rocksdb_options_set_db_log_dir(opts.c, cvalue)
}

// SetWalDir specifies the absolute dir path for write-ahead logs (WAL).
//
// If it is empty, the log files will be in the same dir as data.
// If it is non empty, the log files will be in the specified dir,
// When destroying the db, all log files and the dir itopts is deleted.
// Default: empty
func (opts *Options) SetWalDir(value string) {
	cvalue := C.CString(value)
	defer C.free(unsafe.Pointer(cvalue))
	C.rocksdb_options_set_wal_dir(opts.c, cvalue)
}

// SetDeleteObsoleteFilesPeriodMicros sets the periodicity
// when obsolete files get deleted.
//
// The files that get out of scope by compaction
// process will still get automatically delete on every compaction,
// regardless of this setting.
// Default: 6 hours
func (opts *Options) SetDeleteObsoleteFilesPeriodMicros(value uint64) {
	C.rocksdb_options_set_delete_obsolete_files_period_micros(opts.c, C.uint64_t(value))
}

// SetMaxBackgroundCompactions sets the maximum number of
// concurrent background jobs, submitted to
// the default LOW priority thread pool
// Default: 1
func (opts *Options) SetMaxBackgroundCompactions(value int) {
	C.rocksdb_options_set_max_background_compactions(opts.c, C.int(value))
}

// SetMaxBackgroundFlushes sets the maximum number of
// concurrent background memtable flush jobs, submitted to
// the HIGH priority thread pool.
//
// By default, all background jobs (major compaction and memtable flush) go
// to the LOW priority pool. If this option is set to a positive number,
// memtable flush jobs will be submitted to the HIGH priority pool.
// It is important when the same Env is shared by multiple db instances.
// Without a separate pool, long running major compaction jobs could
// potentially block memtable flush jobs of other db instances, leading to
// unnecessary Put stalls.
// Default: 0
func (opts *Options) SetMaxBackgroundFlushes(value int) {
	C.rocksdb_options_set_max_background_flushes(opts.c, C.int(value))
}

// SetMaxLogFileSize sets the maximal size of the info log file.
//
// If the log file is larger than `max_log_file_size`, a new info log
// file will be created.
// If max_log_file_size == 0, all logs will be written to one log file.
// Default: 0
func (opts *Options) SetMaxLogFileSize(value int) {
	C.rocksdb_options_set_max_log_file_size(opts.c, C.size_t(value))
}

// SetLogFileTimeToRoll sets the time for the info log file to roll (in seconds).
//
// If specified with non-zero value, log file will be rolled
// if it has been active longer than `log_file_time_to_roll`.
// Default: 0 (disabled)
func (opts *Options) SetLogFileTimeToRoll(value int) {
	C.rocksdb_options_set_log_file_time_to_roll(opts.c, C.size_t(value))
}

// SetKeepLogFileNum sets the maximal info log files to be kept.
// Default: 1000
func (opts *Options) SetKeepLogFileNum(value int) {
	C.rocksdb_options_set_keep_log_file_num(opts.c, C.size_t(value))
}

// SetSoftRateLimit sets the soft rate limit.
//
// Puts are delayed 0-1 ms when any level has a compaction score that exceeds
// soft_rate_limit. This is ignored when == 0.0.
// CONSTRAINT: soft_rate_limit <= hard_rate_limit. If this constraint does not
// hold, RocksDB will set soft_rate_limit = hard_rate_limit
// Default: 0.0 (disabled)
func (opts *Options) SetSoftRateLimit(value float64) {
	C.rocksdb_options_set_soft_rate_limit(opts.c, C.double(value))
}

// SetHardRateLimit sets the hard rate limit.
//
// Puts are delayed 1ms at a time when any level has a compaction score that
// exceeds hard_rate_limit. This is ignored when <= 1.0.
// Default: 0.0 (disabled)
func (opts *Options) SetHardRateLimit(value float64) {
	C.rocksdb_options_set_hard_rate_limit(opts.c, C.double(value))
}

// SetRateLimitDelayMaxMilliseconds sets the max time
// a put will be stalled when hard_rate_limit is enforced.
// If 0, then there is no limit.
// Default: 1000
func (opts *Options) SetRateLimitDelayMaxMilliseconds(value uint) {
	C.rocksdb_options_set_rate_limit_delay_max_milliseconds(opts.c, C.uint(value))
}

// SetMaxManifestFileSize sets the maximal manifest file size until is rolled over.
// The older manifest file be deleted.
// Default: MAX_INT so that roll-over does not take place.
func (opts *Options) SetMaxManifestFileSize(value uint64) {
	C.rocksdb_options_set_max_manifest_file_size(opts.c, C.size_t(value))
}

// SetTableCacheNumshardbits sets the number of shards used for table cache.
// Default: 4
func (opts *Options) SetTableCacheNumshardbits(value int) {
	C.rocksdb_options_set_table_cache_numshardbits(opts.c, C.int(value))
}

// SetTableCacheRemoveScanCountLimit sets the count limit during a scan.
//
// During data eviction of table's LRU cache, it would be inefficient
// to strictly follow LRU because this piece of memory will not really
// be released unless its refcount falls to zero. Instead, make two
// passes: the first pass will release items with refcount = 1,
// and if not enough space releases after scanning the number of
// elements specified by this parameter, we will remove items in LRU order.
// Default: 16
func (opts *Options) SetTableCacheRemoveScanCountLimit(value int) {
	C.rocksdb_options_set_table_cache_remove_scan_count_limit(opts.c, C.int(value))
}

// SetArenaBlockSize sets the size of one block in arena memory allocation.
//
// If <= 0, a proper value is automatically calculated (usually 1/10 of
// writer_buffer_size).
// Default: 0
func (opts *Options) SetArenaBlockSize(value int) {
	C.rocksdb_options_set_arena_block_size(opts.c, C.size_t(value))
}

// SetDisableAutoCompactions enable/disable automatic compactions.
//
// Manual compactions can still be issued on this database.
// Default: false
func (opts *Options) SetDisableAutoCompactions(value bool) {
	C.rocksdb_options_set_disable_auto_compactions(opts.c, C.int(btoi(value)))
}

// SetWALRecoveryMode sets the recovery mode
//
// Recovery mode to control the consistency while replaying WAL
// Default: TolerateCorruptedTailRecordsRecovery
func (opts *Options) SetWALRecoveryMode(mode WALRecoveryMode) {
	C.rocksdb_options_set_wal_recovery_mode(opts.c, C.int(mode))
}

// SetWALTtlSeconds sets the WAL ttl in seconds.
//
// The following two options affect how archived logs will be deleted.
// 1. If both set to 0, logs will be deleted asap and will not get into
//    the archive.
// 2. If wal_ttl_seconds is 0 and wal_size_limit_mb is not 0,
//    WAL files will be checked every 10 min and if total size is greater
//    then wal_size_limit_mb, they will be deleted starting with the
//    earliest until size_limit is met. All empty files will be deleted.
// 3. If wal_ttl_seconds is not 0 and wall_size_limit_mb is 0, then
//    WAL files will be checked every wal_ttl_seconds / 2 and those that
//    are older than wal_ttl_seconds will be deleted.
// 4. If both are not 0, WAL files will be checked every 10 min and both
//    checks will be performed with ttl being first.
// Default: 0
func (opts *Options) SetWALTtlSeconds(value uint64) {
	C.rocksdb_options_set_WAL_ttl_seconds(opts.c, C.uint64_t(value))
}

// SetWalSizeLimitMb sets the WAL size limit in MB.
//
// If total size of WAL files is greater then wal_size_limit_mb,
// they will be deleted starting with the earliest until size_limit is met
// Default: 0
func (opts *Options) SetWalSizeLimitMb(value uint64) {
	C.rocksdb_options_set_WAL_size_limit_MB(opts.c, C.uint64_t(value))
}

// SetEnablePipelinedWrite enables pipelined write
//
// Default: false
func (opts *Options) SetEnablePipelinedWrite(value bool) {
	C.rocksdb_options_set_enable_pipelined_write(opts.c, boolToChar(value))
}

// SetManifestPreallocationSize sets the number of bytes
// to preallocate (via fallocate) the manifest files.
//
// Default is 4mb, which is reasonable to reduce random IO
// as well as prevent overallocation for mounts that preallocate
// large amounts of data (such as xfs's allocsize option).
// Default: 4mb
func (opts *Options) SetManifestPreallocationSize(value int) {
	C.rocksdb_options_set_manifest_preallocation_size(opts.c, C.size_t(value))
}

// SetPurgeRedundantKvsWhileFlush enable/disable purging of
// duplicate/deleted keys when a memtable is flushed to storage.
// Default: true
func (opts *Options) SetPurgeRedundantKvsWhileFlush(value bool) {
	C.rocksdb_options_set_purge_redundant_kvs_while_flush(opts.c, boolToChar(value))
}

// SetAllowMmapReads enable/disable mmap reads for reading sst tables.
// Default: false
func (opts *Options) SetAllowMmapReads(value bool) {
	C.rocksdb_options_set_allow_mmap_reads(opts.c, boolToChar(value))
}

// SetAllowMmapWrites enable/disable mmap writes for writing sst tables.
// Default: false
func (opts *Options) SetAllowMmapWrites(value bool) {
	C.rocksdb_options_set_allow_mmap_writes(opts.c, boolToChar(value))
}

// SetUseDirectReads enable/disable direct I/O mode (O_DIRECT) for reads
// Default: false
func (opts *Options) SetUseDirectReads(value bool) {
	C.rocksdb_options_set_use_direct_reads(opts.c, boolToChar(value))
}

// SetUseDirectIOForFlushAndCompaction enable/disable direct I/O mode (O_DIRECT) for both reads and writes in background flush and compactions
// When true, new_table_reader_for_compaction_inputs is forced to true.
// Default: false
func (opts *Options) SetUseDirectIOForFlushAndCompaction(value bool) {
	C.rocksdb_options_set_use_direct_io_for_flush_and_compaction(opts.c, boolToChar(value))
}

// SetIsFdCloseOnExec enable/dsiable child process inherit open files.
// Default: true
func (opts *Options) SetIsFdCloseOnExec(value bool) {
	C.rocksdb_options_set_is_fd_close_on_exec(opts.c, boolToChar(value))
}

// SetSkipLogErrorOnRecovery enable/disable skipping of
// log corruption error on recovery (If client is ok with
// losing most recent changes)
// Default: false
func (opts *Options) SetSkipLogErrorOnRecovery(value bool) {
	C.rocksdb_options_set_skip_log_error_on_recovery(opts.c, boolToChar(value))
}

// SetStatsDumpPeriodSec sets the stats dump period in seconds.
//
// If not zero, dump stats to LOG every stats_dump_period_sec
// Default: 3600 (1 hour)
func (opts *Options) SetStatsDumpPeriodSec(value uint) {
	C.rocksdb_options_set_stats_dump_period_sec(opts.c, C.uint(value))
}

// SetAdviseRandomOnOpen specifies whether we will hint the underlying
// file system that the file access pattern is random, when a sst file is opened.
// Default: true
func (opts *Options) SetAdviseRandomOnOpen(value bool) {
	C.rocksdb_options_set_advise_random_on_open(opts.c, boolToChar(value))
}

// SetDbWriteBufferSize sets the amount of data to build up
// in memtables across all column families before writing to disk.
//
// This is distinct from write_buffer_size, which enforces a limit
// for a single memtable.
//
// This feature is disabled by default. Specify a non-zero value
// to enable it.
//
// Default: 0 (disabled)
func (opts *Options) SetDbWriteBufferSize(value int) {
	C.rocksdb_options_set_db_write_buffer_size(opts.c, C.size_t(value))
}

// SetAccessHintOnCompactionStart specifies the file access pattern
// once a compaction is started.
//
// It will be applied to all input files of a compaction.
// Default: NormalCompactionAccessPattern
func (opts *Options) SetAccessHintOnCompactionStart(value CompactionAccessPattern) {
	C.rocksdb_options_set_access_hint_on_compaction_start(opts.c, C.int(value))
}

// SetUseAdaptiveMutex enable/disable adaptive mutex, which spins
// in the user space before resorting to kernel.
//
// This could reduce context switch when the mutex is not
// heavily contended. However, if the mutex is hot, we could end up
// wasting spin time.
// Default: false
func (opts *Options) SetUseAdaptiveMutex(value bool) {
	C.rocksdb_options_set_use_adaptive_mutex(opts.c, boolToChar(value))
}

// SetBytesPerSync sets the bytes per sync.
//
// Allows OS to incrementally sync files to disk while they are being
// written, asynchronously, in the background.
// Issue one request for every bytes_per_sync written.
// Default: 0 (disabled)
func (opts *Options) SetBytesPerSync(value uint64) {
	C.rocksdb_options_set_bytes_per_sync(opts.c, C.uint64_t(value))
}

// SetCompactionStyle sets the compaction style.
// Default: LevelCompactionStyle
func (opts *Options) SetCompactionStyle(value CompactionStyle) {
	C.rocksdb_options_set_compaction_style(opts.c, C.int(value))
}

// SetUniversalCompactionOptions sets the options needed
// to support Universal Style compactions.
// Default: nil
func (opts *Options) SetUniversalCompactionOptions(value *UniversalCompactionOptions) {
	C.rocksdb_options_set_universal_compaction_options(opts.c, value.c)
}

// SetFIFOCompactionOptions sets the options for FIFO compaction style.
// Default: nil
func (opts *Options) SetFIFOCompactionOptions(value *FIFOCompactionOptions) {
	C.rocksdb_options_set_fifo_compaction_options(opts.c, value.c)
}

// GetStatisticsString returns the statistics as a string.
func (opts *Options) GetStatisticsString() string {
	sString := C.rocksdb_options_statistics_get_string(opts.c)
	defer C.rocksdb_free(unsafe.Pointer(sString))
	return C.GoString(sString)
}

// SetRateLimiter sets the rate limiter of the options.
// Use to control write rate of flush and compaction. Flush has higher
// priority than compaction. Rate limiting is disabled if nullptr.
// If rate limiter is enabled, bytes_per_sync is set to 1MB by default.
// Default: nullptr
func (opts *Options) SetRateLimiter(rateLimiter *RateLimiter) {
	C.rocksdb_options_set_ratelimiter(opts.c, rateLimiter.c)
}

// SetMaxSequentialSkipInIterations specifies whether an iteration->Next()
// sequentially skips over keys with the same user-key or not.
//
// This number specifies the number of keys (with the same userkey)
// that will be sequentially skipped before a reseek is issued.
// Default: 8
func (opts *Options) SetMaxSequentialSkipInIterations(value uint64) {
	C.rocksdb_options_set_max_sequential_skip_in_iterations(opts.c, C.uint64_t(value))
}

// SetInplaceUpdateSupport enable/disable thread-safe inplace updates.
//
// Requires updates if
// * key exists in current memtable
// * new sizeof(new_value) <= sizeof(old_value)
// * old_value for that key is a put i.e. kTypeValue
// Default: false.
func (opts *Options) SetInplaceUpdateSupport(value bool) {
	C.rocksdb_options_set_inplace_update_support(opts.c, boolToChar(value))
}

// SetInplaceUpdateNumLocks sets the number of locks used for inplace update.
// Default: 10000, if inplace_update_support = true, else 0.
func (opts *Options) SetInplaceUpdateNumLocks(value int) {
	C.rocksdb_options_set_inplace_update_num_locks(opts.c, C.size_t(value))
}

// SetMemtableHugePageSize sets the page size for huge page for
// arena used by the memtable.
// If <=0, it won't allocate from huge page but from malloc.
// Users are responsible to reserve huge pages for it to be allocated. For
// example:
//      sysctl -w vm.nr_hugepages=20
// See linux doc Documentation/vm/hugetlbpage.txt
// If there isn't enough free huge page available, it will fall back to
// malloc.
//
// Dynamically changeable through SetOptions() API
func (opts *Options) SetMemtableHugePageSize(value int) {
	C.rocksdb_options_set_memtable_huge_page_size(opts.c, C.size_t(value))
}

// SetBloomLocality sets the bloom locality.
//
// Control locality of bloom filter probes to improve cache miss rate.
// This option only applies to memtable prefix bloom and plaintable
// prefix bloom. It essentially limits the max number of cache lines each
// bloom filter check can touch.
// This optimization is turned off when set to 0. The number should never
// be greater than number of probes. This option can boost performance
// for in-memory workload but should use with care since it can cause
// higher false positive rate.
// Default: 0
func (opts *Options) SetBloomLocality(value uint32) {
	C.rocksdb_options_set_bloom_locality(opts.c, C.uint32_t(value))
}

// SetMaxSuccessiveMerges sets the maximum number of
// successive merge operations on a key in the memtable.
//
// When a merge operation is added to the memtable and the maximum number of
// successive merges is reached, the value of the key will be calculated and
// inserted into the memtable instead of the merge operation. This will
// ensure that there are never more than max_successive_merges merge
// operations in the memtable.
// Default: 0 (disabled)
func (opts *Options) SetMaxSuccessiveMerges(value int) {
	C.rocksdb_options_set_max_successive_merges(opts.c, C.size_t(value))
}

// EnableStatistics enable statistics.
func (opts *Options) EnableStatistics() {
	C.rocksdb_options_enable_statistics(opts.c)
}

// PrepareForBulkLoad prepare the DB for bulk loading.
//
// All data will be in level 0 without any automatic compaction.
// It's recommended to manually call CompactRange(NULL, NULL) before reading
// from the database, because otherwise the read can be very slow.
func (opts *Options) PrepareForBulkLoad() {
	C.rocksdb_options_prepare_for_bulk_load(opts.c)
}

// SetMemtableVectorRep sets a MemTableRep which is backed by a vector.
//
// On iteration, the vector is sorted. This is useful for workloads where
// iteration is very rare and writes are generally not issued after reads begin.
func (opts *Options) SetMemtableVectorRep() {
	C.rocksdb_options_set_memtable_vector_rep(opts.c)
}

// SetHashSkipListRep sets a hash skip list as MemTableRep.
//
// It contains a fixed array of buckets, each
// pointing to a skiplist (null if the bucket is empty).
//
// bucketCount:             number of fixed array buckets
// skiplistHeight:          the max height of the skiplist
// skiplistBranchingFactor: probabilistic size ratio between adjacent
//                          link lists in the skiplist
func (opts *Options) SetHashSkipListRep(bucketCount int, skiplistHeight, skiplistBranchingFactor int32) {
	C.rocksdb_options_set_hash_skip_list_rep(opts.c, C.size_t(bucketCount), C.int32_t(skiplistHeight), C.int32_t(skiplistBranchingFactor))
}

// SetHashLinkListRep sets a hashed linked list as MemTableRep.
//
// It contains a fixed array of buckets, each pointing to a sorted single
// linked list (null if the bucket is empty).
//
// bucketCount: number of fixed array buckets
func (opts *Options) SetHashLinkListRep(bucketCount int) {
	C.rocksdb_options_set_hash_link_list_rep(opts.c, C.size_t(bucketCount))
}

// SetPlainTableFactory sets a plain table factory with prefix-only seek.
//
// For this factory, you need to set prefix_extractor properly to make it
// work. Look-up will starts with prefix hash lookup for key prefix. Inside the
// hash bucket found, a binary search is executed for hash conflicts. Finally,
// a linear search is used.
//
// keyLen: 			plain table has optimization for fix-sized keys,
// 					which can be specified via keyLen.
// bloomBitsPerKey: the number of bits used for bloom filer per prefix. You
//                  may disable it by passing a zero.
// hashTableRatio:  the desired utilization of the hash table used for prefix
//                  hashing. hashTableRatio = number of prefixes / #buckets
//                  in the hash table
// indexSparseness: inside each prefix, need to build one index record for how
//                  many keys for binary search inside each hash bucket.
func (opts *Options) SetPlainTableFactory(keyLen uint32, bloomBitsPerKey int, hashTableRatio float64, indexSparseness int) {
	C.rocksdb_options_set_plain_table_factory(opts.c, C.uint32_t(keyLen), C.int(bloomBitsPerKey), C.double(hashTableRatio), C.size_t(indexSparseness))
}

// SetCreateIfMissingColumnFamilies specifies whether the column families
// should be created if they are missing.
func (opts *Options) SetCreateIfMissingColumnFamilies(value bool) {
	C.rocksdb_options_set_create_missing_column_families(opts.c, boolToChar(value))
}

// SetBlockBasedTableFactory sets the block based table factory.
func (opts *Options) SetBlockBasedTableFactory(value *BlockBasedTableOptions) {
	opts.bbto = value
	C.rocksdb_options_set_block_based_table_factory(opts.c, value.c)
}

// SetAllowIngestBehind sets allow_ingest_behind
// Set this option to true during creation of database if you want
// to be able to ingest behind (call IngestExternalFile() skipping keys
// that already exist, rather than overwriting matching keys).
// Setting this option to true will affect 2 things:
// 1) Disable some internal optimizations around SST file compression
// 2) Reserve bottom-most level for ingested files only.
// 3) Note that num_levels should be >= 3 if this option is turned on.
//
// DEFAULT: false
// Immutable.
func (opts *Options) SetAllowIngestBehind(value bool) {
	C.rocksdb_options_set_allow_ingest_behind(opts.c, boolToChar(value))
}

// SetMemTablePrefixBloomSizeRatio sets memtable_prefix_bloom_size_ratio
// if prefix_extractor is set and memtable_prefix_bloom_size_ratio is not 0,
// create prefix bloom for memtable with the size of
// write_buffer_size * memtable_prefix_bloom_size_ratio.
// If it is larger than 0.25, it is sanitized to 0.25.
//
// Default: 0 (disable)
func (opts *Options) SetMemTablePrefixBloomSizeRatio(value float64) {
	C.rocksdb_options_set_memtable_prefix_bloom_size_ratio(opts.c, C.double(value))
}

// SetOptimizeFiltersForHits sets optimize_filters_for_hits
// This flag specifies that the implementation should optimize the filters
// mainly for cases where keys are found rather than also optimize for keys
// missed. This would be used in cases where the application knows that
// there are very few misses or the performance in the case of misses is not
// important.
//
// For now, this flag allows us to not store filters for the last level i.e
// the largest level which contains data of the LSM store. For keys which
// are hits, the filters in this level are not useful because we will search
// for the data anyway. NOTE: the filters in other levels are still useful
// even for key hit because they tell us whether to look in that level or go
// to the higher level.
//
// Default: false
func (opts *Options) SetOptimizeFiltersForHits(value bool) {
	C.rocksdb_options_set_optimize_filters_for_hits(opts.c, C.int(btoi(value)))
}

// Destroy deallocates the Options object.
func (opts *Options) Destroy() {
	C.rocksdb_options_destroy(opts.c)
	if opts.ccmp != nil {
		C.rocksdb_comparator_destroy(opts.ccmp)
	}
	// don't destroy the opts.cst here, it has already been
	// associated with a PrefixExtractor and this will segfault
	if opts.ccf != nil {
		C.rocksdb_compactionfilter_destroy(opts.ccf)
	}
	opts.c = nil
	opts.env = nil
	opts.bbto = nil
}
