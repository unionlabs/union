/*
 * Copyright 2017 Dgraph Labs, Inc. and Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package badger

import (
	"time"

	"github.com/dgraph-io/badger/v2/options"
	"github.com/dgraph-io/badger/v2/table"
)

// Note: If you add a new option X make sure you also add a WithX method on Options.

// Options are params for creating DB object.
//
// This package provides DefaultOptions which contains options that should
// work for most applications. Consider using that as a starting point before
// customizing it for your own needs.
//
// Each option X is documented on the WithX method.
type Options struct {
	// Required options.

	Dir      string
	ValueDir string

	// Usually modified options.

	SyncWrites          bool
	TableLoadingMode    options.FileLoadingMode
	ValueLogLoadingMode options.FileLoadingMode
	NumVersionsToKeep   int
	ReadOnly            bool
	Truncate            bool
	Logger              Logger
	Compression         options.CompressionType
	InMemory            bool

	// Fine tuning options.

	MaxTableSize        int64
	LevelSizeMultiplier int
	MaxLevels           int
	ValueThreshold      int
	NumMemtables        int
	// Changing BlockSize across DB runs will not break badger. The block size is
	// read from the block index stored at the end of the table.
	BlockSize          int
	BloomFalsePositive float64
	KeepL0InMemory     bool
	BlockCacheSize     int64
	IndexCacheSize     int64
	LoadBloomsOnOpen   bool

	NumLevelZeroTables      int
	NumLevelZeroTablesStall int

	LevelOneSize       int64
	ValueLogFileSize   int64
	ValueLogMaxEntries uint32

	NumCompactors        int
	CompactL0OnClose     bool
	LogRotatesToFlush    int32
	ZSTDCompressionLevel int

	// When set, checksum will be validated for each entry read from the value log file.
	VerifyValueChecksum bool

	// Encryption related options.
	EncryptionKey                 []byte        // encryption key
	EncryptionKeyRotationDuration time.Duration // key rotation duration

	// BypassLockGaurd will bypass the lock guard on badger. Bypassing lock
	// guard can cause data corruption if multiple badger instances are using
	// the same directory. Use this options with caution.
	BypassLockGuard bool

	// ChecksumVerificationMode decides when db should verify checksums for SSTable blocks.
	ChecksumVerificationMode options.ChecksumVerificationMode

	// DetectConflicts determines whether the transactions would be checked for
	// conflicts. The transactions can be processed at a higher rate when
	// conflict detection is disabled.
	DetectConflicts bool

	// Transaction start and commit timestamps are managed by end-user.
	// This is only useful for databases built on top of Badger (like Dgraph).
	// Not recommended for most users.
	managedTxns bool

	// 4. Flags for testing purposes
	// ------------------------------
	maxBatchCount int64 // max entries in batch
	maxBatchSize  int64 // max batch size in bytes
}

// DefaultOptions sets a list of recommended options for good performance.
// Feel free to modify these to suit your needs with the WithX methods.
func DefaultOptions(path string) Options {
	return Options{
		Dir:                 path,
		ValueDir:            path,
		LevelOneSize:        256 << 20,
		LevelSizeMultiplier: 10,
		TableLoadingMode:    options.MemoryMap,
		ValueLogLoadingMode: options.MemoryMap,
		// table.MemoryMap to mmap() the tables.
		// table.Nothing to not preload the tables.
		MaxLevels:               7,
		MaxTableSize:            64 << 20,
		NumCompactors:           2, // Run at least 2 compactors. One is dedicated for L0.
		NumLevelZeroTables:      5,
		NumLevelZeroTablesStall: 15,
		NumMemtables:            5,
		BloomFalsePositive:      0.01,
		BlockSize:               4 * 1024,
		SyncWrites:              true,
		NumVersionsToKeep:       1,
		CompactL0OnClose:        true,
		KeepL0InMemory:          false,
		VerifyValueChecksum:     false,
		Compression:             options.None,
		BlockCacheSize:          0,
		IndexCacheSize:          0,
		LoadBloomsOnOpen:        true,

		// The following benchmarks were done on a 4 KB block size (default block size). The
		// compression is ratio supposed to increase with increasing compression level but since the
		// input for compression algorithm is small (4 KB), we don't get significant benefit at
		// level 3.
		// NOTE: The benchmarks are with DataDog ZSTD that requires CGO. Hence, no longer valid.
		// no_compression-16              10	 502848865 ns/op	 165.46 MB/s	-
		// zstd_compression/level_1-16     7	 739037966 ns/op	 112.58 MB/s	2.93
		// zstd_compression/level_3-16     7	 756950250 ns/op	 109.91 MB/s	2.72
		// zstd_compression/level_15-16    1	11135686219 ns/op	   7.47 MB/s	4.38
		// Benchmark code can be found in table/builder_test.go file
		ZSTDCompressionLevel: 1,

		// Nothing to read/write value log using standard File I/O
		// MemoryMap to mmap() the value log files
		// (2^30 - 1)*2 when mmapping < 2^31 - 1, max int32.
		// -1 so 2*ValueLogFileSize won't overflow on 32-bit systems.
		ValueLogFileSize: 1<<30 - 1,

		ValueLogMaxEntries:            1000000,
		ValueThreshold:                1 << 10, // 1 KB.
		Truncate:                      false,
		Logger:                        defaultLogger(INFO),
		LogRotatesToFlush:             2,
		EncryptionKey:                 []byte{},
		EncryptionKeyRotationDuration: 10 * 24 * time.Hour, // Default 10 days.
		DetectConflicts:               true,
	}
}

func buildTableOptions(opt Options) table.Options {
	return table.Options{
		BlockSize:            opt.BlockSize,
		BloomFalsePositive:   opt.BloomFalsePositive,
		LoadBloomsOnOpen:     opt.LoadBloomsOnOpen,
		LoadingMode:          opt.TableLoadingMode,
		ChkMode:              opt.ChecksumVerificationMode,
		Compression:          opt.Compression,
		ZSTDCompressionLevel: opt.ZSTDCompressionLevel,
	}
}

const (
	maxValueThreshold = (1 << 20) // 1 MB
)

// LSMOnlyOptions follows from DefaultOptions, but sets a higher ValueThreshold
// so values would be collocated with the LSM tree, with value log largely acting
// as a write-ahead log only. These options would reduce the disk usage of value
// log, and make Badger act more like a typical LSM tree.
func LSMOnlyOptions(path string) Options {
	// Let's not set any other options, because they can cause issues with the
	// size of key-value a user can pass to Badger. For e.g., if we set
	// ValueLogFileSize to 64MB, a user can't pass a value more than that.
	// Setting it to ValueLogMaxEntries to 1000, can generate too many files.
	// These options are better configured on a usage basis, than broadly here.
	// The ValueThreshold is the most important setting a user needs to do to
	// achieve a heavier usage of LSM tree.
	// NOTE: If a user does not want to set 64KB as the ValueThreshold because
	// of performance reasons, 1KB would be a good option too, allowing
	// values smaller than 1KB to be collocated with the keys in the LSM tree.
	return DefaultOptions(path).WithValueThreshold(maxValueThreshold /* 1 MB */)
}

// WithDir returns a new Options value with Dir set to the given value.
//
// Dir is the path of the directory where key data will be stored in.
// If it doesn't exist, Badger will try to create it for you.
// This is set automatically to be the path given to `DefaultOptions`.
func (opt Options) WithDir(val string) Options {
	opt.Dir = val
	return opt
}

// WithValueDir returns a new Options value with ValueDir set to the given value.
//
// ValueDir is the path of the directory where value data will be stored in.
// If it doesn't exist, Badger will try to create it for you.
// This is set automatically to be the path given to `DefaultOptions`.
func (opt Options) WithValueDir(val string) Options {
	opt.ValueDir = val
	return opt
}

// WithLoggingLevel returns a new Options value with logging level of the
// default logger set to the given value.
// LoggingLevel sets the level of logging. It should be one of DEBUG, INFO,
// WARNING or ERROR levels.
//
// The default value of LoggingLevel is INFO.
func (opt Options) WithLoggingLevel(val loggingLevel) Options {
	opt.Logger = defaultLogger(val)
	return opt
}

// WithSyncWrites returns a new Options value with SyncWrites set to the given value.
//
// When SyncWrites is true all writes are synced to disk. Setting this to false would achieve better
// performance, but may cause data loss in case of crash.
//
// The default value of SyncWrites is true.
func (opt Options) WithSyncWrites(val bool) Options {
	opt.SyncWrites = val
	return opt
}

// WithTableLoadingMode returns a new Options value with TableLoadingMode set to the given value.
//
// TableLoadingMode indicates which file loading mode should be used for the LSM tree data files.
//
// The default value of TableLoadingMode is options.MemoryMap.
func (opt Options) WithTableLoadingMode(val options.FileLoadingMode) Options {
	opt.TableLoadingMode = val
	return opt
}

// WithValueLogLoadingMode returns a new Options value with ValueLogLoadingMode set to the given
// value.
//
// ValueLogLoadingMode indicates which file loading mode should be used for the value log data
// files.
//
// The default value of ValueLogLoadingMode is options.MemoryMap.
func (opt Options) WithValueLogLoadingMode(val options.FileLoadingMode) Options {
	opt.ValueLogLoadingMode = val
	return opt
}

// WithNumVersionsToKeep returns a new Options value with NumVersionsToKeep set to the given value.
//
// NumVersionsToKeep sets how many versions to keep per key at most.
//
// The default value of NumVersionsToKeep is 1.
func (opt Options) WithNumVersionsToKeep(val int) Options {
	opt.NumVersionsToKeep = val
	return opt
}

// WithReadOnly returns a new Options value with ReadOnly set to the given value.
//
// When ReadOnly is true the DB will be opened on read-only mode.
// Multiple processes can open the same Badger DB.
// Note: if the DB being opened had crashed before and has vlog data to be replayed,
// ReadOnly will cause Open to fail with an appropriate message.
//
// The default value of ReadOnly is false.
func (opt Options) WithReadOnly(val bool) Options {
	opt.ReadOnly = val
	return opt
}

// WithTruncate returns a new Options value with Truncate set to the given value.
//
// Truncate indicates whether value log files should be truncated to delete corrupt data, if any.
// This option is ignored when ReadOnly is true.
//
// The default value of Truncate is false.
func (opt Options) WithTruncate(val bool) Options {
	opt.Truncate = val
	return opt
}

// WithLogger returns a new Options value with Logger set to the given value.
//
// Logger provides a way to configure what logger each value of badger.DB uses.
//
// The default value of Logger writes to stderr using the log package from the Go standard library.
func (opt Options) WithLogger(val Logger) Options {
	opt.Logger = val
	return opt
}

// WithMaxTableSize returns a new Options value with MaxTableSize set to the given value.
//
// MaxTableSize sets the maximum size in bytes for each LSM table or file.
//
// The default value of MaxTableSize is 64MB.
func (opt Options) WithMaxTableSize(val int64) Options {
	opt.MaxTableSize = val
	return opt
}

// WithLevelSizeMultiplier returns a new Options value with LevelSizeMultiplier set to the given
// value.
//
// LevelSizeMultiplier sets the ratio between the maximum sizes of contiguous levels in the LSM.
// Once a level grows to be larger than this ratio allowed, the compaction process will be
//  triggered.
//
// The default value of LevelSizeMultiplier is 10.
func (opt Options) WithLevelSizeMultiplier(val int) Options {
	opt.LevelSizeMultiplier = val
	return opt
}

// WithMaxLevels returns a new Options value with MaxLevels set to the given value.
//
// Maximum number of levels of compaction allowed in the LSM.
//
// The default value of MaxLevels is 7.
func (opt Options) WithMaxLevels(val int) Options {
	opt.MaxLevels = val
	return opt
}

// WithValueThreshold returns a new Options value with ValueThreshold set to the given value.
//
// ValueThreshold sets the threshold used to decide whether a value is stored directly in the LSM
// tree or separately in the log value files.
//
// The default value of ValueThreshold is 1 KB, but LSMOnlyOptions sets it to maxValueThreshold.
func (opt Options) WithValueThreshold(val int) Options {
	opt.ValueThreshold = val
	return opt
}

// WithNumMemtables returns a new Options value with NumMemtables set to the given value.
//
// NumMemtables sets the maximum number of tables to keep in memory before stalling.
//
// The default value of NumMemtables is 5.
func (opt Options) WithNumMemtables(val int) Options {
	opt.NumMemtables = val
	return opt
}

// WithBloomFalsePositive returns a new Options value with BloomFalsePositive set
// to the given value.
//
// BloomFalsePositive sets the false positive probability of the bloom filter in any SSTable.
// Before reading a key from table, the bloom filter is checked for key existence.
// BloomFalsePositive might impact read performance of DB. Lower BloomFalsePositive value might
// consume more memory.
//
// The default value of BloomFalsePositive is 0.01.
func (opt Options) WithBloomFalsePositive(val float64) Options {
	opt.BloomFalsePositive = val
	return opt
}

// WithBlockSize returns a new Options value with BlockSize set to the given value.
//
// BlockSize sets the size of any block in SSTable. SSTable is divided into multiple blocks
// internally. Each block is compressed using prefix diff encoding.
//
// The default value of BlockSize is 4KB.
func (opt Options) WithBlockSize(val int) Options {
	opt.BlockSize = val
	return opt
}

// WithNumLevelZeroTables returns a new Options value with NumLevelZeroTables set to the given
// value.
//
// NumLevelZeroTables sets the maximum number of Level 0 tables before compaction starts.
//
// The default value of NumLevelZeroTables is 5.
func (opt Options) WithNumLevelZeroTables(val int) Options {
	opt.NumLevelZeroTables = val
	return opt
}

// WithNumLevelZeroTablesStall returns a new Options value with NumLevelZeroTablesStall set to the
// given value.
//
// NumLevelZeroTablesStall sets the number of Level 0 tables that once reached causes the DB to
// stall until compaction succeeds.
//
// The default value of NumLevelZeroTablesStall is 10.
func (opt Options) WithNumLevelZeroTablesStall(val int) Options {
	opt.NumLevelZeroTablesStall = val
	return opt
}

// WithLevelOneSize returns a new Options value with LevelOneSize set to the given value.
//
// LevelOneSize sets the maximum total size for Level 1.
//
// The default value of LevelOneSize is 20MB.
func (opt Options) WithLevelOneSize(val int64) Options {
	opt.LevelOneSize = val
	return opt
}

// WithValueLogFileSize returns a new Options value with ValueLogFileSize set to the given value.
//
// ValueLogFileSize sets the maximum size of a single value log file.
//
// The default value of ValueLogFileSize is 1GB.
func (opt Options) WithValueLogFileSize(val int64) Options {
	opt.ValueLogFileSize = val
	return opt
}

// WithValueLogMaxEntries returns a new Options value with ValueLogMaxEntries set to the given
// value.
//
// ValueLogMaxEntries sets the maximum number of entries a value log file can hold approximately.
// A actual size limit of a value log file is the minimum of ValueLogFileSize and
// ValueLogMaxEntries.
//
// The default value of ValueLogMaxEntries is one million (1000000).
func (opt Options) WithValueLogMaxEntries(val uint32) Options {
	opt.ValueLogMaxEntries = val
	return opt
}

// WithNumCompactors returns a new Options value with NumCompactors set to the given value.
//
// NumCompactors sets the number of compaction workers to run concurrently.
// Setting this to zero stops compactions, which could eventually cause writes to block forever.
//
// The default value of NumCompactors is 2. One is dedicated just for L0.
func (opt Options) WithNumCompactors(val int) Options {
	opt.NumCompactors = val
	return opt
}

// WithCompactL0OnClose returns a new Options value with CompactL0OnClose set to the given value.
//
// CompactL0OnClose determines whether Level 0 should be compacted before closing the DB.
// This ensures that both reads and writes are efficient when the DB is opened later.
// CompactL0OnClose is set to true if KeepL0InMemory is set to true.
//
// The default value of CompactL0OnClose is true.
func (opt Options) WithCompactL0OnClose(val bool) Options {
	opt.CompactL0OnClose = val
	return opt
}

// WithLogRotatesToFlush returns a new Options value with LogRotatesToFlush set to the given value.
//
// LogRotatesToFlush sets the number of value log file rotates after which the Memtables are
// flushed to disk. This is useful in write loads with fewer keys and larger values. This work load
// would fill up the value logs quickly, while not filling up the Memtables. Thus, on a crash
// and restart, the value log head could cause the replay of a good number of value log files
// which can slow things on start.
//
// The default value of LogRotatesToFlush is 2.
func (opt Options) WithLogRotatesToFlush(val int32) Options {
	opt.LogRotatesToFlush = val
	return opt
}

// WithEncryptionKey return a new Options value with EncryptionKey set to the given value.
//
// EncryptionKey is used to encrypt the data with AES. Type of AES is used based on the key
// size. For example 16 bytes will use AES-128. 24 bytes will use AES-192. 32 bytes will
// use AES-256.
func (opt Options) WithEncryptionKey(key []byte) Options {
	opt.EncryptionKey = key
	return opt
}

// WithEncryptionKeyRotationDuration returns new Options value with the duration set to
// the given value.
//
// Key Registry will use this duration to create new keys. If the previous generated
// key exceed the given duration. Then the key registry will create new key.
func (opt Options) WithEncryptionKeyRotationDuration(d time.Duration) Options {
	opt.EncryptionKeyRotationDuration = d
	return opt
}

// WithKeepL0InMemory returns a new Options value with KeepL0InMemory set to the given value.
//
// When KeepL0InMemory is set to true we will keep all Level 0 tables in memory. This leads to
// better performance in writes as well as compactions. In case of DB crash, the value log replay
// will take longer to complete since memtables and all level 0 tables will have to be recreated.
// This option also sets CompactL0OnClose option to true.
//
// The default value of KeepL0InMemory is false.
func (opt Options) WithKeepL0InMemory(val bool) Options {
	opt.KeepL0InMemory = val
	return opt
}

// WithCompression returns a new Options value with Compression set to the given value.
//
// When compression is enabled, every block will be compressed using the specified algorithm.
// This option doesn't affect existing tables. Only the newly created tables will be compressed.
//
// The default compression algorithm used is zstd when built with Cgo. Without Cgo, the default is
// snappy. Compression is enabled by default.
func (opt Options) WithCompression(cType options.CompressionType) Options {
	opt.Compression = cType
	return opt
}

// WithVerifyValueChecksum returns a new Options value with VerifyValueChecksum set to
// the given value.
//
// When VerifyValueChecksum is set to true, checksum will be verified for every entry read
// from the value log. If the value is stored in SST (value size less than value threshold) then the
// checksum validation will not be done.
//
// The default value of VerifyValueChecksum is False.
func (opt Options) WithVerifyValueChecksum(val bool) Options {
	opt.VerifyValueChecksum = val
	return opt
}

// WithChecksumVerificationMode returns a new Options value with ChecksumVerificationMode set to
// the given value.
//
// ChecksumVerificationMode indicates when the db should verify checksums for SSTable blocks.
//
// The default value of VerifyValueChecksum is options.NoVerification.
func (opt Options) WithChecksumVerificationMode(cvMode options.ChecksumVerificationMode) Options {
	opt.ChecksumVerificationMode = cvMode
	return opt
}

// WithBlockCacheSize returns a new Options value with BlockCacheSize set to the given value.
//
// This value specifies how much data cache should hold in memory. A small size
// of cache means lower memory consumption and lookups/iterations would take
// longer. It is recommended to use a cache if you're using compression or encryption.
// If compression and encryption both are disabled, adding a cache will lead to
// unnecessary overhead which will affect the read performance. Setting size to
// zero disables the cache altogether.
//
// Default value of BlockCacheSize is zero.
func (opt Options) WithBlockCacheSize(size int64) Options {
	opt.BlockCacheSize = size
	return opt
}

// WithInMemory returns a new Options value with Inmemory mode set to the given value.
//
// When badger is running in InMemory mode, everything is stored in memory. No value/sst files are
// created. In case of a crash all data will be lost.
func (opt Options) WithInMemory(b bool) Options {
	opt.InMemory = b
	return opt
}

// WithZSTDCompressionLevel returns a new Options value with ZSTDCompressionLevel set
// to the given value.
//
// The ZSTD compression algorithm supports 20 compression levels. The higher the compression
// level, the better is the compression ratio but lower is the performance. Lower levels
// have better performance and higher levels have better compression ratios.
// We recommend using level 1 ZSTD Compression Level. Any level higher than 1 seems to
// deteriorate badger's performance.
// The following benchmarks were done on a 4 KB block size (default block size). The compression is
// ratio supposed to increase with increasing compression level but since the input for compression
// algorithm is small (4 KB), we don't get significant benefit at level 3. It is advised to write
// your own benchmarks before choosing a compression algorithm or level.
//
// NOTE: The benchmarks are with DataDog ZSTD that requires CGO. Hence, no longer valid.
// no_compression-16              10	 502848865 ns/op	 165.46 MB/s	-
// zstd_compression/level_1-16     7	 739037966 ns/op	 112.58 MB/s	2.93
// zstd_compression/level_3-16     7	 756950250 ns/op	 109.91 MB/s	2.72
// zstd_compression/level_15-16    1	11135686219 ns/op	   7.47 MB/s	4.38
// Benchmark code can be found in table/builder_test.go file
func (opt Options) WithZSTDCompressionLevel(cLevel int) Options {
	opt.ZSTDCompressionLevel = cLevel
	return opt
}

// WithBypassLockGuard returns a new Options value with BypassLockGuard
// set to the given value.
//
// When BypassLockGuard option is set, badger will not acquire a lock on the
// directory. This could lead to data corruption if multiple badger instances
// write to the same data directory. Use this option with caution.
//
// The default value of BypassLockGuard is false.
func (opt Options) WithBypassLockGuard(b bool) Options {
	opt.BypassLockGuard = b
	return opt
}

// WithLoadBloomsOnOpen returns a new Options value with LoadBloomsOnOpen set to the given value.
//
// Badger uses bloom filters to speed up key lookups. When LoadBloomsOnOpen is set
// to false, bloom filters will be loaded lazily and not on DB open. Set this
// option to false to reduce the time taken to open the DB.
//
// The default value of LoadBloomsOnOpen is true.
func (opt Options) WithLoadBloomsOnOpen(b bool) Options {
	opt.LoadBloomsOnOpen = b
	return opt
}

// WithIndexCacheSize returns a new Options value with IndexCacheSize set to
// the given value.
//
// This value specifies how much memory should be used by table indices. These
// indices include the block offsets and the bloomfilters. Badger uses bloom
// filters to speed up lookups. Each table has its own bloom
// filter and each bloom filter is approximately of 5 MB.
//
// Zero value for IndexCacheSize means all the indices will be kept in
// memory and the cache is disabled.
//
// The default value of IndexCacheSize is 0 which means all indices are kept in
// memory.
func (opt Options) WithIndexCacheSize(size int64) Options {
	opt.IndexCacheSize = size
	return opt
}

// WithDetectConflicts returns a new Options value with DetectConflicts set to the given value.
//
// Detect conflicts options determines if the transactions would be checked for
// conflicts before committing them. When this option is set to false
// (detectConflicts=false) badger can process transactions at a higher rate.
// Setting this options to false might be useful when the user application
// deals with conflict detection and resolution.
//
// The default value of Detect conflicts is True.
func (opt Options) WithDetectConflicts(b bool) Options {
	opt.DetectConflicts = b
	return opt
}
