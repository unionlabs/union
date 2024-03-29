package grocksdb

// #include <stdlib.h>
// #include "rocksdb/c.h"
import "C"

import (
	"fmt"
	"unsafe"
)

// TransactionDB is a reusable handle to a RocksDB transactional database on disk, created by OpenTransactionDb.
type TransactionDB struct {
	c                 *C.rocksdb_transactiondb_t
	name              string
	opts              *Options
	transactionDBOpts *TransactionDBOptions
}

// OpenTransactionDb opens a database with the specified options.
func OpenTransactionDb(
	opts *Options,
	transactionDBOpts *TransactionDBOptions,
	name string,
) (tdb *TransactionDB, err error) {
	var (
		cErr  *C.char
		cName = C.CString(name)
	)

	db := C.rocksdb_transactiondb_open(
		opts.c, transactionDBOpts.c, cName, &cErr)
	if err = fromCError(cErr); err == nil {
		tdb = &TransactionDB{
			name:              name,
			c:                 db,
			opts:              opts,
			transactionDBOpts: transactionDBOpts,
		}
	}

	C.free(unsafe.Pointer(cName))
	return
}

// OpenTransactionDbColumnFamilies opens a database with the specified column families.
func OpenTransactionDbColumnFamilies(
	opts *Options,
	transactionDBOpts *TransactionDBOptions,
	name string,
	cfNames []string,
	cfOpts []*Options,
) (db *TransactionDB, cfHandles []*ColumnFamilyHandle, err error) {
	numColumnFamilies := len(cfNames)
	if numColumnFamilies != len(cfOpts) {
		err = ErrColumnFamilyMustMatch
		return
	}

	cName := C.CString(name)
	cNames := make([]*C.char, numColumnFamilies)
	for i, s := range cfNames {
		cNames[i] = C.CString(s)
	}

	cOpts := make([]*C.rocksdb_options_t, numColumnFamilies)
	for i, o := range cfOpts {
		cOpts[i] = o.c
	}

	cHandles := make([]*C.rocksdb_column_family_handle_t, numColumnFamilies)

	var cErr *C.char
	_db := C.rocksdb_transactiondb_open_column_families(
		opts.c,
		transactionDBOpts.c,
		cName,
		C.int(numColumnFamilies),
		&cNames[0],
		&cOpts[0],
		&cHandles[0],
		&cErr,
	)
	if err = fromCError(cErr); err == nil {
		db = &TransactionDB{
			name:              name,
			c:                 _db,
			opts:              opts,
			transactionDBOpts: transactionDBOpts,
		}
		cfHandles = make([]*ColumnFamilyHandle, numColumnFamilies)
		for i, c := range cHandles {
			cfHandles[i] = newNativeColumnFamilyHandle(c)
		}
	}

	C.free(unsafe.Pointer(cName))
	for _, s := range cNames {
		C.free(unsafe.Pointer(s))
	}
	return
}

// NewSnapshot creates a new snapshot of the database.
func (db *TransactionDB) NewSnapshot() *Snapshot {
	return newNativeSnapshot(C.rocksdb_transactiondb_create_snapshot(db.c))
}

// ReleaseSnapshot releases the snapshot and its resources.
func (db *TransactionDB) ReleaseSnapshot(snapshot *Snapshot) {
	C.rocksdb_transactiondb_release_snapshot(db.c, snapshot.c)
	snapshot.c = nil
}

// GetProperty returns the value of a database property.
func (db *TransactionDB) GetProperty(propName string) (value string) {
	cprop := C.CString(propName)
	cValue := C.rocksdb_transactiondb_property_value(db.c, cprop)

	value = C.GoString(cValue)

	C.rocksdb_free(unsafe.Pointer(cValue))
	C.free(unsafe.Pointer(cprop))
	return
}

// GetIntProperty similar to `GetProperty`, but only works for a subset of properties whose
// return value is an integer. Return the value by integer.
func (db *TransactionDB) GetIntProperty(propName string) (value uint64, success bool) {
	cProp := C.CString(propName)
	success = C.rocksdb_transactiondb_property_int(db.c, cProp, (*C.uint64_t)(&value)) == 0
	C.free(unsafe.Pointer(cProp))
	return
}

// GetBaseDB gets base db.
func (db *TransactionDB) GetBaseDB() *DB {
	base := C.rocksdb_transactiondb_get_base_db(db.c)
	return &DB{c: base}
}

// CloseBaseDBOfTransactionDB closes base db of TransactionDB.
func CloseBaseDBOfTransactionDB(db *DB) {
	if db != nil && db.c != nil {
		C.rocksdb_transactiondb_close_base_db(db.c)
	}
}

// TransactionBegin begins a new transaction
// with the WriteOptions and TransactionOptions given.
func (db *TransactionDB) TransactionBegin(
	opts *WriteOptions,
	transactionOpts *TransactionOptions,
	oldTransaction *Transaction,
) *Transaction {
	if oldTransaction != nil {
		cTx := C.rocksdb_transaction_begin(
			db.c,
			opts.c,
			transactionOpts.c,
			oldTransaction.c,
		)
		return newNativeTransaction(cTx)
	}

	cTx := C.rocksdb_transaction_begin(db.c, opts.c, transactionOpts.c, nil)
	return newNativeTransaction(cTx)
}

// Get returns the data associated with the key from the database.
func (db *TransactionDB) Get(opts *ReadOptions, key []byte) (slice *Slice, err error) {
	var (
		cErr    *C.char
		cValLen C.size_t
		cKey    = refGoBytes(key)
	)

	cValue := C.rocksdb_transactiondb_get(
		db.c, opts.c, cKey, C.size_t(len(key)), &cValLen, &cErr,
	)
	if err = fromCError(cErr); err == nil {
		slice = NewSlice(cValue, cValLen)
	}

	return
}

// GetPinned returns the data associated with the key from the database.
func (db *TransactionDB) GetPinned(opts *ReadOptions, key []byte) (handle *PinnableSliceHandle, err error) {
	var (
		cErr *C.char
		cKey = refGoBytes(key)
	)

	cHandle := C.rocksdb_transactiondb_get_pinned(db.c, opts.c, cKey, C.size_t(len(key)), &cErr)
	if err = fromCError(cErr); err == nil {
		handle = newNativePinnableSliceHandle(cHandle)
	}

	return
}

// GetCF returns the data associated with the key from the database, from column family.
func (db *TransactionDB) GetCF(opts *ReadOptions, cf *ColumnFamilyHandle, key []byte) (slice *Slice, err error) {
	var (
		cErr    *C.char
		cValLen C.size_t
		cKey    = refGoBytes(key)
	)

	cValue := C.rocksdb_transactiondb_get_cf(
		db.c, opts.c, cf.c, cKey, C.size_t(len(key)), &cValLen, &cErr,
	)
	if err = fromCError(cErr); err == nil {
		slice = NewSlice(cValue, cValLen)
	}

	return
}

// GetPinnedWithCF returns the data associated with the key from the database.
func (db *TransactionDB) GetPinnedWithCF(opts *ReadOptions, cf *ColumnFamilyHandle, key []byte) (handle *PinnableSliceHandle, err error) {
	var (
		cErr *C.char
		cKey = refGoBytes(key)
	)

	cHandle := C.rocksdb_transactiondb_get_pinned_cf(db.c, opts.c, cf.c, cKey, C.size_t(len(key)), &cErr)
	if err = fromCError(cErr); err == nil {
		handle = newNativePinnableSliceHandle(cHandle)
	}

	return
}

// MultiGet returns the data associated with the passed keys from the database.
func (db *TransactionDB) MultiGet(opts *ReadOptions, keys ...[]byte) (Slices, error) {
	// will destroy `cKeys` before return
	cKeys, cKeySizes := byteSlicesToCSlices(keys)

	vals := make(charsSlice, len(keys))
	valSizes := make(sizeTSlice, len(keys))
	rocksErrs := make(charsSlice, len(keys))

	C.rocksdb_transactiondb_multi_get(
		db.c,
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

// MultiGetWithCF returns the data associated with the passed keys from the database.
func (db *TransactionDB) MultiGetWithCF(opts *ReadOptions, cf *ColumnFamilyHandle, keys ...[]byte) (Slices, error) {
	// will destroy `cKeys` before return
	cKeys, cKeySizes := byteSlicesToCSlices(keys)

	vals := make(charsSlice, len(keys))
	valSizes := make(sizeTSlice, len(keys))
	rocksErrs := make(charsSlice, len(keys))

	cfs := make(ColumnFamilyHandles, len(keys))
	for i := range keys {
		cfs[i] = cf
	}

	C.rocksdb_transactiondb_multi_get_cf(
		db.c,
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

// Put writes data associated with a key to the database.
func (db *TransactionDB) Put(opts *WriteOptions, key, value []byte) (err error) {
	var (
		cErr   *C.char
		cKey   = refGoBytes(key)
		cValue = refGoBytes(value)
	)

	C.rocksdb_transactiondb_put(
		db.c, opts.c, cKey, C.size_t(len(key)), cValue, C.size_t(len(value)), &cErr,
	)
	err = fromCError(cErr)

	return
}

// PutCF writes data associated with a key to the database on specific column family.
func (db *TransactionDB) PutCF(opts *WriteOptions, cf *ColumnFamilyHandle, key, value []byte) (err error) {
	var (
		cErr   *C.char
		cKey   = refGoBytes(key)
		cValue = refGoBytes(value)
	)

	C.rocksdb_transactiondb_put_cf(
		db.c, opts.c, cf.c, cKey, C.size_t(len(key)), cValue, C.size_t(len(value)), &cErr,
	)
	err = fromCError(cErr)

	return
}

// Merge writes data associated with a key to the database.
func (db *TransactionDB) Merge(opts *WriteOptions, key, value []byte) (err error) {
	var (
		cErr   *C.char
		cKey   = refGoBytes(key)
		cValue = refGoBytes(value)
	)

	C.rocksdb_transactiondb_merge(
		db.c, opts.c, cKey, C.size_t(len(key)), cValue, C.size_t(len(value)), &cErr,
	)
	err = fromCError(cErr)

	return
}

// MergeCF writes data associated with a key to the database on specific column family.
func (db *TransactionDB) MergeCF(opts *WriteOptions, cf *ColumnFamilyHandle, key, value []byte) (err error) {
	var (
		cErr   *C.char
		cKey   = refGoBytes(key)
		cValue = refGoBytes(value)
	)

	C.rocksdb_transactiondb_merge_cf(
		db.c, opts.c, cf.c, cKey, C.size_t(len(key)), cValue, C.size_t(len(value)), &cErr,
	)
	err = fromCError(cErr)

	return
}

// Delete removes the data associated with the key from the database.
func (db *TransactionDB) Delete(opts *WriteOptions, key []byte) (err error) {
	var (
		cErr *C.char
		cKey = refGoBytes(key)
	)

	C.rocksdb_transactiondb_delete(db.c, opts.c, cKey, C.size_t(len(key)), &cErr)
	err = fromCError(cErr)

	return
}

// DeleteCF removes the data associated with the key from the database on specific column family.
func (db *TransactionDB) DeleteCF(opts *WriteOptions, cf *ColumnFamilyHandle, key []byte) (err error) {
	var (
		cErr *C.char
		cKey = refGoBytes(key)
	)

	C.rocksdb_transactiondb_delete_cf(db.c, opts.c, cf.c, cKey, C.size_t(len(key)), &cErr)
	err = fromCError(cErr)

	return
}

// NewCheckpoint creates a new Checkpoint for this db.
func (db *TransactionDB) NewCheckpoint() (cp *Checkpoint, err error) {
	var cErr *C.char

	cCheckpoint := C.rocksdb_transactiondb_checkpoint_object_create(
		db.c, &cErr,
	)
	if err = fromCError(cErr); err == nil {
		cp = newNativeCheckpoint(cCheckpoint)
	}

	return
}

// CreateColumnFamily create a new column family.
func (db *TransactionDB) CreateColumnFamily(opts *Options, name string) (handle *ColumnFamilyHandle, err error) {
	var (
		cErr  *C.char
		cName = C.CString(name)
	)

	cHandle := C.rocksdb_transactiondb_create_column_family(db.c, opts.c, cName, &cErr)
	if err = fromCError(cErr); err == nil {
		handle = newNativeColumnFamilyHandle(cHandle)
	}

	C.free(unsafe.Pointer(cName))
	return
}

// Write writes a WriteBatch to the database.
func (db *TransactionDB) Write(opts *WriteOptions, batch *WriteBatch) (err error) {
	var cErr *C.char

	C.rocksdb_transactiondb_write(db.c, opts.c, batch.c, &cErr)
	err = fromCError(cErr)

	return
}

// Flush triggers a manual flush for the database.
func (db *TransactionDB) Flush(opts *FlushOptions) (err error) {
	var cErr *C.char

	C.rocksdb_transactiondb_flush(db.c, opts.c, &cErr)
	err = fromCError(cErr)

	return
}

// FlushCF triggers a manual flush for the database on specific column family.
func (db *TransactionDB) FlushCF(cf *ColumnFamilyHandle, opts *FlushOptions) (err error) {
	var cErr *C.char

	C.rocksdb_transactiondb_flush_cf(db.c, opts.c, cf.c, &cErr)
	err = fromCError(cErr)

	return
}

// FlushCFs triggers a manual flush for the database on specific column families.
func (db *TransactionDB) FlushCFs(cfs []*ColumnFamilyHandle, opts *FlushOptions) (err error) {
	if n := len(cfs); n > 0 {
		_cfs := make([]*C.rocksdb_column_family_handle_t, n)
		for i := range _cfs {
			_cfs[i] = cfs[i].c
		}

		var cErr *C.char
		C.rocksdb_transactiondb_flush_cfs(db.c, opts.c, &_cfs[0], C.int(n), &cErr)
		err = fromCError(cErr)
	}
	return
}

// FlushWAL flushes the WAL memory buffer to the file. If sync is true, it calls SyncWAL
// afterwards.
func (db *TransactionDB) FlushWAL(sync bool) (err error) {
	var cErr *C.char

	C.rocksdb_transactiondb_flush_wal(db.c, boolToChar(sync), &cErr)
	err = fromCError(cErr)

	return
}

// NewIterator returns an Iterator over the the database that uses the
// ReadOptions given.
func (db *TransactionDB) NewIterator(opts *ReadOptions) *Iterator {
	cIter := C.rocksdb_transactiondb_create_iterator(db.c, opts.c)
	return newNativeIterator(cIter)
}

// NewIteratorCF returns an Iterator over the the database and column family
// that uses the ReadOptions given.
func (db *TransactionDB) NewIteratorCF(opts *ReadOptions, cf *ColumnFamilyHandle) *Iterator {
	cIter := C.rocksdb_transactiondb_create_iterator_cf(db.c, opts.c, cf.c)
	return newNativeIterator(cIter)
}

// Close closes the database.
func (db *TransactionDB) Close() {
	C.rocksdb_transactiondb_close(db.c)
	db.c = nil
}
