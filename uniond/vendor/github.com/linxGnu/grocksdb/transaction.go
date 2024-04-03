package grocksdb

// #include <stdlib.h>
// #include "rocksdb/c.h"
import "C"

import (
	"fmt"
)

// Transaction is used with TransactionDB for transaction support.
type Transaction struct {
	c *C.rocksdb_transaction_t
}

// NewNativeTransaction creates a Transaction object.
func newNativeTransaction(c *C.rocksdb_transaction_t) *Transaction {
	return &Transaction{c: c}
}

// SetName of transaction.
func (transaction *Transaction) SetName(name string) (err error) {
	var (
		cErr  *C.char
		name_ = refGoBytes([]byte(name))
	)

	C.rocksdb_transaction_set_name(transaction.c, name_, C.size_t(len(name)), &cErr)
	err = fromCError(cErr)

	return
}

// GetName of transaction.
func (transaction *Transaction) GetName() string {
	var len C.size_t
	cValue := C.rocksdb_transaction_get_name(transaction.c, &len)
	return toString(cValue, C.int(len))
}

// Prepare transaction.
func (transaction *Transaction) Prepare() (err error) {
	var cErr *C.char
	C.rocksdb_transaction_prepare(transaction.c, &cErr)
	err = fromCError(cErr)
	return
}

// Commit commits the transaction to the database.
func (transaction *Transaction) Commit() (err error) {
	var cErr *C.char
	C.rocksdb_transaction_commit(transaction.c, &cErr)
	err = fromCError(cErr)
	return
}

// Rollback performs a rollback on the transaction.
func (transaction *Transaction) Rollback() (err error) {
	var cErr *C.char
	C.rocksdb_transaction_rollback(transaction.c, &cErr)
	err = fromCError(cErr)
	return
}

// Get returns the data associated with the key from the database given this transaction.
func (transaction *Transaction) Get(opts *ReadOptions, key []byte) (slice *Slice, err error) {
	var (
		cErr    *C.char
		cValLen C.size_t
		cKey    = refGoBytes(key)
	)

	cValue := C.rocksdb_transaction_get(
		transaction.c, opts.c, cKey, C.size_t(len(key)), &cValLen, &cErr,
	)
	if err = fromCError(cErr); err == nil {
		slice = NewSlice(cValue, cValLen)
	}

	return
}

// GetPinned returns the data associated with the key from the transaction.
func (transaction *Transaction) GetPinned(opts *ReadOptions, key []byte) (handle *PinnableSliceHandle, err error) {
	var (
		cErr *C.char
		cKey = refGoBytes(key)
	)

	cHandle := C.rocksdb_transaction_get_pinned(transaction.c, opts.c, cKey, C.size_t(len(key)), &cErr)
	if err = fromCError(cErr); err == nil {
		handle = newNativePinnableSliceHandle(cHandle)
	}

	return
}

// GetWithCF returns the data associated with the key from the database, with column family, given this transaction.
func (transaction *Transaction) GetWithCF(opts *ReadOptions, cf *ColumnFamilyHandle, key []byte) (slice *Slice, err error) {
	var (
		cErr    *C.char
		cValLen C.size_t
		cKey    = refGoBytes(key)
	)

	cValue := C.rocksdb_transaction_get_cf(
		transaction.c, opts.c, cf.c, cKey, C.size_t(len(key)), &cValLen, &cErr,
	)
	if err = fromCError(cErr); err == nil {
		slice = NewSlice(cValue, cValLen)
	}

	return
}

// GetPinnedWithCF returns the data associated with the key from the transaction.
func (transaction *Transaction) GetPinnedWithCF(opts *ReadOptions, cf *ColumnFamilyHandle, key []byte) (handle *PinnableSliceHandle, err error) {
	var (
		cErr *C.char
		cKey = refGoBytes(key)
	)

	cHandle := C.rocksdb_transaction_get_pinned_cf(transaction.c, opts.c, cf.c, cKey, C.size_t(len(key)), &cErr)
	if err = fromCError(cErr); err == nil {
		handle = newNativePinnableSliceHandle(cHandle)
	}

	return
}

// GetForUpdate returns the data associated with the key and puts an exclusive lock on the key
// from the database given this transaction.
func (transaction *Transaction) GetForUpdate(opts *ReadOptions, key []byte) (slice *Slice, err error) {
	var (
		cErr    *C.char
		cValLen C.size_t
		cKey    = refGoBytes(key)
	)

	cValue := C.rocksdb_transaction_get_for_update(
		transaction.c, opts.c, cKey, C.size_t(len(key)), &cValLen, C.uchar(byte(1)) /*exclusive*/, &cErr,
	)
	if err = fromCError(cErr); err == nil {
		slice = NewSlice(cValue, cValLen)
	}

	return
}

// GetPinnedForUpdate returns the data associated with the key and puts an exclusive lock on the key
// from the database given this transaction.
func (transaction *Transaction) GetPinnedForUpdate(opts *ReadOptions, key []byte) (handle *PinnableSliceHandle, err error) {
	var (
		cErr *C.char
		cKey = refGoBytes(key)
	)

	cHandle := C.rocksdb_transaction_get_pinned_for_update(
		transaction.c, opts.c, cKey, C.size_t(len(key)),
		C.uchar(byte(1)), /*exclusive*/
		&cErr)
	if err = fromCError(cErr); err == nil {
		handle = newNativePinnableSliceHandle(cHandle)
	}

	return
}

// GetForUpdateWithCF queries the data associated with the key and puts an exclusive lock on the key
// from the database, with column family, given this transaction.
func (transaction *Transaction) GetForUpdateWithCF(opts *ReadOptions, cf *ColumnFamilyHandle, key []byte) (slice *Slice, err error) {
	var (
		cErr    *C.char
		cValLen C.size_t
		cKey    = refGoBytes(key)
	)

	cValue := C.rocksdb_transaction_get_for_update_cf(
		transaction.c, opts.c, cf.c, cKey, C.size_t(len(key)), &cValLen, C.uchar(byte(1)) /*exclusive*/, &cErr,
	)
	if err = fromCError(cErr); err == nil {
		slice = NewSlice(cValue, cValLen)
	}

	return
}

// GetPinnedForUpdateWithCF returns the data associated with the key and puts an exclusive lock on the key
// from the database given this transaction.
func (transaction *Transaction) GetPinnedForUpdateWithCF(opts *ReadOptions, cf *ColumnFamilyHandle, key []byte) (handle *PinnableSliceHandle, err error) {
	var (
		cErr *C.char
		cKey = refGoBytes(key)
	)

	cHandle := C.rocksdb_transaction_get_pinned_for_update_cf(
		transaction.c, opts.c, cf.c, cKey, C.size_t(len(key)),
		C.uchar(byte(1)), /*exclusive*/
		&cErr)
	if err = fromCError(cErr); err == nil {
		handle = newNativePinnableSliceHandle(cHandle)
	}

	return
}

// MultiGet returns the data associated with the passed keys from the transaction.
func (transaction *Transaction) MultiGet(opts *ReadOptions, keys ...[]byte) (Slices, error) {
	// will destroy `cKeys` before return
	cKeys, cKeySizes := byteSlicesToCSlices(keys)

	vals := make(charsSlice, len(keys))
	valSizes := make(sizeTSlice, len(keys))
	rocksErrs := make(charsSlice, len(keys))

	C.rocksdb_transaction_multi_get(
		transaction.c,
		opts.c,
		C.size_t(len(keys)),
		cKeys.c(),
		cKeySizes.c(),
		vals.c(),
		valSizes.c(),
		rocksErrs.c(),
	)

	var errs []error

	for i, rocksErr := range rocksErrs {
		if err := fromCError(rocksErr); err != nil {
			errs = append(errs, fmt.Errorf("getting %q failed: %v", string(keys[i]), err.Error()))
		}
	}

	if len(errs) > 0 {
		cKeys.Destroy()
		return nil, fmt.Errorf("failed to get %d keys, first error: %v", len(errs), errs[0])
	}

	slices := make(Slices, len(keys))
	for i, val := range vals {
		slices[i] = NewSlice(val, valSizes[i])
	}

	cKeys.Destroy()
	return slices, nil
}

// MultiGetWithCF returns the data associated with the passed keys from the transaction.
func (transaction *Transaction) MultiGetWithCF(opts *ReadOptions, cf *ColumnFamilyHandle, keys ...[]byte) (Slices, error) {
	// will destroy `cKeys` before return
	cKeys, cKeySizes := byteSlicesToCSlices(keys)

	vals := make(charsSlice, len(keys))
	valSizes := make(sizeTSlice, len(keys))
	rocksErrs := make(charsSlice, len(keys))

	cfs := make(ColumnFamilyHandles, len(keys))
	for i := range keys {
		cfs[i] = cf
	}

	C.rocksdb_transaction_multi_get_cf(
		transaction.c,
		opts.c,
		cfs.toCSlice().c(),
		C.size_t(len(keys)),
		cKeys.c(),
		cKeySizes.c(),
		vals.c(),
		valSizes.c(),
		rocksErrs.c(),
	)

	var errs []error

	for i, rocksErr := range rocksErrs {
		if err := fromCError(rocksErr); err != nil {
			errs = append(errs, fmt.Errorf("getting %q failed: %v", string(keys[i]), err.Error()))
		}
	}

	if len(errs) > 0 {
		cKeys.Destroy()
		return nil, fmt.Errorf("failed to get %d keys, first error: %v", len(errs), errs[0])
	}

	slices := make(Slices, len(keys))
	for i, val := range vals {
		slices[i] = NewSlice(val, valSizes[i])
	}

	cKeys.Destroy()
	return slices, nil
}

// Put writes data associated with a key to the transaction.
func (transaction *Transaction) Put(key, value []byte) (err error) {
	var (
		cErr   *C.char
		cKey   = refGoBytes(key)
		cValue = refGoBytes(value)
	)

	C.rocksdb_transaction_put(
		transaction.c, cKey, C.size_t(len(key)), cValue, C.size_t(len(value)), &cErr,
	)
	err = fromCError(cErr)

	return
}

// PutCF writes data associated with a key to the transaction. Key belongs to column family.
func (transaction *Transaction) PutCF(cf *ColumnFamilyHandle, key, value []byte) (err error) {
	var (
		cErr   *C.char
		cKey   = refGoBytes(key)
		cValue = refGoBytes(value)
	)

	C.rocksdb_transaction_put_cf(
		transaction.c, cf.c, cKey, C.size_t(len(key)), cValue, C.size_t(len(value)), &cErr,
	)
	err = fromCError(cErr)

	return
}

// Merge key, value to the transaction.
func (transaction *Transaction) Merge(key, value []byte) (err error) {
	var (
		cErr   *C.char
		cKey   = refGoBytes(key)
		cValue = refGoBytes(value)
	)

	C.rocksdb_transaction_merge(
		transaction.c, cKey, C.size_t(len(key)), cValue, C.size_t(len(value)), &cErr,
	)
	err = fromCError(cErr)

	return
}

// MergeCF key, value to the transaction on specific column family.
func (transaction *Transaction) MergeCF(cf *ColumnFamilyHandle, key, value []byte) (err error) {
	var (
		cErr   *C.char
		cKey   = refGoBytes(key)
		cValue = refGoBytes(value)
	)

	C.rocksdb_transaction_merge_cf(
		transaction.c, cf.c, cKey, C.size_t(len(key)), cValue, C.size_t(len(value)), &cErr,
	)
	err = fromCError(cErr)

	return
}

// Delete removes the data associated with the key from the transaction.
func (transaction *Transaction) Delete(key []byte) (err error) {
	var (
		cErr *C.char
		cKey = refGoBytes(key)
	)

	C.rocksdb_transaction_delete(transaction.c, cKey, C.size_t(len(key)), &cErr)
	err = fromCError(cErr)

	return
}

// DeleteCF removes the data associated with the key (belongs to specific column family) from the transaction.
func (transaction *Transaction) DeleteCF(cf *ColumnFamilyHandle, key []byte) (err error) {
	var (
		cErr *C.char
		cKey = refGoBytes(key)
	)

	C.rocksdb_transaction_delete_cf(transaction.c, cf.c, cKey, C.size_t(len(key)), &cErr)
	err = fromCError(cErr)

	return
}

// NewIterator returns an iterator that will iterate on all keys in the default
// column family including both keys in the DB and uncommitted keys in this
// transaction.
//
// Setting read_options.snapshot will affect what is read from the
// DB but will NOT change which keys are read from this transaction (the keys
// in this transaction do not yet belong to any snapshot and will be fetched
// regardless).
//
// Caller is responsible for deleting the returned Iterator.
func (transaction *Transaction) NewIterator(opts *ReadOptions) *Iterator {
	return newNativeIterator(C.rocksdb_transaction_create_iterator(transaction.c, opts.c))
}

// NewIteratorCF returns an iterator that will iterate on all keys in the specific
// column family including both keys in the DB and uncommitted keys in this
// transaction.
//
// Setting read_options.snapshot will affect what is read from the
// DB but will NOT change which keys are read from this transaction (the keys
// in this transaction do not yet belong to any snapshot and will be fetched
// regardless).
//
// Caller is responsible for deleting the returned Iterator.
func (transaction *Transaction) NewIteratorCF(opts *ReadOptions, cf *ColumnFamilyHandle) *Iterator {
	return newNativeIterator(C.rocksdb_transaction_create_iterator_cf(transaction.c, opts.c, cf.c))
}

// SetSavePoint records the state of the transaction for future calls to
// RollbackToSavePoint().  May be called multiple times to set multiple save
// points.
func (transaction *Transaction) SetSavePoint() {
	C.rocksdb_transaction_set_savepoint(transaction.c)
}

// RollbackToSavePoint undo all operations in this transaction (Put, Merge, Delete, PutLogData)
// since the most recent call to SetSavePoint() and removes the most recent
// SetSavePoint().
func (transaction *Transaction) RollbackToSavePoint() (err error) {
	var cErr *C.char
	C.rocksdb_transaction_rollback_to_savepoint(transaction.c, &cErr)
	err = fromCError(cErr)
	return
}

// GetSnapshot returns the Snapshot created by the last call to SetSnapshot().
func (transaction *Transaction) GetSnapshot() *Snapshot {
	return newNativeSnapshot(C.rocksdb_transaction_get_snapshot(transaction.c))
}

// Destroy deallocates the transaction object.
func (transaction *Transaction) Destroy() {
	C.rocksdb_transaction_destroy(transaction.c)
	transaction.c = nil
}

// GetWriteBatchWI returns underlying write batch wi.
func (transaction *Transaction) GetWriteBatchWI() *WriteBatchWI {
	wi := C.rocksdb_transaction_get_writebatch_wi(transaction.c)
	return newNativeWriteBatchWI(wi)
}

// RebuildFromWriteBatch rebuilds transaction from write_batch.
// Note: If no error, write_batch will be destroyed. It's move-op (see also: C++ Move)
func (transaction *Transaction) RebuildFromWriteBatch(wb *WriteBatch) (err error) {
	var cErr *C.char
	C.rocksdb_transaction_rebuild_from_writebatch(transaction.c, wb.c, &cErr)
	err = fromCError(cErr)
	if err == nil {
		wb.Destroy()
	}
	return
}

// RebuildFromWriteBatchWI rebuilds transaction from write_batch.
// Note: If no error, write_batch will be destroyed. It's move-op (see also: C++ Move)
func (transaction *Transaction) RebuildFromWriteBatchWI(wb *WriteBatchWI) (err error) {
	var cErr *C.char
	C.rocksdb_transaction_rebuild_from_writebatch_wi(transaction.c, wb.c, &cErr)
	err = fromCError(cErr)
	if err == nil {
		wb.Destroy()
	}
	return
}

// SetCommitTimestamp sets the commit timestamp for the transaction.
// If a transaction's write batch includes at least one key for a column family that enables user-defined timestamp,
// then the transaction must be assigned a commit timestamp in order to commit.
// SetCommitTimestamp should be called before transaction commits.
// If two-phase commit (2PC) is enabled, then SetCommitTimestamp should be called after Transaction Prepare succeeds.
func (transaction *Transaction) SetCommitTimestamp(ts uint64) {
	C.rocksdb_transaction_set_commit_timestamp(transaction.c, C.uint64_t(ts))
}

// SetReadTimestampForValidation sets the read timestamp for the transaction.
// Each transaction can have a read timestamp.
// The transaction will use this timestamp to read data from the database.
// Any data with timestamp after this read timestamp should be considered invisible to this transaction.
// The same read timestamp is also used for validation.
func (transaction *Transaction) SetReadTimestampForValidation(ts uint64) {
	C.rocksdb_transaction_set_read_timestamp_for_validation(transaction.c, C.uint64_t(ts))
}
