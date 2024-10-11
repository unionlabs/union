// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package sstable

import (
	"github.com/cockroachdb/fifo"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/cache"
)

// Compression is the per-block compression algorithm to use.
type Compression int

// The available compression types.
const (
	DefaultCompression Compression = iota
	NoCompression
	SnappyCompression
	ZstdCompression
	NCompression
)

var ignoredInternalProperties = map[string]struct{}{
	"rocksdb.column.family.id":             {},
	"rocksdb.fixed.key.length":             {},
	"rocksdb.index.key.is.user.key":        {},
	"rocksdb.index.value.is.delta.encoded": {},
	"rocksdb.oldest.key.time":              {},
	"rocksdb.creation.time":                {},
	"rocksdb.file.creation.time":           {},
	"rocksdb.format.version":               {},
}

func (c Compression) String() string {
	switch c {
	case DefaultCompression:
		return "Default"
	case NoCompression:
		return "NoCompression"
	case SnappyCompression:
		return "Snappy"
	case ZstdCompression:
		return "ZSTD"
	default:
		return "Unknown"
	}
}

// FilterType exports the base.FilterType type.
type FilterType = base.FilterType

// Exported TableFilter constants.
const (
	TableFilter = base.TableFilter
)

// FilterWriter exports the base.FilterWriter type.
type FilterWriter = base.FilterWriter

// FilterPolicy exports the base.FilterPolicy type.
type FilterPolicy = base.FilterPolicy

// TablePropertyCollector provides a hook for collecting user-defined
// properties based on the keys and values stored in an sstable. A new
// TablePropertyCollector is created for an sstable when the sstable is being
// written.
type TablePropertyCollector interface {
	// Add is called with each new entry added to the sstable. While the sstable
	// is itself sorted by key, do not assume that the entries are added in any
	// order. In particular, the ordering of point entries and range tombstones
	// is unspecified.
	Add(key InternalKey, value []byte) error

	// Finish is called when all entries have been added to the sstable. The
	// collected properties (if any) should be added to the specified map. Note
	// that in case of an error during sstable construction, Finish may not be
	// called.
	Finish(userProps map[string]string) error

	// The name of the property collector.
	Name() string
}

// SuffixReplaceableTableCollector is an extension to the TablePropertyCollector
// interface that allows a table property collector to indicate that it supports
// being *updated* during suffix replacement, i.e. when an existing SST in which
// all keys have the same key suffix is updated to have a new suffix.
//
// A collector which supports being updated in such cases must be able to derive
// its updated value from its old value and the change being made to the suffix,
// without needing to be passed each updated K/V.
//
// For example, a collector that only inspects values can simply copy its
// previously computed property as-is, since key-suffix replacement does not
// change values, while a collector that depends only on key suffixes, like one
// which collected mvcc-timestamp bounds from timestamp-suffixed keys, can just
// set its new bounds from the new suffix, as it is common to all keys, without
// needing to recompute it from every key.
type SuffixReplaceableTableCollector interface {
	// UpdateKeySuffixes is called when a table is updated to change the suffix of
	// all keys in the table, and is passed the old value for that prop, if any,
	// for that table as well as the old and new suffix.
	UpdateKeySuffixes(oldProps map[string]string, oldSuffix, newSuffix []byte) error
}

// ReaderOptions holds the parameters needed for reading an sstable.
type ReaderOptions struct {
	// Cache is used to cache uncompressed blocks from sstables.
	//
	// The default cache size is a zero-size cache.
	Cache *cache.Cache

	// LoadBlockSema, if set, is used to limit the number of blocks that can be
	// loaded (i.e. read from the filesystem) in parallel. Each load acquires one
	// unit from the semaphore for the duration of the read.
	LoadBlockSema *fifo.Semaphore

	// User properties specified in this map will not be added to sst.Properties.UserProperties.
	DeniedUserProperties map[string]struct{}

	// Comparer defines a total ordering over the space of []byte keys: a 'less
	// than' relationship. The same comparison algorithm must be used for reads
	// and writes over the lifetime of the DB.
	//
	// The default value uses the same ordering as bytes.Compare.
	Comparer *Comparer

	// Merge defines the Merge function in use for this keyspace.
	Merge base.Merge

	// Filters is a map from filter policy name to filter policy. It is used for
	// debugging tools which may be used on multiple databases configured with
	// different filter policies. It is not necessary to populate this filters
	// map during normal usage of a DB.
	Filters map[string]FilterPolicy

	// Merger defines the associative merge operation to use for merging values
	// written with {Batch,DB}.Merge. The MergerName is checked for consistency
	// with the value stored in the sstable when it was written.
	MergerName string

	// Logger is an optional logger and tracer.
	LoggerAndTracer base.LoggerAndTracer
}

func (o ReaderOptions) ensureDefaults() ReaderOptions {
	if o.Comparer == nil {
		o.Comparer = base.DefaultComparer
	}
	if o.Merge == nil {
		o.Merge = base.DefaultMerger.Merge
	}
	if o.MergerName == "" {
		o.MergerName = base.DefaultMerger.Name
	}
	if o.LoggerAndTracer == nil {
		o.LoggerAndTracer = base.NoopLoggerAndTracer{}
	}
	if o.DeniedUserProperties == nil {
		o.DeniedUserProperties = ignoredInternalProperties
	}
	return o
}

// WriterOptions holds the parameters used to control building an sstable.
type WriterOptions struct {
	// BlockRestartInterval is the number of keys between restart points
	// for delta encoding of keys.
	//
	// The default value is 16.
	BlockRestartInterval int

	// BlockSize is the target uncompressed size in bytes of each table block.
	//
	// The default value is 4096.
	BlockSize int

	// BlockSizeThreshold finishes a block if the block size is larger than the
	// specified percentage of the target block size and adding the next entry
	// would cause the block to be larger than the target block size.
	//
	// The default value is 90
	BlockSizeThreshold int

	// Cache is used to cache uncompressed blocks from sstables.
	//
	// The default is a nil cache.
	Cache *cache.Cache

	// Comparer defines a total ordering over the space of []byte keys: a 'less
	// than' relationship. The same comparison algorithm must be used for reads
	// and writes over the lifetime of the DB.
	//
	// The default value uses the same ordering as bytes.Compare.
	Comparer *Comparer

	// Compression defines the per-block compression to use.
	//
	// The default value (DefaultCompression) uses snappy compression.
	Compression Compression

	// FilterPolicy defines a filter algorithm (such as a Bloom filter) that can
	// reduce disk reads for Get calls.
	//
	// One such implementation is bloom.FilterPolicy(10) from the pebble/bloom
	// package.
	//
	// The default value means to use no filter.
	FilterPolicy FilterPolicy

	// FilterType defines whether an existing filter policy is applied at a
	// block-level or table-level. Block-level filters use less memory to create,
	// but are slower to access as a check for the key in the index must first be
	// performed to locate the filter block. A table-level filter will require
	// memory proportional to the number of keys in an sstable to create, but
	// avoids the index lookup when determining if a key is present. Table-level
	// filters should be preferred except under constrained memory situations.
	FilterType FilterType

	// IndexBlockSize is the target uncompressed size in bytes of each index
	// block. When the index block size is larger than this target, two-level
	// indexes are automatically enabled. Setting this option to a large value
	// (such as math.MaxInt32) disables the automatic creation of two-level
	// indexes.
	//
	// The default value is the value of BlockSize.
	IndexBlockSize int

	// Merger defines the associative merge operation to use for merging values
	// written with {Batch,DB}.Merge. The MergerName is checked for consistency
	// with the value stored in the sstable when it was written.
	MergerName string

	// TableFormat specifies the format version for writing sstables. The default
	// is TableFormatRocksDBv2 which creates RocksDB compatible sstables. Use
	// TableFormatLevelDB to create LevelDB compatible sstable which can be used
	// by a wider range of tools and libraries.
	TableFormat TableFormat

	// IsStrictObsolete is only relevant for >= TableFormatPebblev4. See comment
	// in format.go. Must be false if format < TableFormatPebblev4.
	//
	// TODO(bilal): set this when writing shared ssts.
	IsStrictObsolete bool

	// WritingToLowestLevel is only relevant for >= TableFormatPebblev4. It is
	// used to set the obsolete bit on DEL/DELSIZED/SINGLEDEL if they are the
	// youngest for a userkey.
	WritingToLowestLevel bool

	// TablePropertyCollectors is a list of TablePropertyCollector creation
	// functions. A new TablePropertyCollector is created for each sstable built
	// and lives for the lifetime of the table.
	TablePropertyCollectors []func() TablePropertyCollector

	// BlockPropertyCollectors is a list of BlockPropertyCollector creation
	// functions. A new BlockPropertyCollector is created for each sstable
	// built and lives for the lifetime of writing that table.
	BlockPropertyCollectors []func() BlockPropertyCollector

	// Checksum specifies which checksum to use.
	Checksum ChecksumType

	// Parallelism is used to indicate that the sstable Writer is allowed to
	// compress data blocks and write datablocks to disk in parallel with the
	// Writer client goroutine.
	Parallelism bool

	// ShortAttributeExtractor mirrors
	// Options.Experimental.ShortAttributeExtractor.
	ShortAttributeExtractor base.ShortAttributeExtractor

	// RequiredInPlaceValueBound mirrors
	// Options.Experimental.RequiredInPlaceValueBound.
	RequiredInPlaceValueBound UserKeyPrefixBound
}

func (o WriterOptions) ensureDefaults() WriterOptions {
	if o.BlockRestartInterval <= 0 {
		o.BlockRestartInterval = base.DefaultBlockRestartInterval
	}
	if o.BlockSize <= 0 {
		o.BlockSize = base.DefaultBlockSize
	}
	if o.BlockSizeThreshold <= 0 {
		o.BlockSizeThreshold = base.DefaultBlockSizeThreshold
	}
	if o.Comparer == nil {
		o.Comparer = base.DefaultComparer
	}
	if o.Compression <= DefaultCompression || o.Compression >= NCompression {
		o.Compression = SnappyCompression
	}
	if o.IndexBlockSize <= 0 {
		o.IndexBlockSize = o.BlockSize
	}
	if o.MergerName == "" {
		o.MergerName = base.DefaultMerger.Name
	}
	if o.Checksum == ChecksumTypeNone {
		o.Checksum = ChecksumTypeCRC32c
	}
	// By default, if the table format is not specified, fall back to using the
	// most compatible format.
	if o.TableFormat == TableFormatUnspecified {
		o.TableFormat = TableFormatRocksDBv2
	}
	return o
}
