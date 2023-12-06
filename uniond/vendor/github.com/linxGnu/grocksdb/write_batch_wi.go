package grocksdb

// #include "rocksdb/c.h"
import "C"

// WriteBatchWI is a batching with index of Puts, Merges and Deletes to implement read-your-own-write.
// See also: https://rocksdb.org/blog/2015/02/27/write-batch-with-index.html
type WriteBatchWI struct {
	c *C.rocksdb_writebatch_wi_t
}

// NewWriteBatchWI create a WriteBatchWI object.
//   - reserved_bytes: reserved bytes in underlying WriteBatch
//   - overwrite_key: if true, overwrite the key in the index when inserting
//     the same key as previously, so iterator will never
//     show two entries with the same key.
func NewWriteBatchWI(reservedBytes uint, overwriteKeys bool) *WriteBatchWI {
	cWB := C.rocksdb_writebatch_wi_create(C.size_t(reservedBytes), boolToChar(overwriteKeys))
	return newNativeWriteBatchWI(cWB)
}

// NewNativeWriteBatchWI create a WriteBatchWI object.
func newNativeWriteBatchWI(c *C.rocksdb_writebatch_wi_t) *WriteBatchWI {
	return &WriteBatchWI{c: c}
}

// Put queues a key-value pair.
func (wb *WriteBatchWI) Put(key, value []byte) {
	cKey := refGoBytes(key)
	cValue := refGoBytes(value)
	C.rocksdb_writebatch_wi_put(wb.c, cKey, C.size_t(len(key)), cValue, C.size_t(len(value)))
}

// PutCF queues a key-value pair in a column family.
func (wb *WriteBatchWI) PutCF(cf *ColumnFamilyHandle, key, value []byte) {
	cKey := refGoBytes(key)
	cValue := refGoBytes(value)
	C.rocksdb_writebatch_wi_put_cf(wb.c, cf.c, cKey, C.size_t(len(key)), cValue, C.size_t(len(value)))
}

// PutLogData appends a blob of arbitrary size to the records in this batch.
func (wb *WriteBatchWI) PutLogData(blob []byte) {
	cBlob := refGoBytes(blob)
	C.rocksdb_writebatch_wi_put_log_data(wb.c, cBlob, C.size_t(len(blob)))
}

// Merge queues a merge of "value" with the existing value of "key".
func (wb *WriteBatchWI) Merge(key, value []byte) {
	cKey := refGoBytes(key)
	cValue := refGoBytes(value)
	C.rocksdb_writebatch_wi_merge(wb.c, cKey, C.size_t(len(key)), cValue, C.size_t(len(value)))
}

// MergeCF queues a merge of "value" with the existing value of "key" in a
// column family.
func (wb *WriteBatchWI) MergeCF(cf *ColumnFamilyHandle, key, value []byte) {
	cKey := refGoBytes(key)
	cValue := refGoBytes(value)
	C.rocksdb_writebatch_wi_merge_cf(wb.c, cf.c, cKey, C.size_t(len(key)), cValue, C.size_t(len(value)))
}

// Delete queues a deletion of the data at key.
func (wb *WriteBatchWI) Delete(key []byte) {
	cKey := refGoBytes(key)
	C.rocksdb_writebatch_wi_delete(wb.c, cKey, C.size_t(len(key)))
}

// SingleDelete removes the database entry for "key". Requires that the key exists
// and was not overwritten. Returns OK on success, and a non-OK status
// on error.  It is not an error if "key" did not exist in the database.
//
// If a key is overwritten (by calling Put() multiple times), then the result
// of calling SingleDelete() on this key is undefined.  SingleDelete() only
// behaves correctly if there has been only one Put() for this key since the
// previous call to SingleDelete() for this key.
//
// This feature is currently an experimental performance optimization
// for a very specific workload.  It is up to the caller to ensure that
// SingleDelete is only used for a key that is not deleted using Delete() or
// written using Merge().  Mixing SingleDelete operations with Deletes and
// Merges can result in undefined behavior.
//
// Note: consider setting options.sync = true.
func (wb *WriteBatchWI) SingleDelete(key []byte) {
	cKey := refGoBytes(key)
	C.rocksdb_writebatch_wi_singledelete(wb.c, cKey, C.size_t(len(key)))
}

// DeleteCF queues a deletion of the data at key in a column family.
func (wb *WriteBatchWI) DeleteCF(cf *ColumnFamilyHandle, key []byte) {
	cKey := refGoBytes(key)
	C.rocksdb_writebatch_wi_delete_cf(wb.c, cf.c, cKey, C.size_t(len(key)))
}

// SingleDeleteCF same as SingleDelete but specific column family
func (wb *WriteBatchWI) SingleDeleteCF(cf *ColumnFamilyHandle, key []byte) {
	cKey := refGoBytes(key)
	C.rocksdb_writebatch_wi_singledelete_cf(wb.c, cf.c, cKey, C.size_t(len(key)))
}

// DeleteRange deletes keys that are between [startKey, endKey)
func (wb *WriteBatchWI) DeleteRange(startKey []byte, endKey []byte) {
	cStartKey := refGoBytes(startKey)
	cEndKey := refGoBytes(endKey)
	C.rocksdb_writebatch_wi_delete_range(wb.c, cStartKey, C.size_t(len(startKey)), cEndKey, C.size_t(len(endKey)))
}

// DeleteRangeCF deletes keys that are between [startKey, endKey) and
// belong to a given column family
func (wb *WriteBatchWI) DeleteRangeCF(cf *ColumnFamilyHandle, startKey []byte, endKey []byte) {
	cStartKey := refGoBytes(startKey)
	cEndKey := refGoBytes(endKey)
	C.rocksdb_writebatch_wi_delete_range_cf(wb.c, cf.c, cStartKey, C.size_t(len(startKey)), cEndKey, C.size_t(len(endKey)))
}

// Data returns the serialized version of this batch.
func (wb *WriteBatchWI) Data() []byte {
	var cSize C.size_t
	cValue := C.rocksdb_writebatch_wi_data(wb.c, &cSize)
	return refCBytes(cValue, cSize)
}

// Count returns the number of updates in the batch.
func (wb *WriteBatchWI) Count() int {
	return int(C.rocksdb_writebatch_wi_count(wb.c))
}

// NewIterator returns a iterator to iterate over the records in the batch.
func (wb *WriteBatchWI) NewIterator() *WriteBatchIterator {
	data := wb.Data()
	if len(data) < 8+4 {
		return &WriteBatchIterator{}
	}
	return &WriteBatchIterator{data: data[12:]}
}

// SetSavePoint records the state of the batch for future calls to RollbackToSavePoint().
// May be called multiple times to set multiple save points.
func (wb *WriteBatchWI) SetSavePoint() {
	C.rocksdb_writebatch_wi_set_save_point(wb.c)
}

// RollbackToSavePoint removes all entries in this batch (Put, Merge, Delete, PutLogData) since the
// most recent call to SetSavePoint() and removes the most recent save point.
func (wb *WriteBatchWI) RollbackToSavePoint() (err error) {
	var cErr *C.char
	C.rocksdb_writebatch_wi_rollback_to_save_point(wb.c, &cErr)
	err = fromCError(cErr)
	return
}

// Get returns the data associated with the key from batch.
func (wb *WriteBatchWI) Get(opts *Options, key []byte) (slice *Slice, err error) {
	var (
		cErr    *C.char
		cValLen C.size_t
		cKey    = refGoBytes(key)
	)

	cValue := C.rocksdb_writebatch_wi_get_from_batch(wb.c, opts.c, cKey, C.size_t(len(key)), &cValLen, &cErr)
	if err = fromCError(cErr); err == nil {
		slice = NewSlice(cValue, cValLen)
	}

	return
}

// GetWithCF returns the data associated with the key from batch.
// Key belongs to specific column family.
func (wb *WriteBatchWI) GetWithCF(opts *Options, cf *ColumnFamilyHandle, key []byte) (slice *Slice, err error) {
	var (
		cErr    *C.char
		cValLen C.size_t
		cKey    = refGoBytes(key)
	)

	cValue := C.rocksdb_writebatch_wi_get_from_batch_cf(wb.c, opts.c, cf.c, cKey, C.size_t(len(key)), &cValLen, &cErr)
	if err = fromCError(cErr); err == nil {
		slice = NewSlice(cValue, cValLen)
	}

	return
}

// GetFromDB returns the data associated with the key from the database and write batch.
func (wb *WriteBatchWI) GetFromDB(db *DB, opts *ReadOptions, key []byte) (slice *Slice, err error) {
	var (
		cErr    *C.char
		cValLen C.size_t
		cKey    = refGoBytes(key)
	)

	cValue := C.rocksdb_writebatch_wi_get_from_batch_and_db(wb.c, db.c, opts.c, cKey, C.size_t(len(key)), &cValLen, &cErr)
	if err = fromCError(cErr); err == nil {
		slice = NewSlice(cValue, cValLen)
	}

	return
}

// GetFromDBWithCF returns the data associated with the key from the database and write batch.
// Key belongs to specific column family.
func (wb *WriteBatchWI) GetFromDBWithCF(db *DB, opts *ReadOptions, cf *ColumnFamilyHandle, key []byte) (slice *Slice, err error) {
	var (
		cErr    *C.char
		cValLen C.size_t
		cKey    = refGoBytes(key)
	)

	cValue := C.rocksdb_writebatch_wi_get_from_batch_and_db_cf(wb.c, db.c, opts.c, cf.c, cKey, C.size_t(len(key)), &cValLen, &cErr)
	if err = fromCError(cErr); err == nil {
		slice = NewSlice(cValue, cValLen)
	}

	return
}

// NewIteratorWithBase will create a new Iterator that will use WBWIIterator as a delta and
// base_iterator as base.
//
// This function is only supported if the WriteBatchWithIndex was
// constructed with overwrite_key=true.
//
// The returned iterator should be deleted by the caller.
// The base_iterator is now 'owned' by the returned iterator. Deleting the
// returned iterator will also delete the base_iterator.
//
// Updating write batch with the current key of the iterator is not safe.
// We strongly recommend users not to do it. It will invalidate the current
// key() and value() of the iterator. This invalidation happens even before
// the write batch update finishes. The state may recover after Next() is
// called.
func (wb *WriteBatchWI) NewIteratorWithBase(db *DB, baseIter *Iterator) *Iterator {
	cIter := C.rocksdb_writebatch_wi_create_iterator_with_base(wb.c, baseIter.c)
	return newNativeIterator(cIter)
}

// NewIteratorWithBaseCF will create a new Iterator that will use WBWIIterator as a delta and
// base_iterator as base.
//
// This function is only supported if the WriteBatchWithIndex was
// constructed with overwrite_key=true.
//
// The returned iterator should be deleted by the caller.
// The base_iterator is now 'owned' by the returned iterator. Deleting the
// returned iterator will also delete the base_iterator.
//
// Updating write batch with the current key of the iterator is not safe.
// We strongly recommend users not to do it. It will invalidate the current
// key() and value() of the iterator. This invalidation happens even before
// the write batch update finishes. The state may recover after Next() is
// called.
func (wb *WriteBatchWI) NewIteratorWithBaseCF(db *DB, baseIter *Iterator, cf *ColumnFamilyHandle) *Iterator {
	cIter := C.rocksdb_writebatch_wi_create_iterator_with_base_cf(wb.c, baseIter.c, cf.c)
	return newNativeIterator(cIter)
}

// Clear removes all the enqueued Put and Deletes.
func (wb *WriteBatchWI) Clear() {
	C.rocksdb_writebatch_wi_clear(wb.c)
}

// Destroy deallocates the WriteBatch object.
func (wb *WriteBatchWI) Destroy() {
	if wb.c != nil {
		C.rocksdb_writebatch_wi_destroy(wb.c)
		wb.c = nil
	}
}
