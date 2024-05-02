package grocksdb

// #include "rocksdb/c.h"
// #include "grocksdb.h"
import "C"

import (
	"fmt"
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

// WALRecoveryMode mode of WAL Recovery.
type WALRecoveryMode int

const (
	// TolerateCorruptedTailRecordsRecovery is original levelDB recovery
	// We tolerate incomplete record in trailing data on all logs
	// Use case : This is legacy behavior
	TolerateCorruptedTailRecordsRecovery = WALRecoveryMode(0)
	// AbsoluteConsistencyRecovery recover from clean shutdown
	// We don't expect to find any corruption in the WAL
	// Use case : This is ideal for unit tests and rare applications that
	// can require high consistency guarantee
	AbsoluteConsistencyRecovery = WALRecoveryMode(1)
	// PointInTimeRecovery recover to point-in-time consistency (default)
	// We stop the WAL playback on discovering WAL inconsistency
	// Use case : Ideal for systems that have disk controller cache like
	// hard disk, SSD without super capacitor that store related data
	PointInTimeRecovery = WALRecoveryMode(2)
	// SkipAnyCorruptedRecordsRecovery recovery after a disaster
	// We ignore any corruption in the WAL and try to salvage as much data as
	// possible
	// Use case : Ideal for last ditch effort to recover data or systems that
	// operate with low grade unrelated data
	SkipAnyCorruptedRecordsRecovery = WALRecoveryMode(3)
)

// PrepopulateBlob represents strategy for prepopulate warm/hot blobs which are already in memory into
// blob cache at the time of flush.
type PrepopulateBlob int

const (
	// PrepopulateBlobDisable disables prepopulate blob cache.
	PrepopulateBlobDisable = PrepopulateBlob(0)
	// PrepopulateBlobFlushOnly prepopulates blobs during flush only.
	PrepopulateBlobFlushOnly = PrepopulateBlob(1)
)

// Options represent all of the available options when opening a database with Open.
type Options struct {
	c   *C.rocksdb_options_t
	env *C.rocksdb_env_t

	// Hold references for GC.
	bbto *BlockBasedTableOptions

	// We keep these so we can free their memory in Destroy.
	ccmp *C.rocksdb_comparator_t
	cmo  *C.rocksdb_mergeoperator_t
	cst  *C.rocksdb_slicetransform_t
	ccf  *C.rocksdb_compactionfilter_t
}

// NewDefaultOptions creates the default Options.
func NewDefaultOptions() *Options {
	return newNativeOptions(C.rocksdb_options_create())
}

// NewNativeOptions creates a Options object.
func newNativeOptions(c *C.rocksdb_options_t) *Options {
	return &Options{c: c}
}

// GetOptionsFromString creates a Options object from existing opt and string.
// If base is nil, a default opt create by NewDefaultOptions will be used as base opt.
func GetOptionsFromString(base *Options, optStr string) (newOpt *Options, err error) {
	providedBaseNil := base == nil
	if providedBaseNil {
		base = NewDefaultOptions()
	}

	var (
		cErr    *C.char
		cOptStr = C.CString(optStr)
	)

	newOpt = NewDefaultOptions()
	C.rocksdb_get_options_from_string(base.c, cOptStr, newOpt.c, &cErr)
	if err = fromCError(cErr); err != nil {
		newOpt.Destroy()
	}

	C.free(unsafe.Pointer(cOptStr))
	if providedBaseNil {
		base.Destroy()
	}

	return
}

// Clone the options
func (opts *Options) Clone() *Options {
	cloned := *opts
	cloned.c = C.rocksdb_options_create_copy(opts.c)
	return &cloned
}

// -------------------
// Parameters that affect behavior

// SetCompactionFilter sets the specified compaction filter
// which will be applied on compactions.
//
// Default: nil
func (opts *Options) SetCompactionFilter(value CompactionFilter) {
	C.rocksdb_compactionfilter_destroy(opts.ccf)

	if nc, ok := value.(*nativeCompactionFilter); ok {
		opts.ccf = nc.c
	} else {
		idx := registerCompactionFilter(value)
		opts.ccf = C.gorocksdb_compactionfilter_create(C.uintptr_t(idx))
	}

	C.rocksdb_options_set_compaction_filter(opts.c, opts.ccf)
}

// SetComparator sets the comparator which define the order of keys in the table.
// This operation is `move`, thus underlying native c-pointer is owned by Options.
// `cmp` is no longer usable.
//
// Default: a comparator that uses lexicographic byte-wise ordering
func (opts *Options) SetComparator(cmp *Comparator) {
	cmp_ := unsafe.Pointer(cmp.c)
	opts.SetNativeComparator(cmp_)
	cmp.c = nil
}

// SetNativeComparator sets the comparator which define the order of keys in the table.
//
// Default: a comparator that uses lexicographic byte-wise ordering
func (opts *Options) SetNativeComparator(cmp unsafe.Pointer) {
	C.rocksdb_comparator_destroy(opts.ccmp)
	opts.ccmp = (*C.rocksdb_comparator_t)(cmp)
	C.rocksdb_options_set_comparator(opts.c, opts.ccmp)
}

// SetMergeOperator sets the merge operator which will be called
// if a merge operations are used.
//
// Default: nil
func (opts *Options) SetMergeOperator(value MergeOperator) {
	C.rocksdb_mergeoperator_destroy(opts.cmo)

	if nmo, ok := value.(*nativeMergeOperator); ok {
		opts.cmo = nmo.c
	} else {
		idx := registerMergeOperator(value)
		opts.cmo = C.gorocksdb_mergeoperator_create(C.uintptr_t(idx))
	}

	C.rocksdb_options_set_merge_operator(opts.c, opts.cmo)
}

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

// CreateIfMissing checks if create_if_mission option is set
func (opts *Options) CreateIfMissing() bool {
	return charToBool(C.rocksdb_options_get_create_if_missing(opts.c))
}

// SetErrorIfExists specifies whether an error should be raised
// if the database already exists.
// Default: false
func (opts *Options) SetErrorIfExists(value bool) {
	C.rocksdb_options_set_error_if_exists(opts.c, boolToChar(value))
}

// ErrorIfExists checks if error_if_exist option is set
func (opts *Options) ErrorIfExists() bool {
	return charToBool(C.rocksdb_options_get_error_if_exists(opts.c))
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

// ParanoidChecks checks if paranoid_check option is set
func (opts *Options) ParanoidChecks() bool {
	return charToBool(C.rocksdb_options_get_paranoid_checks(opts.c))
}

// SetDBPaths sets the db_paths option.
//
// db_paths is a list of paths where SST files can be put into, with its target size.
// Newer data is placed into paths specified earlier in the vector while
// older data gradually moves to paths specified later in the vector.
//
// For example, you have a flash device with 10GB allocated for the DB,
// as well as a hard drive of 2TB, you should config it to be:
//
//	[{"/flash_path", 10GB}, {"/hard_drive", 2TB}]
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
//
// Default: empty
func (opts *Options) SetDBPaths(dbpaths []*DBPath) {
	if n := len(dbpaths); n > 0 {
		cDbpaths := make([]*C.rocksdb_dbpath_t, n)
		for i, v := range dbpaths {
			cDbpaths[i] = v.c
		}

		C.rocksdb_options_set_db_paths(opts.c, &cDbpaths[0], C.size_t(n))
	}
}

// SetCFPaths sets cf_paths option. cf_paths is a list of paths where SST files
// for this column family can be put into, with its target size.
//
// Similar to db_paths, newer data is placed into paths specified earlier in the
// vector while older data gradually moves to paths specified later in the vector.
//
// Note that, if a path is supplied to multiple column
// families, it would have files and total size from all
// the column families combined. User should provision for the
// total size(from all the column families) in such cases.
//
// If left empty, db_paths will be used.
// Default: empty
func (opts *Options) SetCFPaths(dbpaths []*DBPath) {
	if n := len(dbpaths); n > 0 {
		cDbpaths := make([]*C.rocksdb_dbpath_t, n)
		for i, v := range dbpaths {
			cDbpaths[i] = v.c
		}

		C.rocksdb_options_set_cf_paths(opts.c, &cDbpaths[0], C.size_t(n))
	}
}

// SetEnv sets the specified object to interact with the environment,
// e.g. to read/write files, schedule background work, etc.
//
// NOTE: move semantic. Don't use env after calling this function
func (opts *Options) SetEnv(env *Env) {
	if opts.env != nil {
		C.rocksdb_env_destroy(opts.env)
	}

	C.rocksdb_options_set_env(opts.c, env.c)
	opts.env = env.c

	env.c = nil
}

// SetInfoLogLevel sets the info log level.
//
// Default: InfoInfoLogLevel
func (opts *Options) SetInfoLogLevel(value InfoLogLevel) {
	C.rocksdb_options_set_info_log_level(opts.c, C.int(value))
}

// GetInfoLogLevel gets the info log level which options hold
func (opts *Options) GetInfoLogLevel() InfoLogLevel {
	return InfoLogLevel(C.rocksdb_options_get_info_log_level(opts.c))
}

// IncreaseParallelism sets the parallelism.
//
// By default, RocksDB uses only one background thread for flush and
// compaction. Calling this function will set it up such that total of
// `total_threads` is used. Good value for `total_threads` is the number of
// cores. You almost definitely want to call this function if your system is
// bottlenecked by RocksDB.
func (opts *Options) IncreaseParallelism(totalThreads int) {
	C.rocksdb_options_increase_parallelism(opts.c, C.int(totalThreads))
}

// OptimizeForPointLookup optimize the DB for point lookups.
//
// Use this if you don't need to keep the data sorted, i.e. you'll never use
// an iterator, only Put() and Get() API calls
//
// If you use this with rocksdb >= 5.0.2, you must call `SetAllowConcurrentMemtableWrites(false)`
// to avoid an assertion error immediately on opening the db.
func (opts *Options) OptimizeForPointLookup(blockCacheSizeMB uint64) {
	C.rocksdb_options_optimize_for_point_lookup(opts.c, C.uint64_t(blockCacheSizeMB))
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
func (opts *Options) OptimizeLevelStyleCompaction(memtableMemoryBudget uint64) {
	C.rocksdb_options_optimize_level_style_compaction(opts.c, C.uint64_t(memtableMemoryBudget))
}

// OptimizeUniversalStyleCompaction optimize the DB for universal compaction.
// See note on OptimizeLevelStyleCompaction.
func (opts *Options) OptimizeUniversalStyleCompaction(memtableMemoryBudget uint64) {
	C.rocksdb_options_optimize_universal_style_compaction(opts.c, C.uint64_t(memtableMemoryBudget))
}

// SetAllowConcurrentMemtableWrites whether to allow concurrent memtable writes. Conccurent writes are
// not supported by all memtable factories (currently only SkipList memtables).
// As of rocksdb 5.0.2 you must call `SetAllowConcurrentMemtableWrites(false)`
// if you use `OptimizeForPointLookup`.
func (opts *Options) SetAllowConcurrentMemtableWrites(allow bool) {
	C.rocksdb_options_set_allow_concurrent_memtable_write(opts.c, boolToChar(allow))
}

// AllowConcurrentMemtableWrites whether to allow concurrent memtable writes. Conccurent writes are
// not supported by all memtable factories (currently only SkipList memtables).
// As of rocksdb 5.0.2 you must call `SetAllowConcurrentMemtableWrites(false)`
// if you use `OptimizeForPointLookup`.
func (opts *Options) AllowConcurrentMemtableWrites() bool {
	return charToBool(C.rocksdb_options_get_allow_concurrent_memtable_write(opts.c))
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
//
// Default: 64MB
func (opts *Options) SetWriteBufferSize(value uint64) {
	C.rocksdb_options_set_write_buffer_size(opts.c, C.size_t(value))
}

// GetWriteBufferSize gets write_buffer_size which is set for options
func (opts *Options) GetWriteBufferSize() uint64 {
	return uint64(C.rocksdb_options_get_write_buffer_size(opts.c))
}

// SetMaxWriteBufferNumber sets the maximum number of write buffers
// that are built up in memory.
//
// The default is 2, so that when 1 write buffer is being flushed to
// storage, new writes can continue to the other write buffer.
//
// Default: 2
func (opts *Options) SetMaxWriteBufferNumber(value int) {
	C.rocksdb_options_set_max_write_buffer_number(opts.c, C.int(value))
}

// GetMaxWriteBufferNumber gets the maximum number of write buffers
// that are built up in memory.
func (opts *Options) GetMaxWriteBufferNumber() int {
	return int(C.rocksdb_options_get_max_write_buffer_number(opts.c))
}

// SetMinWriteBufferNumberToMerge sets the minimum number of write buffers
// that will be merged together before writing to storage.
//
// If set to 1, then all write buffers are flushed to L0 as individual files
// and this increases read amplification because a get request has to check
// in all of these files. Also, an in-memory merge may result in writing lesser
// data to storage if there are duplicate records in each of these
// individual write buffers.
//
// Default: 1
func (opts *Options) SetMinWriteBufferNumberToMerge(value int) {
	C.rocksdb_options_set_min_write_buffer_number_to_merge(opts.c, C.int(value))
}

// GetMinWriteBufferNumberToMerge gets the minimum number of write buffers
// that will be merged together before writing to storage.
func (opts *Options) GetMinWriteBufferNumberToMerge() int {
	return int(C.rocksdb_options_get_min_write_buffer_number_to_merge(opts.c))
}

// SetMaxOpenFiles sets the number of open files that can be used by the DB.
//
// You may need to increase this if your database has a large working set
// (budget one open file per 2MB of working set).
//
// Default: -1 - unlimited
func (opts *Options) SetMaxOpenFiles(value int) {
	C.rocksdb_options_set_max_open_files(opts.c, C.int(value))
}

// GetMaxOpenFiles gets the number of open files that can be used by the DB.
func (opts *Options) GetMaxOpenFiles() int {
	return int(C.rocksdb_options_get_max_open_files(opts.c))
}

// SetMaxFileOpeningThreads sets the maximum number of file opening threads.
// If max_open_files is -1, DB will open all files on DB::Open(). You can
// use this option to increase the number of threads used to open the files.
//
// Default: 16
func (opts *Options) SetMaxFileOpeningThreads(value int) {
	C.rocksdb_options_set_max_file_opening_threads(opts.c, C.int(value))
}

// GetMaxFileOpeningThreads gets the maximum number of file opening threads.
func (opts *Options) GetMaxFileOpeningThreads() int {
	return int(C.rocksdb_options_get_max_file_opening_threads(opts.c))
}

// SetMaxTotalWalSize sets the maximum total wal size (in bytes).
// Once write-ahead logs exceed this size, we will start forcing the flush of
// column families whose memtables are backed by the oldest live WAL file
// (i.e. the ones that are causing all the space amplification). If set to 0
// (default), we will dynamically choose the WAL size limit to be
// [sum of all write_buffer_size * max_write_buffer_number] * 4
// Default: 0
func (opts *Options) SetMaxTotalWalSize(value uint64) {
	C.rocksdb_options_set_max_total_wal_size(opts.c, C.uint64_t(value))
}

// GetMaxTotalWalSize gets the maximum total wal size (in bytes).
func (opts *Options) GetMaxTotalWalSize() uint64 {
	return uint64(C.rocksdb_options_get_max_total_wal_size(opts.c))
}

// SetCompression sets the compression algorithm.
//
// Default: SnappyCompression, which gives lightweight but fast
// compression.
func (opts *Options) SetCompression(value CompressionType) {
	C.rocksdb_options_set_compression(opts.c, C.int(value))
}

// GetCompression returns the compression algorithm.
func (opts *Options) GetCompression() CompressionType {
	return CompressionType(C.rocksdb_options_get_compression(opts.c))
}

// SetCompressionOptions sets different options for compression algorithms.
func (opts *Options) SetCompressionOptions(value CompressionOptions) {
	C.rocksdb_options_set_compression_options(
		opts.c,
		C.int(value.WindowBits),
		C.int(value.Level),
		C.int(value.Strategy),
		C.int(value.MaxDictBytes),
	)
}

// SetBottommostCompression sets the compression algorithm for
// bottommost level.
func (opts *Options) SetBottommostCompression(value CompressionType) {
	C.rocksdb_options_set_bottommost_compression(opts.c, C.int(value))
}

// GetBottommostCompression returns the compression algorithm for
// bottommost level.
func (opts *Options) GetBottommostCompression() CompressionType {
	return CompressionType(C.rocksdb_options_get_bottommost_compression(opts.c))
}

// SetBottommostCompressionOptions sets different options for compression algorithms, for bottommost.
//
// `enabled` true to use these compression options.
func (opts *Options) SetBottommostCompressionOptions(value CompressionOptions, enabled bool) {
	C.rocksdb_options_set_bottommost_compression_options(
		opts.c,
		C.int(value.WindowBits),
		C.int(value.Level),
		C.int(value.Strategy),
		C.int(value.MaxDictBytes),
		boolToChar(enabled),
	)
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
	if len(value) > 0 {
		cLevels := make([]C.int, len(value))
		for i, v := range value {
			cLevels[i] = C.int(v)
		}

		C.rocksdb_options_set_compression_per_level(opts.c, &cLevels[0], C.size_t(len(value)))
	}
}

// SetCompressionOptionsZstdMaxTrainBytes sets maximum size of training data passed
// to zstd's dictionary trainer. Using zstd's dictionary trainer can achieve even
// better compression ratio improvements than using `max_dict_bytes` alone.
//
// The training data will be used to generate a dictionary of max_dict_bytes.
//
// Default: 0.
func (opts *Options) SetCompressionOptionsZstdMaxTrainBytes(value int) {
	C.rocksdb_options_set_compression_options_zstd_max_train_bytes(opts.c, C.int(value))
}

// GetCompressionOptionsZstdMaxTrainBytes gets maximum size of training data passed
// to zstd's dictionary trainer. Using zstd's dictionary trainer can achieve even
// better compression ratio improvements than using `max_dict_bytes` alone.
func (opts *Options) GetCompressionOptionsZstdMaxTrainBytes() int {
	return int(C.rocksdb_options_get_compression_options_zstd_max_train_bytes(opts.c))
}

// SetCompressionOptionsZstdDictTrainer uses/not use zstd trainer to generate dictionaries.
// When this option is set to true, zstd_max_train_bytes of training data sampled from
// max_dict_buffer_bytes buffered data will be passed to zstd dictionary trainer to generate a
// dictionary of size max_dict_bytes.
//
// When this option is false, zstd's API ZDICT_finalizeDictionary() will be
// called to generate dictionaries. zstd_max_train_bytes of training sampled
// data will be passed to this API. Using this API should save CPU time on
// dictionary training, but the compression ratio may not be as good as using
// a dictionary trainer.
//
// Default: true
func (opts *Options) SetCompressionOptionsZstdDictTrainer(enabled bool) {
	C.rocksdb_options_set_compression_options_use_zstd_dict_trainer(opts.c, boolToChar(enabled))
}

// GetCompressionOptionsZstdDictTrainer returns if zstd dict trainer is used or not.
func (opts *Options) GetCompressionOptionsZstdDictTrainer() bool {
	return charToBool(C.rocksdb_options_get_compression_options_use_zstd_dict_trainer(opts.c))
}

// SetCompressionOptionsParallelThreads sets number of threads for
// parallel compression. Parallel compression is enabled only if threads > 1.
//
// This option is valid only when BlockBasedTable is used.
//
// When parallel compression is enabled, SST size file sizes might be
// more inflated compared to the target size, because more data of unknown
// compressed size is in flight when compression is parallelized. To be
// reasonably accurate, this inflation is also estimated by using historical
// compression ratio and current bytes inflight.
//
// Default: 1.
//
// Note: THE FEATURE IS STILL EXPERIMENTAL
func (opts *Options) SetCompressionOptionsParallelThreads(n int) {
	C.rocksdb_options_set_compression_options_parallel_threads(opts.c, C.int(n))
}

// GetCompressionOptionsParallelThreads returns  number of threads for
// parallel compression. Parallel compression is enabled only if threads > 1.
//
// This option is valid only when BlockBasedTable is used.
// Default: 1.
//
// Note: THE FEATURE IS STILL EXPERIMENTAL
func (opts *Options) GetCompressionOptionsParallelThreads() int {
	return int(C.rocksdb_options_get_compression_options_parallel_threads(opts.c))
}

// SetCompressionOptionsMaxDictBufferBytes limits on data buffering when
// gathering samples to build a dictionary.  Zero means no limit. When dictionary
// is disabled (`max_dict_bytes == 0`), enabling this limit (`max_dict_buffer_bytes != 0`)
// has no effect.
//
// In compaction, the buffering is limited to the target file size (see
// `target_file_size_base` and `target_file_size_multiplier`) even if this
// setting permits more buffering. Since we cannot determine where the file
// should be cut until data blocks are compressed with dictionary, buffering
// more than the target file size could lead to selecting samples that belong
// to a later output SST.
//
// Limiting too strictly may harm dictionary effectiveness since it forces
// RocksDB to pick samples from the initial portion of the output SST, which
// may not be representative of the whole file. Configuring this limit below
// `zstd_max_train_bytes` (when enabled) can restrict how many samples we can
// pass to the dictionary trainer. Configuring it below `max_dict_bytes` can
// restrict the size of the final dictionary.
//
// Default: 0 (unlimited)
func (opts *Options) SetCompressionOptionsMaxDictBufferBytes(value uint64) {
	C.rocksdb_options_set_compression_options_max_dict_buffer_bytes(opts.c, C.uint64_t(value))
}

// GetCompressionOptionsMaxDictBufferBytes returns the limit on data buffering when
// gathering samples to build a dictionary.  Zero means no limit. When dictionary
// is disabled (`max_dict_bytes == 0`), enabling this limit (`max_dict_buffer_bytes != 0`)
// has no effect.
func (opts *Options) GetCompressionOptionsMaxDictBufferBytes() uint64 {
	return uint64(C.rocksdb_options_get_compression_options_max_dict_buffer_bytes(opts.c))
}

// SetBottommostCompressionOptionsZstdMaxTrainBytes sets maximum size of training data passed
// to zstd's dictionary trainer for bottommost level. Using zstd's dictionary trainer can achieve even
// better compression ratio improvements than using `max_dict_bytes` alone.
//
// `enabled` true to use these compression options.
func (opts *Options) SetBottommostCompressionOptionsZstdMaxTrainBytes(value int, enabled bool) {
	C.rocksdb_options_set_bottommost_compression_options_zstd_max_train_bytes(opts.c, C.int(value), boolToChar(enabled))
}

// SetBottommostCompressionOptionsMaxDictBufferBytes limits on data buffering
// when gathering samples to build a dictionary, for bottom most level.
// Zero means no limit. When dictionary is disabled (`max_dict_bytes == 0`),
// enabling this limit (`max_dict_buffer_bytes != 0`) has no effect.
//
// In compaction, the buffering is limited to the target file size (see
// `target_file_size_base` and `target_file_size_multiplier`) even if this
// setting permits more buffering. Since we cannot determine where the file
// should be cut until data blocks are compressed with dictionary, buffering
// more than the target file size could lead to selecting samples that belong
// to a later output SST.
//
// Limiting too strictly may harm dictionary effectiveness since it forces
// RocksDB to pick samples from the initial portion of the output SST, which
// may not be representative of the whole file. Configuring this limit below
// `zstd_max_train_bytes` (when enabled) can restrict how many samples we can
// pass to the dictionary trainer. Configuring it below `max_dict_bytes` can
// restrict the size of the final dictionary.
//
// Default: 0 (unlimited)
func (opts *Options) SetBottommostCompressionOptionsMaxDictBufferBytes(value uint64, enabled bool) {
	C.rocksdb_options_set_bottommost_compression_options_max_dict_buffer_bytes(
		opts.c,
		C.uint64_t(value),
		boolToChar(enabled),
	)
}

// SetBottommostCompressionOptionsZstdDictTrainer uses/not use zstd trainer to generate dictionaries.
// When this option is set to true, zstd_max_train_bytes of training data sampled from
// max_dict_buffer_bytes buffered data will be passed to zstd dictionary trainer to generate a
// dictionary of size max_dict_bytes.
//
// When this option is false, zstd's API ZDICT_finalizeDictionary() will be
// called to generate dictionaries. zstd_max_train_bytes of training sampled
// data will be passed to this API. Using this API should save CPU time on
// dictionary training, but the compression ratio may not be as good as using
// a dictionary trainer.
//
// Default: true
func (opts *Options) SetBottommostCompressionOptionsZstdDictTrainer(enabled bool) {
	c := boolToChar(enabled)
	C.rocksdb_options_set_bottommost_compression_options_use_zstd_dict_trainer(opts.c, c, c)
}

// GetBottommostCompressionOptionsZstdDictTrainer returns if zstd dict trainer is used or not.
func (opts *Options) GetBottommostCompressionOptionsZstdDictTrainer() bool {
	return charToBool(C.rocksdb_options_get_bottommost_compression_options_use_zstd_dict_trainer(opts.c))
}

// SetMinLevelToCompress sets the start level to use compression.
func (opts *Options) SetMinLevelToCompress(value int) {
	C.rocksdb_options_set_min_level_to_compress(opts.c, C.int(value))
}

// SetPrefixExtractor sets the prefic extractor.
//
// If set, use the specified function to determine the
// prefixes for keys. These prefixes will be placed in the filter.
// Depending on the workload, this can reduce the number of read-IOP
// cost for scans when a prefix is passed via ReadOptions to
// db.NewIterator().
//
// Note: move semantic. Don't use slice transform after calling this function.
func (opts *Options) SetPrefixExtractor(value SliceTransform) {
	C.rocksdb_slicetransform_destroy(opts.cst)

	if nst, ok := value.(*nativeSliceTransform); ok {
		opts.cst = nst.c
	} else {
		idx := registerSliceTransform(value)
		opts.cst = C.gorocksdb_slicetransform_create(C.uintptr_t(idx))
	}

	C.rocksdb_options_set_prefix_extractor(opts.c, opts.cst)
}

// SetNumLevels sets the number of levels for this database.
//
// Default: 7
func (opts *Options) SetNumLevels(value int) {
	C.rocksdb_options_set_num_levels(opts.c, C.int(value))
}

// GetNumLevels gets the number of levels.
func (opts *Options) GetNumLevels() int {
	return int(C.rocksdb_options_get_num_levels(opts.c))
}

// SetLevel0FileNumCompactionTrigger sets the number of files
// to trigger level-0 compaction.
//
// A value <0 means that level-0 compaction will not be
// triggered by number of files at all.
//
// Default: 2
func (opts *Options) SetLevel0FileNumCompactionTrigger(value int) {
	C.rocksdb_options_set_level0_file_num_compaction_trigger(opts.c, C.int(value))
}

// GetLevel0FileNumCompactionTrigger gets the number of files to trigger level-0 compaction.
func (opts *Options) GetLevel0FileNumCompactionTrigger() int {
	return int(C.rocksdb_options_get_level0_file_num_compaction_trigger(opts.c))
}

// SetLevel0SlowdownWritesTrigger sets the soft limit on number of level-0 files.
//
// We start slowing down writes at this point.
// A value <0 means that no writing slow down will be triggered by
// number of files in level-0.
//
// Default: 20
func (opts *Options) SetLevel0SlowdownWritesTrigger(value int) {
	C.rocksdb_options_set_level0_slowdown_writes_trigger(opts.c, C.int(value))
}

// GetLevel0SlowdownWritesTrigger gets the soft limit on number of level-0 files.
// We start slowing down writes at this point.
func (opts *Options) GetLevel0SlowdownWritesTrigger() int {
	return int(C.rocksdb_options_get_level0_slowdown_writes_trigger(opts.c))
}

// SetLevel0StopWritesTrigger sets the maximum number of level-0 files.
// We stop writes at this point.
//
// Default: 36
func (opts *Options) SetLevel0StopWritesTrigger(value int) {
	C.rocksdb_options_set_level0_stop_writes_trigger(opts.c, C.int(value))
}

// GetLevel0StopWritesTrigger gets the maximum number of level-0 files.
// We stop writes at this point.
func (opts *Options) GetLevel0StopWritesTrigger() int {
	return int(C.rocksdb_options_get_level0_stop_writes_trigger(opts.c))
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
//
// Default: 1MB
func (opts *Options) SetTargetFileSizeBase(value uint64) {
	C.rocksdb_options_set_target_file_size_base(opts.c, C.uint64_t(value))
}

// GetTargetFileSizeBase gets the target file size base for compaction.
func (opts *Options) GetTargetFileSizeBase() uint64 {
	return uint64(C.rocksdb_options_get_target_file_size_base(opts.c))
}

// SetTargetFileSizeMultiplier sets the target file size multiplier for compaction.
//
// Default: 1
func (opts *Options) SetTargetFileSizeMultiplier(value int) {
	C.rocksdb_options_set_target_file_size_multiplier(opts.c, C.int(value))
}

// GetTargetFileSizeMultiplier gets the target file size multiplier for compaction.
func (opts *Options) GetTargetFileSizeMultiplier() int {
	return int(C.rocksdb_options_get_target_file_size_multiplier(opts.c))
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
//
// Default: 10MB
func (opts *Options) SetMaxBytesForLevelBase(value uint64) {
	C.rocksdb_options_set_max_bytes_for_level_base(opts.c, C.uint64_t(value))
}

// GetMaxBytesForLevelBase gets the maximum total data size for a level.
func (opts *Options) GetMaxBytesForLevelBase() uint64 {
	return uint64(C.rocksdb_options_get_max_bytes_for_level_base(opts.c))
}

// SetMaxBytesForLevelMultiplier sets the max bytes for level multiplier.
//
// Default: 10
func (opts *Options) SetMaxBytesForLevelMultiplier(value float64) {
	C.rocksdb_options_set_max_bytes_for_level_multiplier(opts.c, C.double(value))
}

// GetMaxBytesForLevelMultiplier gets the max bytes for level multiplier.
func (opts *Options) GetMaxBytesForLevelMultiplier() float64 {
	return float64(C.rocksdb_options_get_max_bytes_for_level_multiplier(opts.c))
}

// SetLevelCompactionDynamicLevelBytes specifies whether to pick
// target size of each level dynamically.
//
// We will pick a base level b >= 1. L0 will be directly merged into level b,
// instead of always into level 1. Level 1 to b-1 need to be empty.
// We try to pick b and its target size so that
//  1. target size is in the range of
//     (max_bytes_for_level_base / max_bytes_for_level_multiplier,
//     max_bytes_for_level_base]
//  2. target size of the last level (level num_levels-1) equals to extra size
//     of the level.
//
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

// GetLevelCompactionDynamicLevelBytes checks if level_compaction_dynamic_level_bytes option
// is set.
func (opts *Options) GetLevelCompactionDynamicLevelBytes() bool {
	return charToBool(C.rocksdb_options_get_level_compaction_dynamic_level_bytes(opts.c))
}

// SetMaxCompactionBytes sets the maximum number of bytes in all compacted files.
// We try to limit number of bytes in one compaction to be lower than this
// threshold. But it's not guaranteed.
// Value 0 will be sanitized.
//
// Default: result.target_file_size_base * 25
func (opts *Options) SetMaxCompactionBytes(value uint64) {
	C.rocksdb_options_set_max_compaction_bytes(opts.c, C.uint64_t(value))
}

// GetMaxCompactionBytes returns the maximum number of bytes in all compacted files.
// We try to limit number of bytes in one compaction to be lower than this
// threshold. But it's not guaranteed.
func (opts *Options) GetMaxCompactionBytes() uint64 {
	return uint64(C.rocksdb_options_get_max_compaction_bytes(opts.c))
}

// SetSoftPendingCompactionBytesLimit sets the threshold at which
// all writes will be slowed down to at least delayed_write_rate if estimated
// bytes needed to be compaction exceed this threshold.
//
// Default: 64GB
func (opts *Options) SetSoftPendingCompactionBytesLimit(value uint64) {
	C.rocksdb_options_set_soft_pending_compaction_bytes_limit(opts.c, C.size_t(value))
}

// GetSoftPendingCompactionBytesLimit returns the threshold at which
// all writes will be slowed down to at least delayed_write_rate if estimated
// bytes needed to be compaction exceed this threshold.
func (opts *Options) GetSoftPendingCompactionBytesLimit() uint64 {
	return uint64(C.rocksdb_options_get_soft_pending_compaction_bytes_limit(opts.c))
}

// SetHardPendingCompactionBytesLimit sets the bytes threshold at which
// all writes are stopped if estimated bytes needed to be compaction exceed
// this threshold.
//
// Default: 256GB
func (opts *Options) SetHardPendingCompactionBytesLimit(value uint64) {
	C.rocksdb_options_set_hard_pending_compaction_bytes_limit(opts.c, C.size_t(value))
}

// GetHardPendingCompactionBytesLimit returns the threshold at which
// all writes will be slowed down to at least delayed_write_rate if estimated
// bytes needed to be compaction exceed this threshold.
func (opts *Options) GetHardPendingCompactionBytesLimit() uint64 {
	return uint64(C.rocksdb_options_get_hard_pending_compaction_bytes_limit(opts.c))
}

// SetMaxBytesForLevelMultiplierAdditional sets different max-size multipliers
// for different levels.
//
// These are multiplied by max_bytes_for_level_multiplier to arrive
// at the max-size of each level.
//
// Default: 1 for each level
func (opts *Options) SetMaxBytesForLevelMultiplierAdditional(value []int) {
	if n := len(value); n > 0 {
		cLevels := make([]C.int, n)
		for i, v := range value {
			cLevels[i] = C.int(v)
		}
		C.rocksdb_options_set_max_bytes_for_level_multiplier_additional(opts.c, &cLevels[0], C.size_t(len(value)))
	}
}

// SetUseFsync enable/disable fsync.
//
// If true, then every store to stable storage will issue a fsync.
// If false, then every store to stable storage will issue a fdatasync.
// This parameter should be set to true while storing data to
// filesystem like ext3 that can lose files after a reboot.
// Default: false
func (opts *Options) SetUseFsync(value bool) {
	C.rocksdb_options_set_use_fsync(opts.c, C.int(boolToChar(value)))
}

// UseFsync returns fsync setting.
func (opts *Options) UseFsync() bool {
	return C.rocksdb_options_get_use_fsync(opts.c) != 0
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
	C.rocksdb_options_set_db_log_dir(opts.c, cvalue)
	C.free(unsafe.Pointer(cvalue))
}

// SetWalDir specifies the absolute dir path for write-ahead logs (WAL).
//
// If it is empty, the log files will be in the same dir as data.
// If it is non empty, the log files will be in the specified dir,
// When destroying the db, all log files and the dir itopts is deleted.
// Default: empty
func (opts *Options) SetWalDir(value string) {
	cvalue := C.CString(value)
	C.rocksdb_options_set_wal_dir(opts.c, cvalue)
	C.free(unsafe.Pointer(cvalue))
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

// GetDeleteObsoleteFilesPeriodMicros returns the periodicity
// when obsolete files get deleted.
func (opts *Options) GetDeleteObsoleteFilesPeriodMicros() uint64 {
	return uint64(C.rocksdb_options_get_delete_obsolete_files_period_micros(opts.c))
}

// SetMaxBackgroundCompactions sets the maximum number of
// concurrent background compaction jobs, submitted to
// the default LOW priority thread pool
// Default: 1
//
// Deprecated: RocksDB automatically decides this based on the
// value of max_background_jobs. For backwards compatibility we will set
// `max_background_jobs = max_background_compactions + max_background_flushes`
// in the case where user sets at least one of `max_background_compactions` or
// `max_background_flushes` (we replace -1 by 1 in case one option is unset).
func (opts *Options) SetMaxBackgroundCompactions(value int) {
	C.rocksdb_options_set_max_background_compactions(opts.c, C.int(value))
}

// GetMaxBackgroundCompactions returns maximum number of concurrent background compaction jobs setting.
func (opts *Options) GetMaxBackgroundCompactions() int {
	return int(C.rocksdb_options_get_max_background_compactions(opts.c))
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
//
// Deprecated: RocksDB automatically decides this based on the
// value of max_background_jobs. For backwards compatibility we will set
// `max_background_jobs = max_background_compactions + max_background_flushes`
// in the case where user sets at least one of `max_background_compactions` or
// `max_background_flushes`.
func (opts *Options) SetMaxBackgroundFlushes(value int) {
	C.rocksdb_options_set_max_background_flushes(opts.c, C.int(value))
}

// GetMaxBackgroundFlushes returns the maximum number of concurrent background
// memtable flush jobs setting.
func (opts *Options) GetMaxBackgroundFlushes() int {
	return int(C.rocksdb_options_get_max_background_flushes(opts.c))
}

// SetMaxLogFileSize sets the maximum size of the info log file.
//
// If the log file is larger than `max_log_file_size`, a new info log
// file will be created.
// If max_log_file_size == 0, all logs will be written to one log file.
// Default: 0
func (opts *Options) SetMaxLogFileSize(value uint64) {
	C.rocksdb_options_set_max_log_file_size(opts.c, C.size_t(value))
}

// GetMaxLogFileSize returns setting for maximum size of the info log file.
func (opts *Options) GetMaxLogFileSize() uint64 {
	return uint64(C.rocksdb_options_get_max_log_file_size(opts.c))
}

// SetLogFileTimeToRoll sets the time for the info log file to roll (in seconds).
//
// If specified with non-zero value, log file will be rolled
// if it has been active longer than `log_file_time_to_roll`.
// Default: 0 (disabled)
func (opts *Options) SetLogFileTimeToRoll(value uint64) {
	C.rocksdb_options_set_log_file_time_to_roll(opts.c, C.size_t(value))
}

// GetLogFileTimeToRoll returns the time for info log file to roll (in seconds).
func (opts *Options) GetLogFileTimeToRoll() uint64 {
	return uint64(C.rocksdb_options_get_log_file_time_to_roll(opts.c))
}

// SetKeepLogFileNum sets the maximum info log files to be kept.
// Default: 1000
func (opts *Options) SetKeepLogFileNum(value uint) {
	C.rocksdb_options_set_keep_log_file_num(opts.c, C.size_t(value))
}

// GetKeepLogFileNum return setting for maximum info log files to be kept.
func (opts *Options) GetKeepLogFileNum() uint {
	return uint(C.rocksdb_options_get_keep_log_file_num(opts.c))
}

// SetMaxManifestFileSize sets the maximum manifest file size until is rolled over.
// The older manifest file be deleted.
// Default: MAX_INT so that roll-over does not take place.
func (opts *Options) SetMaxManifestFileSize(value uint64) {
	C.rocksdb_options_set_max_manifest_file_size(opts.c, C.size_t(value))
}

// GetMaxManifestFileSize returns the maximum manifest file size until is rolled over.
// The older manifest file be deleted.
func (opts *Options) GetMaxManifestFileSize() uint64 {
	return uint64(C.rocksdb_options_get_max_manifest_file_size(opts.c))
}

// SetTableCacheNumshardbits sets the number of shards used for table cache.
// Default: 4
func (opts *Options) SetTableCacheNumshardbits(value int) {
	C.rocksdb_options_set_table_cache_numshardbits(opts.c, C.int(value))
}

// GetTableCacheNumshardbits returns the number of shards used for table cache.
func (opts *Options) GetTableCacheNumshardbits() int {
	return int(C.rocksdb_options_get_table_cache_numshardbits(opts.c))
}

// SetArenaBlockSize sets the size of one block in arena memory allocation.
//
// If <= 0, a proper value is automatically calculated (usually 1/10 of
// writer_buffer_size).
//
// Default: 0
func (opts *Options) SetArenaBlockSize(value uint64) {
	C.rocksdb_options_set_arena_block_size(opts.c, C.size_t(value))
}

// GetArenaBlockSize returns the size of one block in arena memory allocation.
func (opts *Options) GetArenaBlockSize() uint64 {
	return uint64(C.rocksdb_options_get_arena_block_size(opts.c))
}

// SetDisableAutoCompactions enable/disable automatic compactions.
//
// Manual compactions can still be issued on this database.
//
// Default: false
func (opts *Options) SetDisableAutoCompactions(value bool) {
	C.rocksdb_options_set_disable_auto_compactions(opts.c, C.int(boolToChar(value)))
}

// DisabledAutoCompactions returns if automatic compactions is disabled.
func (opts *Options) DisabledAutoCompactions() bool {
	return charToBool(C.rocksdb_options_get_disable_auto_compactions(opts.c))
}

// SetWALRecoveryMode sets the recovery mode.
// Recovery mode to control the consistency while replaying WAL.
//
// Default: PointInTimeRecovery
func (opts *Options) SetWALRecoveryMode(mode WALRecoveryMode) {
	C.rocksdb_options_set_wal_recovery_mode(opts.c, C.int(mode))
}

// GetWALRecoveryMode returns the recovery mode.
func (opts *Options) GetWALRecoveryMode() WALRecoveryMode {
	return WALRecoveryMode(C.rocksdb_options_get_wal_recovery_mode(opts.c))
}

// SetWALTtlSeconds sets the WAL ttl in seconds.
//
// The following two options affect how archived logs will be deleted.
//  1. If both set to 0, logs will be deleted asap and will not get into
//     the archive.
//  2. If wal_ttl_seconds is 0 and wal_size_limit_mb is not 0,
//     WAL files will be checked every 10 min and if total size is greater
//     then wal_size_limit_mb, they will be deleted starting with the
//     earliest until size_limit is met. All empty files will be deleted.
//  3. If wal_ttl_seconds is not 0 and wall_size_limit_mb is 0, then
//     WAL files will be checked every wal_ttl_seconds / 2 and those that
//     are older than wal_ttl_seconds will be deleted.
//  4. If both are not 0, WAL files will be checked every 10 min and both
//     checks will be performed with ttl being first.
//
// Default: 0
func (opts *Options) SetWALTtlSeconds(value uint64) {
	C.rocksdb_options_set_WAL_ttl_seconds(opts.c, C.uint64_t(value))
}

// GetWALTtlSeconds returns WAL ttl in seconds.
func (opts *Options) GetWALTtlSeconds() uint64 {
	return uint64(C.rocksdb_options_get_WAL_ttl_seconds(opts.c))
}

// SetWalSizeLimitMb sets the WAL size limit in MB.
//
// If total size of WAL files is greater then wal_size_limit_mb,
// they will be deleted starting with the earliest until size_limit is met.
//
// Default: 0
func (opts *Options) SetWalSizeLimitMb(value uint64) {
	C.rocksdb_options_set_WAL_size_limit_MB(opts.c, C.uint64_t(value))
}

// GetWalSizeLimitMb returns the WAL size limit in MB.
func (opts *Options) GetWalSizeLimitMb() uint64 {
	return uint64(C.rocksdb_options_get_WAL_size_limit_MB(opts.c))
}

// SetEnablePipelinedWrite enables pipelined write.
//
// By default, a single write thread queue is maintained. The thread gets
// to the head of the queue becomes write batch group leader and responsible
// for writing to WAL and memtable for the batch group.
//
// If enable_pipelined_write is true, separate write thread queue is
// maintained for WAL write and memtable write. A write thread first enter WAL
// writer queue and then memtable writer queue. Pending thread on the WAL
// writer queue thus only have to wait for previous writers to finish their
// WAL writing but not the memtable writing. Enabling the feature may improve
// write throughput and reduce latency of the prepare phase of two-phase
// commit.
//
// Default: false
func (opts *Options) SetEnablePipelinedWrite(value bool) {
	C.rocksdb_options_set_enable_pipelined_write(opts.c, boolToChar(value))
}

// EnabledPipelinedWrite check if enable_pipelined_write is turned on.
func (opts *Options) EnabledPipelinedWrite() bool {
	return charToBool(C.rocksdb_options_get_enable_pipelined_write(opts.c))
}

// SetManifestPreallocationSize sets the number of bytes
// to preallocate (via fallocate) the manifest files.
//
// Default is 4MB, which is reasonable to reduce random IO
// as well as prevent overallocation for mounts that preallocate
// large amounts of data (such as xfs's allocsize option).
func (opts *Options) SetManifestPreallocationSize(value uint64) {
	C.rocksdb_options_set_manifest_preallocation_size(opts.c, C.size_t(value))
}

// GetManifestPreallocationSize returns the number of bytes
// to preallocate (via fallocate) the manifest files.
func (opts *Options) GetManifestPreallocationSize() uint64 {
	return uint64(C.rocksdb_options_get_manifest_preallocation_size(opts.c))
}

// SetAllowMmapReads enable/disable mmap reads for reading sst tables.
// Default: false
func (opts *Options) SetAllowMmapReads(value bool) {
	C.rocksdb_options_set_allow_mmap_reads(opts.c, boolToChar(value))
}

// AllowMmapReads returns setting for enable/disable mmap reads for sst tables.
func (opts *Options) AllowMmapReads() bool {
	return charToBool(C.rocksdb_options_get_allow_mmap_reads(opts.c))
}

// SetAllowMmapWrites enable/disable mmap writes for writing sst tables.
// Default: false
func (opts *Options) SetAllowMmapWrites(value bool) {
	C.rocksdb_options_set_allow_mmap_writes(opts.c, boolToChar(value))
}

// AllowMmapWrites returns setting for enable/disable mmap writes for sst tables.
func (opts *Options) AllowMmapWrites() bool {
	return charToBool(C.rocksdb_options_get_allow_mmap_writes(opts.c))
}

// SetUseDirectReads enable/disable direct I/O mode (O_DIRECT) for reads
// Default: false
func (opts *Options) SetUseDirectReads(value bool) {
	C.rocksdb_options_set_use_direct_reads(opts.c, boolToChar(value))
}

// UseDirectReads returns setting for enable/disable direct I/O mode (O_DIRECT) for reads
func (opts *Options) UseDirectReads() bool {
	return charToBool(C.rocksdb_options_get_use_direct_reads(opts.c))
}

// SetUseDirectIOForFlushAndCompaction enable/disable direct I/O mode (O_DIRECT) for both reads and writes in background flush and compactions
// When true, new_table_reader_for_compaction_inputs is forced to true.
// Default: false
func (opts *Options) SetUseDirectIOForFlushAndCompaction(value bool) {
	C.rocksdb_options_set_use_direct_io_for_flush_and_compaction(opts.c, boolToChar(value))
}

// UseDirectIOForFlushAndCompaction returns setting for enable/disable direct I/O mode (O_DIRECT)
// for both reads and writes in background flush and compactions
func (opts *Options) UseDirectIOForFlushAndCompaction() bool {
	return charToBool(C.rocksdb_options_get_use_direct_io_for_flush_and_compaction(opts.c))
}

// SetIsFdCloseOnExec enable/dsiable child process inherit open files.
// Default: true
func (opts *Options) SetIsFdCloseOnExec(value bool) {
	C.rocksdb_options_set_is_fd_close_on_exec(opts.c, boolToChar(value))
}

// IsFdCloseOnExec returns setting for enable/dsiable child process inherit open files.
func (opts *Options) IsFdCloseOnExec() bool {
	return charToBool(C.rocksdb_options_get_is_fd_close_on_exec(opts.c))
}

// SetStatsDumpPeriodSec sets the stats dump period in seconds.
//
// If not zero, dump stats to LOG every stats_dump_period_sec
// Default: 3600 (1 hour)
func (opts *Options) SetStatsDumpPeriodSec(value uint) {
	C.rocksdb_options_set_stats_dump_period_sec(opts.c, C.uint(value))
}

// GetStatsDumpPeriodSec returns the stats dump period in seconds.
func (opts *Options) GetStatsDumpPeriodSec() uint {
	return uint(C.rocksdb_options_get_stats_dump_period_sec(opts.c))
}

// SetStatsPersistPeriodSec if not zero, dump rocksdb.stats to RocksDB every stats_persist_period_sec
//
// Default: 600
func (opts *Options) SetStatsPersistPeriodSec(value uint) {
	C.rocksdb_options_set_stats_persist_period_sec(opts.c, C.uint(value))
}

// GetStatsPersistPeriodSec returns number of sec that RocksDB periodically dump stats.
func (opts *Options) GetStatsPersistPeriodSec() uint {
	return uint(C.rocksdb_options_get_stats_persist_period_sec(opts.c))
}

// SetAdviseRandomOnOpen specifies whether we will hint the underlying
// file system that the file access pattern is random, when a sst file is opened.
// Default: true
func (opts *Options) SetAdviseRandomOnOpen(value bool) {
	C.rocksdb_options_set_advise_random_on_open(opts.c, boolToChar(value))
}

// AdviseRandomOnOpen returns whether we will hint the underlying
// file system that the file access pattern is random, when a sst file is opened.
func (opts *Options) AdviseRandomOnOpen() bool {
	return charToBool(C.rocksdb_options_get_advise_random_on_open(opts.c))
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
func (opts *Options) SetDbWriteBufferSize(value uint64) {
	C.rocksdb_options_set_db_write_buffer_size(opts.c, C.size_t(value))
}

// GetDbWriteBufferSize gets db_write_buffer_size which is set in options
func (opts *Options) GetDbWriteBufferSize() uint64 {
	return uint64(C.rocksdb_options_get_db_write_buffer_size(opts.c))
}

// SetAccessHintOnCompactionStart specifies the file access pattern
// once a compaction is started.
//
// It will be applied to all input files of a compaction.
// Default: NormalCompactionAccessPattern
func (opts *Options) SetAccessHintOnCompactionStart(value CompactionAccessPattern) {
	C.rocksdb_options_set_access_hint_on_compaction_start(opts.c, C.int(value))
}

// GetAccessHintOnCompactionStart returns the file access pattern
// once a compaction is started.
func (opts *Options) GetAccessHintOnCompactionStart() CompactionAccessPattern {
	return CompactionAccessPattern(C.rocksdb_options_get_access_hint_on_compaction_start(opts.c))
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

// UseAdaptiveMutex returns setting for enable/disable adaptive mutex, which spins
// in the user space before resorting to kernel.
func (opts *Options) UseAdaptiveMutex() bool {
	return charToBool(C.rocksdb_options_get_use_adaptive_mutex(opts.c))
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

// GetBytesPerSync return setting for bytes (size) per sync.
func (opts *Options) GetBytesPerSync() uint64 {
	return uint64(C.rocksdb_options_get_bytes_per_sync(opts.c))
}

// SetCompactionStyle sets compaction style.
//
// Default: LevelCompactionStyle
func (opts *Options) SetCompactionStyle(value CompactionStyle) {
	C.rocksdb_options_set_compaction_style(opts.c, C.int(value))
}

// GetCompactionStyle returns compaction style.
func (opts *Options) GetCompactionStyle() CompactionStyle {
	return CompactionStyle(C.rocksdb_options_get_compaction_style(opts.c))
}

// SetUniversalCompactionOptions sets the options needed
// to support Universal Style compactions.
//
// Note: move semantic. Don't use universal compaction options after calling
// this function
//
// Default: nil
func (opts *Options) SetUniversalCompactionOptions(value *UniversalCompactionOptions) {
	C.rocksdb_options_set_universal_compaction_options(opts.c, value.c)
	value.Destroy()
}

// SetFIFOCompactionOptions sets the options for FIFO compaction style.
//
// Note: move semantic. Don't use fifo compaction options after calling
// this function
//
// Default: nil
func (opts *Options) SetFIFOCompactionOptions(value *FIFOCompactionOptions) {
	C.rocksdb_options_set_fifo_compaction_options(opts.c, value.c)
	value.Destroy()
}

// GetStatisticsString returns the statistics as a string.
func (opts *Options) GetStatisticsString() (stats string) {
	cValue := C.rocksdb_options_statistics_get_string(opts.c)
	stats = C.GoString(cValue)
	C.rocksdb_free(unsafe.Pointer(cValue))
	return
}

func (opts *Options) GetTickerCount(tickerType TickerType) uint64 {
	return uint64(C.rocksdb_options_statistics_get_ticker_count(opts.c, C.uint32_t(tickerType)))
}

func (opts *Options) GetHistogramData(histogramType HistogramType) (histogram HistogramData) {
	hData := C.rocksdb_statistics_histogram_data_create()
	C.rocksdb_options_statistics_get_histogram_data(opts.c, C.uint32_t(histogramType), hData)

	histogram.Median = float64(C.rocksdb_statistics_histogram_data_get_median(hData))
	histogram.P95 = float64(C.rocksdb_statistics_histogram_data_get_p95(hData))
	histogram.P99 = float64(C.rocksdb_statistics_histogram_data_get_p99(hData))
	histogram.Average = float64(C.rocksdb_statistics_histogram_data_get_average(hData))
	histogram.StdDev = float64(C.rocksdb_statistics_histogram_data_get_std_dev(hData))
	histogram.Max = float64(C.rocksdb_statistics_histogram_data_get_max(hData))
	histogram.Min = float64(C.rocksdb_statistics_histogram_data_get_min(hData))
	histogram.Count = uint64(C.rocksdb_statistics_histogram_data_get_count(hData))
	histogram.Sum = uint64(C.rocksdb_statistics_histogram_data_get_sum(hData))

	C.rocksdb_statistics_histogram_data_destroy(hData)

	return
}

// SetRateLimiter sets the rate limiter of the options.
// Use to control write rate of flush and compaction. Flush has higher
// priority than compaction. Rate limiting is disabled if nullptr.
// If rate limiter is enabled, bytes_per_sync is set to 1MB by default.
//
// Note: move semantic. Don't use rate limiter after calling
// this function
//
// Default: nil
func (opts *Options) SetRateLimiter(rateLimiter *RateLimiter) {
	C.rocksdb_options_set_ratelimiter(opts.c, rateLimiter.c)
	rateLimiter.Destroy()
}

// SetAtomicFlush if true, RocksDB supports flushing multiple column families
// and committing their results atomically to MANIFEST. Note that it is not
// necessary to set atomic_flush to true if WAL is always enabled since WAL
// allows the database to be restored to the last persistent state in WAL.
// This option is useful when there are column families with writes NOT
// protected by WAL.
// For manual flush, application has to specify which column families to
// flush atomically in DB::Flush.
// For auto-triggered flush, RocksDB atomically flushes ALL column families.
//
// Currently, any WAL-enabled writes after atomic flush may be replayed
// independently if the process crashes later and tries to recover.
func (opts *Options) SetAtomicFlush(value bool) {
	C.rocksdb_options_set_atomic_flush(opts.c, boolToChar(value))
}

// IsAtomicFlush returns setting for atomic flushing.
// If true, RocksDB supports flushing multiple column families and committing
// their results atomically to MANIFEST. Note that it is not
// necessary to set atomic_flush to true if WAL is always enabled since WAL
// allows the database to be restored to the last persistent state in WAL.
// This option is useful when there are column families with writes NOT
// protected by WAL.
// For manual flush, application has to specify which column families to
// flush atomically in DB::Flush.
// For auto-triggered flush, RocksDB atomically flushes ALL column families.
//
// Currently, any WAL-enabled writes after atomic flush may be replayed
// independently if the process crashes later and tries to recover.
func (opts *Options) IsAtomicFlush() bool {
	return charToBool(C.rocksdb_options_get_atomic_flush(opts.c))
}

// SetRowCache set global cache for table-level rows.
//
// Default: nil (disabled)
// Not supported in ROCKSDB_LITE mode!
func (opts *Options) SetRowCache(cache *Cache) {
	C.rocksdb_options_set_row_cache(opts.c, cache.c)
}

// AddCompactOnDeletionCollectorFactory marks a SST
// file as need-compaction when it observe at least "D" deletion
// entries in any "N" consecutive entries or the ratio of tombstone
// entries in the whole file >= the specified deletion ratio.
func (opts *Options) AddCompactOnDeletionCollectorFactory(windowSize, numDelsTrigger uint) {
	C.rocksdb_options_add_compact_on_deletion_collector_factory(opts.c, C.size_t(windowSize), C.size_t(numDelsTrigger))
}

// AddCompactOnDeletionCollectorFactoryWithRatio similar to AddCompactOnDeletionCollectorFactory
// with specific deletion ratio.
func (opts *Options) AddCompactOnDeletionCollectorFactoryWithRatio(windowSize, numDelsTrigger uint, deletionRatio float64) {
	C.rocksdb_options_add_compact_on_deletion_collector_factory_del_ratio(opts.c, C.size_t(windowSize), C.size_t(numDelsTrigger), C.double(deletionRatio))
}

// SetManualWALFlush if true WAL is not flushed automatically after each write. Instead it
// relies on manual invocation of db.FlushWAL to write the WAL buffer to its
// file.
//
// Default: false
func (opts *Options) SetManualWALFlush(v bool) {
	C.rocksdb_options_set_manual_wal_flush(opts.c, boolToChar(v))
}

// IsManualWALFlush returns true if WAL is not flushed automatically after each write.
func (opts *Options) IsManualWALFlush() bool {
	return charToBool(C.rocksdb_options_get_manual_wal_flush(opts.c))
}

// SetWALCompression sets compression type for WAL.
//
// Note: this feature is WORK IN PROGRESS
// If enabled WAL records will be compressed before they are written.
// Only zstd is supported. Compressed WAL records will be read in supported
// versions regardless of the wal_compression settings.
//
// Default: no compression
func (opts *Options) SetWALCompression(cType CompressionType) {
	C.rocksdb_options_set_wal_compression(opts.c, C.int(cType))
}

// GetWALCompression returns compression type of WAL.
func (opts *Options) GetWALCompression() CompressionType {
	return CompressionType(C.rocksdb_options_get_wal_compression(opts.c))
}

// SetMaxSequentialSkipInIterations specifies whether an iteration->Next()
// sequentially skips over keys with the same user-key or not.
//
// This number specifies the number of keys (with the same userkey)
// that will be sequentially skipped before a reseek is issued.
//
// Default: 8
func (opts *Options) SetMaxSequentialSkipInIterations(value uint64) {
	C.rocksdb_options_set_max_sequential_skip_in_iterations(opts.c, C.uint64_t(value))
}

// GetMaxSequentialSkipInIterations returns the number of keys (with the same userkey)
// that will be sequentially skipped before a reseek is issued.
func (opts *Options) GetMaxSequentialSkipInIterations() uint64 {
	return uint64(C.rocksdb_options_get_max_sequential_skip_in_iterations(opts.c))
}

// SetInplaceUpdateSupport enable/disable thread-safe inplace updates.
//
// Requires updates if
// * key exists in current memtable
// * new sizeof(new_value) <= sizeof(old_value)
// * old_value for that key is a put i.e. kTypeValue
//
// Default: false.
func (opts *Options) SetInplaceUpdateSupport(value bool) {
	C.rocksdb_options_set_inplace_update_support(opts.c, boolToChar(value))
}

// InplaceUpdateSupport returns setting for enable/disable
// thread-safe inplace updates.
func (opts *Options) InplaceUpdateSupport() bool {
	return charToBool(C.rocksdb_options_get_inplace_update_support(opts.c))
}

// SetInplaceUpdateNumLocks sets the number of locks used for inplace update.
//
// Default: 10000, if inplace_update_support = true, else 0.
func (opts *Options) SetInplaceUpdateNumLocks(value uint) {
	C.rocksdb_options_set_inplace_update_num_locks(opts.c, C.size_t(value))
}

// GetInplaceUpdateNumLocks returns number of locks used for inplace upddate.
func (opts *Options) GetInplaceUpdateNumLocks() uint {
	return uint(C.rocksdb_options_get_inplace_update_num_locks(opts.c))
}

// SetMemtableHugePageSize sets the page size for huge page for
// arena used by the memtable.
// If <=0, it won't allocate from huge page but from malloc.
// Users are responsible to reserve huge pages for it to be allocated. For
// example:
//
//	sysctl -w vm.nr_hugepages=20
//
// See linux doc Documentation/vm/hugetlbpage.txt
// If there isn't enough free huge page available, it will fall back to
// malloc.
//
// Dynamically changeable through SetOptions() API
func (opts *Options) SetMemtableHugePageSize(value uint64) {
	C.rocksdb_options_set_memtable_huge_page_size(opts.c, C.size_t(value))
}

// GetMemtableHugePageSize returns the page size for huge page for
// arena used by the memtable.
func (opts *Options) GetMemtableHugePageSize() uint64 {
	return uint64(C.rocksdb_options_get_memtable_huge_page_size(opts.c))
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

// GetBloomLocality returns control locality of bloom filter probes to improve cache miss rate.
// This option only applies to memtable prefix bloom and plaintable
// prefix bloom. It essentially limits the max number of cache lines each
// bloom filter check can touch.
// This optimization is turned off when set to 0. The number should never
// be greater than number of probes. This option can boost performance
// for in-memory workload but should use with care since it can cause
// higher false positive rate.
func (opts *Options) GetBloomLocality() uint32 {
	return uint32(C.rocksdb_options_get_bloom_locality(opts.c))
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
func (opts *Options) SetMaxSuccessiveMerges(value uint) {
	C.rocksdb_options_set_max_successive_merges(opts.c, C.size_t(value))
}

// GetMaxSuccessiveMerges returns the maximum number of
// successive merge operations on a key in the memtable.
//
// When a merge operation is added to the memtable and the maximum number of
// successive merges is reached, the value of the key will be calculated and
// inserted into the memtable instead of the merge operation. This will
// ensure that there are never more than max_successive_merges merge
// operations in the memtable.
func (opts *Options) GetMaxSuccessiveMerges() uint {
	return uint(C.rocksdb_options_get_max_successive_merges(opts.c))
}

// EnableStatistics enable statistics.
func (opts *Options) EnableStatistics() {
	C.rocksdb_options_enable_statistics(opts.c)
}

// SetTTL sets TTL. This option has different meanings for different compaction styles:
//
// Leveled: Non-bottom-level files with all keys older than TTL will go
//
//	through the compaction process. This usually happens in a cascading
//	way so that those entries will be compacted to bottommost level/file.
//	The feature is used to remove stale entries that have been deleted or
//	updated from the file system.
//
// FIFO: Files with all keys older than TTL will be deleted. TTL is only
//
//	supported if option max_open_files is set to -1.
//
// Universal: users should only set the option `periodic_compaction_seconds`
//
//	below instead. For backward compatibility, this option has the same
//	meaning as `periodic_compaction_seconds`. See more in comments for
//	`periodic_compaction_seconds` on the interaction between these two
//	options.
//
// This option only supports block based table format for any compaction
// style.
//
// unit: seconds. Ex: 1 day = 1 * 24 * 60 * 60
// 0 means disabling.
// UINT64_MAX - 1 (0xfffffffffffffffe) is special flag to allow RocksDB to
// pick default.
//
// Default: 30 days if using block based table. 0 (disable) otherwise.
//
// Dynamically changeable through SetOptions() API
// Note that dynamically changing this option only works for leveled and FIFO
// compaction. For universal compaction, dynamically changing this option has
// no effect, users should dynamically change `periodic_compaction_seconds`
// instead.
func (opts *Options) SetTTL(seconds uint64) {
	C.rocksdb_options_set_ttl(opts.c, C.uint64_t(seconds))
}

// GetTTL gets TTL option.
func (opts *Options) GetTTL() uint64 {
	return uint64(C.rocksdb_options_get_ttl(opts.c))
}

// SetPeriodicCompactionSeconds sets periodic_compaction_seconds option.
//
// This option has different meanings for different compaction styles:
//
// Leveled: files older than `periodic_compaction_seconds` will be picked up
//
//	for compaction and will be re-written to the same level as they were
//	before.
//
// FIFO: not supported. Setting this option has no effect for FIFO compaction.
//
// Universal: when there are files older than `periodic_compaction_seconds`,
//
//	rocksdb will try to do as large a compaction as possible including the
//	last level. Such compaction is only skipped if only last level is to
//	be compacted and no file in last level is older than
//	`periodic_compaction_seconds`. See more in
//	UniversalCompactionBuilder::PickPeriodicCompaction().
//	For backward compatibility, the effective value of this option takes
//	into account the value of option `ttl`. The logic is as follows:
//	- both options are set to 30 days if they have the default value.
//	- if both options are zero, zero is picked. Otherwise, we take the min
//	value among non-zero options values (i.e. takes the stricter limit).
//
// One main use of the feature is to make sure a file goes through compaction
// filters periodically. Users can also use the feature to clear up SST
// files using old format.
//
// A file's age is computed by looking at file_creation_time or creation_time
// table properties in order, if they have valid non-zero values; if not, the
// age is based on the file's last modified time (given by the underlying
// Env).
//
// This option only supports block based table format for any compaction
// style.
//
// unit: seconds. Ex: 7 days = 7 * 24 * 60 * 60
//
// Values:
// 0: Turn off Periodic compactions.
// UINT64_MAX - 1 (0xfffffffffffffffe) is special flag to allow RocksDB to
// pick default.
//
// Default: 30 days if using block based table format + compaction filter +
//
//	leveled compaction or block based table format + universal compaction.
//	0 (disabled) otherwise.
//
// Dynamically changeable through SetOptions() API
func (opts *Options) SetPeriodicCompactionSeconds(v uint64) {
	C.rocksdb_options_set_periodic_compaction_seconds(opts.c, C.uint64_t(v))
}

// GetPeriodicCompactionSeconds gets periodic periodic_compaction_seconds option.
func (opts *Options) GetPeriodicCompactionSeconds() uint64 {
	return uint64(C.rocksdb_options_get_periodic_compaction_seconds(opts.c))
}

// SetStatisticsLevel set statistics level.
func (opts *Options) SetStatisticsLevel(level StatisticsLevel) {
	C.rocksdb_options_set_statistics_level(opts.c, C.int(level))
}

// GetStatisticsLevel get statistics level.
func (opts *Options) GetStatisticsLevel() StatisticsLevel {
	return StatisticsLevel(C.rocksdb_options_get_statistics_level(opts.c))
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
//
//	link lists in the skiplist
func (opts *Options) SetHashSkipListRep(bucketCount uint, skiplistHeight, skiplistBranchingFactor int32) {
	C.rocksdb_options_set_hash_skip_list_rep(
		opts.c,
		C.size_t(bucketCount),
		C.int32_t(skiplistHeight),
		C.int32_t(skiplistBranchingFactor),
	)
}

// SetHashLinkListRep sets a hashed linked list as MemTableRep.
//
// It contains a fixed array of buckets, each pointing to a sorted single
// linked list (null if the bucket is empty).
//
// bucketCount: number of fixed array buckets
func (opts *Options) SetHashLinkListRep(bucketCount uint) {
	C.rocksdb_options_set_hash_link_list_rep(opts.c, C.size_t(bucketCount))
}

// SetPlainTableFactory sets a plain table factory with prefix-only seek.
//
// For this factory, you need to set prefix_extractor properly to make it
// work. Look-up will starts with prefix hash lookup for key prefix. Inside the
// hash bucket found, a binary search is executed for hash conflicts. Finally,
// a linear search is used.
//
// keyLen: plain table has optimization for fix-sized keys, which can be specified via keyLen.
//
// bloomBitsPerKey: the number of bits used for bloom filer per prefix. You may disable it by passing a zero.
//
// hashTableRatio: the desired utilization of the hash table used for prefix hashing.
// hashTableRatio = number of prefixes / #buckets in the hash table
//
// indexSparseness: inside each prefix, need to build one index record for how
// many keys for binary search inside each hash bucket.
//
// hugePageTlbSize: if <=0, allocate hash indexes and blooms from malloc.
// Otherwise from huge page TLB. The user needs to reserve huge pages for it to be allocated, like: sysctl -w vm.nr_hugepages=20
// See linux doc Documentation/vm/hugetlbpage.txt
//
// encodeType: how to encode the keys. See enum EncodingType above for
// the choices. The value will determine how to encode keys
// when writing to a new SST file. This value will be stored
// inside the SST file which will be used when reading from
// the file, which makes it possible for users to choose
// different encoding type when reopening a DB. Files with
// different encoding types can co-exist in the same DB and
// can be read.
//
// fullScanMode: mode for reading the whole file one record by one without
// using the index.
//
// storeIndexInFile: compute plain table index and bloom filter during
// file building and store it in file. When reading file, index will be mapped instead of recomputation.
func (opts *Options) SetPlainTableFactory(
	keyLen uint32,
	bloomBitsPerKey int,
	hashTableRatio float64,
	indexSparseness uint,
	hugePageTlbSize int,
	encodeType EncodingType,
	fullScanMode bool,
	storeIndexInFile bool,
) {
	C.rocksdb_options_set_plain_table_factory(
		opts.c,
		C.uint32_t(keyLen),
		C.int(bloomBitsPerKey),
		C.double(hashTableRatio),
		C.size_t(indexSparseness),
		C.size_t(hugePageTlbSize),
		C.char(encodeType),
		boolToChar(fullScanMode),
		boolToChar(storeIndexInFile),
	)
}

// SetWriteBufferManager binds with a WriteBufferManager.
//
// The memory usage of memtable will report to this object. The same object
// can be passed into multiple DBs and it will track the sum of size of all
// the DBs. If the total size of all live memtables of all the DBs exceeds
// a limit, a flush will be triggered in the next DB to which the next write
// is issued, as long as there is one or more column family not already
// flushing.
//
// If the object is only passed to one DB, the behavior is the same as
// db_write_buffer_size. When write_buffer_manager is set, the value set will
// override db_write_buffer_size.
//
// This feature is disabled by default. Specify a non-zero value
// to enable it.
func (opts *Options) SetWriteBufferManager(wbm *WriteBufferManager) {
	C.rocksdb_options_set_write_buffer_manager(opts.c, wbm.p)
}

// SetCreateIfMissingColumnFamilies specifies whether the column families
// should be created if they are missing.
func (opts *Options) SetCreateIfMissingColumnFamilies(value bool) {
	C.rocksdb_options_set_create_missing_column_families(opts.c, boolToChar(value))
}

// CreateIfMissingColumnFamilies checks if create_if_missing_cf option is set
func (opts *Options) CreateIfMissingColumnFamilies() bool {
	return charToBool(C.rocksdb_options_get_create_missing_column_families(opts.c))
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
// Default: false
func (opts *Options) SetAllowIngestBehind(value bool) {
	C.rocksdb_options_set_allow_ingest_behind(opts.c, boolToChar(value))
}

// AllowIngestBehind checks if allow_ingest_behind is set
func (opts *Options) AllowIngestBehind() bool {
	return charToBool(C.rocksdb_options_get_allow_ingest_behind(opts.c))
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

// GetMemTablePrefixBloomSizeRatio returns memtable_prefix_bloom_size_ratio.
func (opts *Options) GetMemTablePrefixBloomSizeRatio() float64 {
	return float64(C.rocksdb_options_get_memtable_prefix_bloom_size_ratio(opts.c))
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
	C.rocksdb_options_set_optimize_filters_for_hits(opts.c, C.int(boolToChar(value)))
}

// OptimizeFiltersForHits gets setting for optimize_filters_for_hits.
func (opts *Options) OptimizeFiltersForHits() bool {
	return charToBool(C.rocksdb_options_get_optimize_filters_for_hits(opts.c))
}

// CompactionReadaheadSize if non-zero, we perform bigger reads when doing
// compaction. If you're running RocksDB on spinning disks, you should set
// this to at least 2MB. That way RocksDB's compaction is doing sequential
// instead of random reads.
//
// When non-zero, we also force new_table_reader_for_compaction_inputs to
// true.
//
// Default: 0
//
// Dynamically changeable through SetDBOptions() API.
func (opts *Options) CompactionReadaheadSize(value uint64) {
	C.rocksdb_options_compaction_readahead_size(opts.c, C.size_t(value))
}

// GetCompactionReadaheadSize gets readahead size
func (opts *Options) GetCompactionReadaheadSize() uint64 {
	return uint64(C.rocksdb_options_get_compaction_readahead_size(opts.c))
}

// SetUint64AddMergeOperator set add/merge operator.
func (opts *Options) SetUint64AddMergeOperator() {
	C.rocksdb_options_set_uint64add_merge_operator(opts.c)
}

// SetSkipStatsUpdateOnDBOpen if true, then DB::Open() will not update
// the statistics used to optimize compaction decision by loading table
// properties from many files. Turning off this feature will improve
// DBOpen time especially in disk environment.
//
// Default: false
func (opts *Options) SetSkipStatsUpdateOnDBOpen(value bool) {
	C.rocksdb_options_set_skip_stats_update_on_db_open(opts.c, boolToChar(value))
}

// SkipStatsUpdateOnDBOpen checks if skip_stats_update_on_db_open is set.
func (opts *Options) SkipStatsUpdateOnDBOpen() bool {
	return charToBool(C.rocksdb_options_get_skip_stats_update_on_db_open(opts.c))
}

// SetSkipCheckingSSTFileSizesOnDBOpen skips checking sst file sizes on db openning
//
// Default: false
func (opts *Options) SetSkipCheckingSSTFileSizesOnDBOpen(value bool) {
	C.rocksdb_options_set_skip_checking_sst_file_sizes_on_db_open(opts.c, boolToChar(value))
}

// SkipCheckingSSTFileSizesOnDBOpen checks if skips_checking_sst_file_sizes_on_db_openning is set.
func (opts *Options) SkipCheckingSSTFileSizesOnDBOpen() bool {
	return charToBool(C.rocksdb_options_get_skip_checking_sst_file_sizes_on_db_open(opts.c))
}

/* Blob Options Settings */

// EnableBlobFiles when set, large values (blobs) are written to separate blob files, and
// only pointers to them are stored in SST files. This can reduce write
// amplification for large-value use cases at the cost of introducing a level
// of indirection for reads. See also the options min_blob_size,
// blob_file_size, blob_compression_type, enable_blob_garbage_collection,
// and blob_garbage_collection_age_cutoff below.
//
// Default: false
//
// Dynamically changeable through the API.
func (opts *Options) EnableBlobFiles(value bool) {
	C.rocksdb_options_set_enable_blob_files(opts.c, boolToChar(value))
}

// IsBlobFilesEnabled returns if blob-file setting is enabled.
func (opts *Options) IsBlobFilesEnabled() bool {
	return charToBool(C.rocksdb_options_get_enable_blob_files(opts.c))
}

// SetMinBlogSize sets the size of the smallest value to be stored separately in a blob file.
// Values which have an uncompressed size smaller than this threshold are
// stored alongside the keys in SST files in the usual fashion. A value of
// zero for this option means that all values are stored in blob files. Note
// that enable_blob_files has to be set in order for this option to have any
// effect.
//
// Default: 0
//
// Dynamically changeable through the API.
func (opts *Options) SetMinBlobSize(value uint64) {
	C.rocksdb_options_set_min_blob_size(opts.c, C.uint64_t(value))
}

// GetMinBlobSize returns the size of the smallest value to be stored separately in a blob file.
func (opts *Options) GetMinBlobSize() uint64 {
	return uint64(C.rocksdb_options_get_min_blob_size(opts.c))
}

// SetBlobFileSize sets the size limit for blob files. When writing blob files, a new file is
// opened once this limit is reached. Note that enable_blob_files has to be
// set in order for this option to have any effect.
//
// Default: 256 MB
//
// Dynamically changeable through the API.
func (opts *Options) SetBlobFileSize(value uint64) {
	C.rocksdb_options_set_blob_file_size(opts.c, C.uint64_t(value))
}

// GetBlobFileSize gets the size limit for blob files.
func (opts *Options) GetBlobFileSize() uint64 {
	return uint64(C.rocksdb_options_get_blob_file_size(opts.c))
}

// SetBlobCompressionType sets the compression algorithm to use for large values stored in blob files.
// Note that enable_blob_files has to be set in order for this option to have
// any effect.
//
// Default: no compression
//
// Dynamically changeable through the API.
func (opts *Options) SetBlobCompressionType(compressionType CompressionType) {
	C.rocksdb_options_set_blob_compression_type(opts.c, C.int(compressionType))
}

// GetBlobCompressionType gets the compression algorithm to use for large values stored in blob files.
// Note that enable_blob_files has to be set in order for this option to have
// any effect.
func (opts *Options) GetBlobCompressionType() CompressionType {
	return CompressionType(C.rocksdb_options_get_blob_compression_type(opts.c))
}

// EnableBlobGC toggles garbage collection of blobs. Blob GC is performed as part of
// compaction. Valid blobs residing in blob files older than a cutoff get
// relocated to new files as they are encountered during compaction, which
// makes it possible to clean up blob files once they contain nothing but
// obsolete/garbage blobs. See also blob_garbage_collection_age_cutoff below.
//
// Default: false
//
// Dynamically changeable through the API.
func (opts *Options) EnableBlobGC(value bool) {
	C.rocksdb_options_set_enable_blob_gc(opts.c, boolToChar(value))
}

// IsBlobGCEnabled returns if blob garbage collection is enabled.
func (opts *Options) IsBlobGCEnabled() bool {
	return charToBool(C.rocksdb_options_get_enable_blob_gc(opts.c))
}

// SetBlobGCAgeCutoff sets the cutoff in terms of blob file age for garbage collection. Blobs in
// the oldest N blob files will be relocated when encountered during
// compaction, where N = garbage_collection_cutoff * number_of_blob_files.
// Note that enable_blob_garbage_collection has to be set in order for this
// option to have any effect.
//
// Default: 0.25
//
// Dynamically changeable through the API.
func (opts *Options) SetBlobGCAgeCutoff(value float64) {
	C.rocksdb_options_set_blob_gc_age_cutoff(opts.c, C.double(value))
}

// GetBlobGCAgeCutoff returns the cutoff in terms of blob file age for garbage collection.
func (opts *Options) GetBlobGCAgeCutoff() float64 {
	return float64(C.rocksdb_options_get_blob_gc_age_cutoff(opts.c))
}

// SetBlobGCForceThreshold if the ratio of garbage in the oldest blob files exceeds this threshold,
// targeted compactions are scheduled in order to force garbage collecting
// the blob files in question, assuming they are all eligible based on the
// value of blob_garbage_collection_age_cutoff above. This option is
// currently only supported with leveled compactions.
// Note that enable_blob_garbage_collection has to be set in order for this
// option to have any effect.
//
// Default: 1.0
func (opts *Options) SetBlobGCForceThreshold(val float64) {
	C.rocksdb_options_set_blob_gc_force_threshold(opts.c, C.double(val))
}

// GetBlobGCForceThreshold get the threshold for ratio of garbage in the oldest blob files.
// See also: `SetBlobGCForceThreshold`
//
// Default: 1.0
func (opts *Options) GetBlobGCForceThreshold() float64 {
	return float64(C.rocksdb_options_get_blob_gc_force_threshold(opts.c))
}

// SetBlobCompactionReadaheadSize sets compaction readahead for blob files.
//
// Default: 0
//
// Dynamically changeable through the SetOptions() API.
func (opts *Options) SetBlobCompactionReadaheadSize(val uint64) {
	C.rocksdb_options_set_blob_compaction_readahead_size(opts.c, C.uint64_t(val))
}

// GetBlobCompactionReadaheadSize returns compaction readahead size for blob files.
func (opts *Options) GetBlobCompactionReadaheadSize() uint64 {
	return uint64(C.rocksdb_options_get_blob_compaction_readahead_size(opts.c))
}

// SetBlobFileStartingLevel enables blob files starting from a certain LSM tree level.
//
// For certain use cases that have a mix of short-lived and long-lived values,
// it might make sense to support extracting large values only during
// compactions whose output level is greater than or equal to a specified LSM
// tree level (e.g. compactions into L1/L2/... or above). This could reduce
// the space amplification caused by large values that are turned into garbage
// shortly after being written at the price of some write amplification
// incurred by long-lived values whose extraction to blob files is delayed.
//
// Default: 0
//
// Dynamically changeable through the SetOptions() API
func (opts *Options) SetBlobFileStartingLevel(level int) {
	C.rocksdb_options_set_blob_file_starting_level(opts.c, C.int(level))
}

// GetBlobFileStartingLevel returns blob starting level.
func (opts *Options) GetBlobFileStartingLevel() int {
	return int(C.rocksdb_options_get_blob_file_starting_level(opts.c))
}

// SetBlobCache caches blob.
func (opts *Options) SetBlobCache(cache *Cache) {
	C.rocksdb_options_set_blob_cache(opts.c, cache.c)
}

// SetPrepopulateBlobCache sets strategy for prepopulate blob caching strategy.
//
// If enabled, prepopulate warm/hot blobs which are already in memory into
// blob cache at the time of flush. On a flush, the blob that is in memory (in
// memtables) get flushed to the device. If using Direct IO, additional IO is
// incurred to read this blob back into memory again, which is avoided by
// enabling this option. This further helps if the workload exhibits high
// temporal locality, where most of the reads go to recently written data.
// This also helps in case of the remote file system since it involves network
// traffic and higher latencies.
//
// Default: disabled
//
// Dynamically changeable through this API
func (opts *Options) SetPrepopulateBlobCache(strategy PrepopulateBlob) {
	C.rocksdb_options_set_prepopulate_blob_cache(opts.c, C.int(strategy))
}

// GetPrepopulateBlobCache gets prepopulate blob caching strategy
func (opts *Options) GetPrepopulateBlobCache() PrepopulateBlob {
	return PrepopulateBlob(C.rocksdb_options_get_prepopulate_blob_cache(opts.c))
}

// SetMaxWriteBufferNumberToMaintain sets total maximum number of write buffers
// to maintain in memory including copies of buffers that have already been flushed.
// Unlike max_write_buffer_number, this parameter does not affect flushing.
// This controls the minimum amount of write history that will be available
// in memory for conflict checking when Transactions are used.
//
// When using an OptimisticTransactionDB:
// If this value is too low, some transactions may fail at commit time due
// to not being able to determine whether there were any write conflicts.
//
// When using a TransactionDB:
// If Transaction::SetSnapshot is used, TransactionDB will read either
// in-memory write buffers or SST files to do write-conflict checking.
// Increasing this value can reduce the number of reads to SST files
// done for conflict detection.
//
// Setting this value to 0 will cause write buffers to be freed immediately
// after they are flushed.
// If this value is set to -1, 'max_write_buffer_number' will be used.
//
// Default:
// If using a TransactionDB/OptimisticTransactionDB, the default value will
// be set to the value of 'max_write_buffer_number' if it is not explicitly
// set by the user.  Otherwise, the default is 0.
//
// Deprecated: soon
func (opts *Options) SetMaxWriteBufferNumberToMaintain(value int) {
	C.rocksdb_options_set_max_write_buffer_number_to_maintain(opts.c, C.int(value))
}

// GetMaxWriteBufferNumberToMaintain gets total maximum number of write buffers
// to maintain in memory including copies of buffers that have already been flushed.
// Unlike max_write_buffer_number, this parameter does not affect flushing.
// This controls the minimum amount of write history that will be available
// in memory for conflict checking when Transactions are used.
//
// Deprecated: soon
func (opts *Options) GetMaxWriteBufferNumberToMaintain() int {
	return int(C.rocksdb_options_get_max_write_buffer_number_to_maintain(opts.c))
}

// SetMaxWriteBufferSizeToMaintain is the total maximum size(bytes) of write buffers to maintain in memory
// including copies of buffers that have already been flushed. This parameter
// only affects trimming of flushed buffers and does not affect flushing.
// This controls the maximum amount of write history that will be available
// in memory for conflict checking when Transactions are used. The actual
// size of write history (flushed Memtables) might be higher than this limit
// if further trimming will reduce write history total size below this
// limit. For example, if max_write_buffer_size_to_maintain is set to 64MB,
// and there are three flushed Memtables, with sizes of 32MB, 20MB, 20MB.
// Because trimming the next Memtable of size 20MB will reduce total memory
// usage to 52MB which is below the limit, RocksDB will stop trimming.
//
// When using an OptimisticTransactionDB:
// If this value is too low, some transactions may fail at commit time due
// to not being able to determine whether there were any write conflicts.
//
// When using a TransactionDB:
// If Transaction::SetSnapshot is used, TransactionDB will read either
// in-memory write buffers or SST files to do write-conflict checking.
// Increasing this value can reduce the number of reads to SST files
// done for conflict detection.
//
// Setting this value to 0 will cause write buffers to be freed immediately
// after they are flushed. If this value is set to -1,
// 'max_write_buffer_number * write_buffer_size' will be used.
//
// Default:
// If using a TransactionDB/OptimisticTransactionDB, the default value will
// be set to the value of 'max_write_buffer_number * write_buffer_size'
// if it is not explicitly set by the user.  Otherwise, the default is 0.
func (opts *Options) SetMaxWriteBufferSizeToMaintain(value int64) {
	C.rocksdb_options_set_max_write_buffer_size_to_maintain(opts.c, C.int64_t(value))
}

// GetMaxWriteBufferSizeToMaintain gets the total maximum size(bytes) of write buffers to maintain in memory
// including copies of buffers that have already been flushed. This parameter
// only affects trimming of flushed buffers and does not affect flushing.
// This controls the maximum amount of write history that will be available
// in memory for conflict checking when Transactions are used. The actual
// size of write history (flushed Memtables) might be higher than this limit
// if further trimming will reduce write history total size below this
// limit. For example, if max_write_buffer_size_to_maintain is set to 64MB,
// and there are three flushed Memtables, with sizes of 32MB, 20MB, 20MB.
// Because trimming the next Memtable of size 20MB will reduce total memory
// usage to 52MB which is below the limit, RocksDB will stop trimming.
func (opts *Options) GetMaxWriteBufferSizeToMaintain() int64 {
	return int64(C.rocksdb_options_get_max_write_buffer_size_to_maintain(opts.c))
}

// SetMaxSubcompactions represents the maximum number of threads that will
// concurrently perform a compaction job by breaking it into multiple,
// smaller ones that are run simultaneously.
//
// Default: 1 (i.e. no subcompactions)
func (opts *Options) SetMaxSubcompactions(value uint32) {
	C.rocksdb_options_set_max_subcompactions(opts.c, C.uint32_t(value))
}

// GetMaxSubcompactions gets the maximum number of threads that will
// concurrently perform a compaction job by breaking it into multiple,
// smaller ones that are run simultaneously.
func (opts *Options) GetMaxSubcompactions() uint32 {
	return uint32(C.rocksdb_options_get_max_subcompactions(opts.c))
}

// SetMaxBackgroundJobs maximum number of concurrent background jobs
// (compactions and flushes).
//
// Default: 2
//
// Dynamically changeable through SetDBOptions() API.
func (opts *Options) SetMaxBackgroundJobs(value int) {
	C.rocksdb_options_set_max_background_jobs(opts.c, C.int(value))
}

// GetMaxBackgroundJobs returns maximum number of concurrent background jobs setting.
func (opts *Options) GetMaxBackgroundJobs() int {
	return int(C.rocksdb_options_get_max_background_jobs(opts.c))
}

// SetRecycleLogFileNum if non-zero, we will reuse previously written
// log files for new logs, overwriting the old data. The value
// indicates how many such files we will keep around at any point in
// time for later use. This is more efficient because the blocks
// are already allocated and fdatasync does not need to update
// the inode after each write.
// Default: 0
func (opts *Options) SetRecycleLogFileNum(value uint) {
	C.rocksdb_options_set_recycle_log_file_num(opts.c, C.size_t(value))
}

// GetRecycleLogFileNum returns setting for number of recycling log files.
func (opts *Options) GetRecycleLogFileNum() uint {
	return uint(C.rocksdb_options_get_recycle_log_file_num(opts.c))
}

// SetWALBytesPerSync same as bytes_per_sync, but applies to WAL files.
//
// Default: 0, turned off
//
// Dynamically changeable through SetDBOptions() API.
func (opts *Options) SetWALBytesPerSync(value uint64) {
	C.rocksdb_options_set_wal_bytes_per_sync(opts.c, C.uint64_t(value))
}

// GetWALBytesPerSync same as bytes_per_sync, but applies to WAL files.
func (opts *Options) GetWALBytesPerSync() uint64 {
	return uint64(C.rocksdb_options_get_wal_bytes_per_sync(opts.c))
}

// SetWritableFileMaxBufferSize is the maximum buffer size that is
// used by WritableFileWriter.
// On Windows, we need to maintain an aligned buffer for writes.
// We allow the buffer to grow until it's size hits the limit in buffered
// IO and fix the buffer size when using direct IO to ensure alignment of
// write requests if the logical sector size is unusual
//
// Default: 1024 * 1024 (1 MB)
//
// Dynamically changeable through SetDBOptions() API.
func (opts *Options) SetWritableFileMaxBufferSize(value uint64) {
	C.rocksdb_options_set_writable_file_max_buffer_size(opts.c, C.uint64_t(value))
}

// GetWritableFileMaxBufferSize returns the maximum buffer size that is
// used by WritableFileWriter.
// On Windows, we need to maintain an aligned buffer for writes.
// We allow the buffer to grow until it's size hits the limit in buffered
// IO and fix the buffer size when using direct IO to ensure alignment of
// write requests if the logical sector size is unusual
func (opts *Options) GetWritableFileMaxBufferSize() uint64 {
	return uint64(C.rocksdb_options_get_writable_file_max_buffer_size(opts.c))
}

// SetEnableWriteThreadAdaptiveYield if true, threads synchronizing with
// the write batch group leader will wait for up to write_thread_max_yield_usec
// before blocking on a mutex. This can substantially improve throughput
// for concurrent workloads, regardless of whether allow_concurrent_memtable_write
// is enabled.
//
// Default: true
func (opts *Options) SetEnableWriteThreadAdaptiveYield(value bool) {
	C.rocksdb_options_set_enable_write_thread_adaptive_yield(opts.c, boolToChar(value))
}

// EnabledWriteThreadAdaptiveYield if true, threads synchronizing with
// the write batch group leader will wait for up to write_thread_max_yield_usec
// before blocking on a mutex. This can substantially improve throughput
// for concurrent workloads, regardless of whether allow_concurrent_memtable_write
// is enabled.
func (opts *Options) EnabledWriteThreadAdaptiveYield() bool {
	return charToBool(C.rocksdb_options_get_enable_write_thread_adaptive_yield(opts.c))
}

// SetReportBackgroundIOStats measures IO stats in compactions and
// flushes, if true.
//
// Default: false
//
// Dynamically changeable through SetOptions() API
func (opts *Options) SetReportBackgroundIOStats(value bool) {
	C.rocksdb_options_set_report_bg_io_stats(opts.c, C.int(boolToChar(value)))
}

// ReportBackgroundIOStats returns if measureing IO stats in compactions and
// flushes is turned on.
func (opts *Options) ReportBackgroundIOStats() bool {
	return charToBool(C.rocksdb_options_get_report_bg_io_stats(opts.c))
}

// AvoidUnnecessaryBlockingIO if true, working thread may avoid doing unnecessary and long-latency
// operation (such as deleting obsolete files directly or deleting memtable)
// and will instead schedule a background job to do it.
// Use it if you're latency-sensitive.
//
// If set to true, takes precedence over ReadOptions::background_purge_on_iterator_cleanup.
func (opts *Options) AvoidUnnecessaryBlockingIO(v bool) {
	C.rocksdb_options_set_avoid_unnecessary_blocking_io(opts.c, boolToChar(v))
}

// GetAvoidUnnecessaryBlockingIOFlag returns value of avoid unnecessary blocking io flag.
func (opts *Options) GetAvoidUnnecessaryBlockingIOFlag() bool {
	return charToBool(C.rocksdb_options_get_avoid_unnecessary_blocking_io(opts.c))
}

// SetMempurgeThreshold is experimental function to set mempurge threshold.
//
// It is used to activate or deactive the Mempurge feature (memtable garbage
// collection, which is deactivated by default).
//
// At every flush, the total useful payload (total entries minus garbage entries) is estimated as a ratio
// [useful payload bytes]/[size of a memtable (in bytes)]. This ratio is then
// compared to this `threshold` value:
//   - if ratio<threshold: the flush is replaced by a mempurge operation
//   - else: a regular flush operation takes place.
//
// Threshold values:
//
//	0.0: mempurge deactivated (default).
//	1.0: recommended threshold value.
//	>1.0 : aggressive mempurge.
//	0 < threshold < 1.0: mempurge triggered only for very low useful payload
//	ratios.
func (opts *Options) SetMempurgeThreshold(threshold float64) {
	C.rocksdb_options_set_experimental_mempurge_threshold(opts.c, C.double(threshold))
}

// GetMempurgeThreshold gets current mempurge threshold value.
func (opts *Options) GetMempurgeThreshold() float64 {
	return float64(C.rocksdb_options_get_experimental_mempurge_threshold(opts.c))
}

// SetUnorderedWrite sets unordered_write to true trades higher write throughput with
// relaxing the immutability guarantee of snapshots. This violates the
// repeatability one expects from ::Get from a snapshot, as well as
// ::MultiGet and Iterator's consistent-point-in-time view property.
// If the application cannot tolerate the relaxed guarantees, it can implement
// its own mechanisms to work around that and yet benefit from the higher
// throughput. Using TransactionDB with WRITE_PREPARED write policy and
// two_write_queues=true is one way to achieve immutable snapshots despite
// unordered_write.
//
// By default, i.e., when it is false, rocksdb does not advance the sequence
// number for new snapshots unless all the writes with lower sequence numbers
// are already finished. This provides the immutability that we except from
// snapshots. Moreover, since Iterator and MultiGet internally depend on
// snapshots, the snapshot immutability results into Iterator and MultiGet
// offering consistent-point-in-time view. If set to true, although
// Read-Your-Own-Write property is still provided, the snapshot immutability
// property is relaxed: the writes issued after the snapshot is obtained (with
// larger sequence numbers) will be still not visible to the reads from that
// snapshot, however, there still might be pending writes (with lower sequence
// number) that will change the state visible to the snapshot after they are
// landed to the memtable.
//
// Default: false
func (opts *Options) SetUnorderedWrite(value bool) {
	C.rocksdb_options_set_unordered_write(opts.c, boolToChar(value))
}

// UnorderedWrite checks if unordered_write is turned on.
func (opts *Options) UnorderedWrite() bool {
	return charToBool(C.rocksdb_options_get_unordered_write(opts.c))
}

// SetCuckooTableFactory sets to use cuckoo table factory.
//
// Note: move semantic. Don't use cuckoo options after calling this function.
//
// Default: nil.
func (opts *Options) SetCuckooTableFactory(cuckooOpts *CuckooTableOptions) {
	if cuckooOpts != nil {
		C.rocksdb_options_set_cuckoo_table_factory(opts.c, cuckooOpts.c)
		cuckooOpts.Destroy()
	}
}

// SetDumpMallocStats if true, then print malloc stats together with rocksdb.stats
// when printing to LOG.
func (opts *Options) SetDumpMallocStats(value bool) {
	C.rocksdb_options_set_dump_malloc_stats(opts.c, boolToChar(value))
}

// SetMemtableWholeKeyFiltering enable whole key bloom filter in memtable. Note this will only take effect
// if memtable_prefix_bloom_size_ratio is not 0. Enabling whole key filtering
// can potentially reduce CPU usage for point-look-ups.
//
// Default: false (disable)
//
// Dynamically changeable through SetOptions() API
func (opts *Options) SetMemtableWholeKeyFiltering(value bool) {
	C.rocksdb_options_set_memtable_whole_key_filtering(opts.c, boolToChar(value))
}

// Destroy deallocates the Options object.
func (opts *Options) Destroy() {
	C.rocksdb_options_destroy(opts.c)
	opts.c = nil

	C.rocksdb_comparator_destroy(opts.ccmp)
	opts.ccmp = nil

	C.rocksdb_slicetransform_destroy(opts.cst)
	opts.cst = nil

	C.rocksdb_compactionfilter_destroy(opts.ccf)
	opts.ccf = nil

	C.rocksdb_mergeoperator_destroy(opts.cmo)
	opts.cmo = nil

	if opts.env != nil {
		C.rocksdb_env_destroy(opts.env)
		opts.env = nil
	}

	opts.bbto = nil
}

type LatestOptions struct {
	opts Options

	cfNames_ **C.char
	cfNames  []string

	cfOptions_ **C.rocksdb_options_t
	cfOptions  []Options
}

// LoadLatestOptions loads the latest rocksdb options from the specified db_path.
//
// On success, num_column_families will be updated with a non-zero
// number indicating the number of column families.
func LoadLatestOptions(path string, env *Env, ignoreUnknownOpts bool, cache *Cache) (lo *LatestOptions, err error) {
	if env == nil || cache == nil {
		return nil, fmt.Errorf("please specify env and cache")
	}

	cPath := C.CString(path)

	var env_ *C.rocksdb_env_t
	if env != nil {
		env_ = env.c
	}

	var cache_ *C.rocksdb_cache_t
	if cache != nil {
		cache_ = cache.c
	}

	var (
		numCF   C.size_t
		opts    *C.rocksdb_options_t
		cfNames **C.char
		cfOpts  **C.rocksdb_options_t
		cErr    *C.char
	)
	C.rocksdb_load_latest_options(cPath, env_, C.bool(ignoreUnknownOpts), cache_, &opts, &numCF, &cfNames, &cfOpts, &cErr)

	if err = fromCError(cErr); err == nil {
		// convert **C.rocksdb_options_t into []Options
		cfOptions_ := unsafe.Slice(cfOpts, int(numCF))
		cfOptions := make([]Options, int(numCF))
		for i := range cfOptions {
			cfOptions[i] = Options{c: cfOptions_[i]}
		}

		lo = &LatestOptions{
			opts: Options{c: opts},

			cfNames_: cfNames,
			cfNames:  charSliceIntoStringSlice(cfNames, C.int(numCF)),

			cfOptions_: cfOpts,
			cfOptions:  cfOptions,
		}
	}

	C.free(unsafe.Pointer(cPath))
	return
}

// Options gets the latest options.
func (l *LatestOptions) Options() *Options {
	return &l.opts
}

// ColumnFamilyNames gets column family names.
func (l *LatestOptions) ColumnFamilyNames() []string {
	return l.cfNames
}

// ColumnFamilyOpts returns corresponding options of column families.
func (l *LatestOptions) ColumnFamilyOpts() []Options {
	return l.cfOptions
}

// Destroy release underlying db_options, column_family_names, and column_family_options.
func (l *LatestOptions) Destroy() {
	C.rocksdb_load_latest_options_destroy(l.opts.c, l.cfNames_, l.cfOptions_, C.size_t(len(l.cfNames)))
	l.opts.c = nil
}

type EncodingType byte

const (
	// EncodingTypePlain always write full keys without any special encoding.
	EncodingTypePlain EncodingType = iota

	// EncodingTypePrefix find opportunity to write the same prefix once for multiple rows.
	// In some cases, when a key follows a previous key with the same prefix,
	// instead of writing out the full key, it just writes out the size of the
	// shared prefix, as well as other bytes, to save some bytes.
	//
	// When using this option, the user is required to use the same prefix
	// extractor to make sure the same prefix will be extracted from the same key.
	// The Name() value of the prefix extractor will be stored in the file. When
	// reopening the file, the name of the options.prefix_extractor given will be
	// bitwise compared to the prefix extractors stored in the file. An error
	// will be returned if the two don't match.
	EncodingTypePrefix
)
