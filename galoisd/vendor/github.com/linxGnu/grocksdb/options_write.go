package grocksdb

// #include "rocksdb/c.h"
import "C"

// WriteOptions represent all of the available options when writing to a
// database.
type WriteOptions struct {
	c *C.rocksdb_writeoptions_t
}

// NewDefaultWriteOptions creates a default WriteOptions object.
func NewDefaultWriteOptions() *WriteOptions {
	return newNativeWriteOptions(C.rocksdb_writeoptions_create())
}

// NewNativeWriteOptions creates a WriteOptions object.
func newNativeWriteOptions(c *C.rocksdb_writeoptions_t) *WriteOptions {
	return &WriteOptions{c: c}
}

// SetSync sets the sync mode. If true, the write will be flushed
// from the operating system buffer cache before the write is considered complete.
// If this flag is true, writes will be slower.
//
// Default: false
func (opts *WriteOptions) SetSync(value bool) {
	C.rocksdb_writeoptions_set_sync(opts.c, boolToChar(value))
}

// IsSync returns if sync mode is turned on.
func (opts *WriteOptions) IsSync() bool {
	return charToBool(C.rocksdb_writeoptions_get_sync(opts.c))
}

// DisableWAL sets whether WAL should be active or not.
// If true, writes will not first go to the write ahead log,
// and the write may got lost after a crash.
//
// Default: false
func (opts *WriteOptions) DisableWAL(value bool) {
	C.rocksdb_writeoptions_disable_WAL(opts.c, C.int(boolToChar(value)))
}

// IsDisableWAL returns if we turned on DisableWAL flag for writing.
func (opts *WriteOptions) IsDisableWAL() bool {
	return charToBool(C.rocksdb_writeoptions_get_disable_WAL(opts.c))
}

// SetIgnoreMissingColumnFamilies if true and if user is trying to write
// to column families that don't exist (they were dropped), ignore the
// write (don't return an error). If there are multiple writes in a WriteBatch,
// other writes will succeed.
//
// Default: false
func (opts *WriteOptions) SetIgnoreMissingColumnFamilies(value bool) {
	C.rocksdb_writeoptions_set_ignore_missing_column_families(opts.c, boolToChar(value))
}

// IgnoreMissingColumnFamilies returns the setting for ignoring missing column famlies.
//
// If true and if user is trying to write
// to column families that don't exist (they were dropped), ignore the
// write (don't return an error). If there are multiple writes in a WriteBatch,
// other writes will succeed.
func (opts *WriteOptions) IgnoreMissingColumnFamilies() bool {
	return charToBool(C.rocksdb_writeoptions_get_ignore_missing_column_families(opts.c))
}

// SetNoSlowdown if true and we need to wait or sleep for the write request, fails
// immediately with Status::Incomplete().
//
// Default: false
func (opts *WriteOptions) SetNoSlowdown(value bool) {
	C.rocksdb_writeoptions_set_no_slowdown(opts.c, boolToChar(value))
}

// IsNoSlowdown returns no_slow_down setting.
func (opts *WriteOptions) IsNoSlowdown() bool {
	return charToBool(C.rocksdb_writeoptions_get_no_slowdown(opts.c))
}

// SetLowPri if true, this write request is of lower priority if compaction is
// behind. In this case, no_slowdown = true, the request will be cancelled
// immediately with Status::Incomplete() returned. Otherwise, it will be
// slowed down. The slowdown value is determined by RocksDB to guarantee
// it introduces minimum impacts to high priority writes.
//
// Default: false
func (opts *WriteOptions) SetLowPri(value bool) {
	C.rocksdb_writeoptions_set_low_pri(opts.c, boolToChar(value))
}

// IsLowPri returns if the write request is of lower priority if compaction is behind.
func (opts *WriteOptions) IsLowPri() bool {
	return charToBool(C.rocksdb_writeoptions_get_low_pri(opts.c))
}

// SetMemtableInsertHintPerBatch if true, this writebatch will maintain the last insert positions of each
// memtable as hints in concurrent write. It can improve write performance
// in concurrent writes if keys in one writebatch are sequential. In
// non-concurrent writes (when concurrent_memtable_writes is false) this
// option will be ignored.
//
// Default: false
func (opts *WriteOptions) SetMemtableInsertHintPerBatch(value bool) {
	C.rocksdb_writeoptions_set_memtable_insert_hint_per_batch(opts.c, boolToChar(value))
}

// MemtableInsertHintPerBatch returns if this writebatch will maintain the last insert positions of each
// memtable as hints in concurrent write.
func (opts *WriteOptions) MemtableInsertHintPerBatch() bool {
	return charToBool(C.rocksdb_writeoptions_get_memtable_insert_hint_per_batch(opts.c))
}

// Destroy deallocates the WriteOptions object.
func (opts *WriteOptions) Destroy() {
	C.rocksdb_writeoptions_destroy(opts.c)
	opts.c = nil
}
