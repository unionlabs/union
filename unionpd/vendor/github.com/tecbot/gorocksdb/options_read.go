package gorocksdb

// #include "rocksdb/c.h"
import "C"
import "unsafe"

// ReadTier controls fetching of data during a read request.
// An application can issue a read request (via Get/Iterators) and specify
// if that read should process data that ALREADY resides on a specified cache
// level. For example, if an application specifies BlockCacheTier then the
// Get call will process data that is already processed in the memtable or
// the block cache. It will not page in data from the OS cache or data that
// resides in storage.
type ReadTier uint

const (
	// ReadAllTier reads data in memtable, block cache, OS cache or storage.
	ReadAllTier = ReadTier(0)
	// BlockCacheTier reads data in memtable or block cache.
	BlockCacheTier = ReadTier(1)
)

// ReadOptions represent all of the available options when reading from a
// database.
type ReadOptions struct {
	c *C.rocksdb_readoptions_t
}

// NewDefaultReadOptions creates a default ReadOptions object.
func NewDefaultReadOptions() *ReadOptions {
	return NewNativeReadOptions(C.rocksdb_readoptions_create())
}

// NewNativeReadOptions creates a ReadOptions object.
func NewNativeReadOptions(c *C.rocksdb_readoptions_t) *ReadOptions {
	return &ReadOptions{c}
}

// UnsafeGetReadOptions returns the underlying c read options object.
func (opts *ReadOptions) UnsafeGetReadOptions() unsafe.Pointer {
	return unsafe.Pointer(opts.c)
}

// SetVerifyChecksums speciy if all data read from underlying storage will be
// verified against corresponding checksums.
// Default: false
func (opts *ReadOptions) SetVerifyChecksums(value bool) {
	C.rocksdb_readoptions_set_verify_checksums(opts.c, boolToChar(value))
}

// SetPrefixSameAsStart Enforce that the iterator only iterates over the same
// prefix as the seek.
// This option is effective only for prefix seeks, i.e. prefix_extractor is
// non-null for the column family and total_order_seek is false.  Unlike
// iterate_upper_bound, prefix_same_as_start only works within a prefix
// but in both directions.
// Default: false
func (opts *ReadOptions) SetPrefixSameAsStart(value bool) {
	C.rocksdb_readoptions_set_prefix_same_as_start(opts.c, boolToChar(value))
}

// SetFillCache specify whether the "data block"/"index block"/"filter block"
// read for this iteration should be cached in memory?
// Callers may wish to set this field to false for bulk scans.
// Default: true
func (opts *ReadOptions) SetFillCache(value bool) {
	C.rocksdb_readoptions_set_fill_cache(opts.c, boolToChar(value))
}

// SetSnapshot sets the snapshot which should be used for the read.
// The snapshot must belong to the DB that is being read and must
// not have been released.
// Default: nil
func (opts *ReadOptions) SetSnapshot(snap *Snapshot) {
	C.rocksdb_readoptions_set_snapshot(opts.c, snap.c)
}

// SetReadTier specify if this read request should process data that ALREADY
// resides on a particular cache. If the required data is not
// found at the specified cache, then Status::Incomplete is returned.
// Default: ReadAllTier
func (opts *ReadOptions) SetReadTier(value ReadTier) {
	C.rocksdb_readoptions_set_read_tier(opts.c, C.int(value))
}

// SetTailing specify if to create a tailing iterator.
// A special iterator that has a view of the complete database
// (i.e. it can also be used to read newly added data) and
// is optimized for sequential reads. It will return records
// that were inserted into the database after the creation of the iterator.
// Default: false
func (opts *ReadOptions) SetTailing(value bool) {
	C.rocksdb_readoptions_set_tailing(opts.c, boolToChar(value))
}

// SetIterateUpperBound specifies "iterate_upper_bound", which defines
// the extent upto which the forward iterator can returns entries.
// Once the bound is reached, Valid() will be false.
// "iterate_upper_bound" is exclusive ie the bound value is
// not a valid entry.  If iterator_extractor is not null, the Seek target
// and iterator_upper_bound need to have the same prefix.
// This is because ordering is not guaranteed outside of prefix domain.
// There is no lower bound on the iterator. If needed, that can be easily
// implemented.
// Default: nullptr
func (opts *ReadOptions) SetIterateUpperBound(key []byte) {
	cKey := byteToChar(key)
	cKeyLen := C.size_t(len(key))
	C.rocksdb_readoptions_set_iterate_upper_bound(opts.c, cKey, cKeyLen)
}

// SetPinData specifies the value of "pin_data". If true, it keeps the blocks
// loaded by the iterator pinned in memory as long as the iterator is not deleted,
// If used when reading from tables created with
// BlockBasedTableOptions::use_delta_encoding = false,
// Iterator's property "rocksdb.iterator.is-key-pinned" is guaranteed to
// return 1.
// Default: false
func (opts *ReadOptions) SetPinData(value bool) {
	C.rocksdb_readoptions_set_pin_data(opts.c, boolToChar(value))
}

// SetReadaheadSize specifies the value of "readahead_size".
// If non-zero, NewIterator will create a new table reader which
// performs reads of the given size. Using a large size (> 2MB) can
// improve the performance of forward iteration on spinning disks.
// Default: 0
func (opts *ReadOptions) SetReadaheadSize(value uint64) {
	C.rocksdb_readoptions_set_readahead_size(opts.c, C.size_t(value))
}

// Destroy deallocates the ReadOptions object.
func (opts *ReadOptions) Destroy() {
	C.rocksdb_readoptions_destroy(opts.c)
	opts.c = nil
}
