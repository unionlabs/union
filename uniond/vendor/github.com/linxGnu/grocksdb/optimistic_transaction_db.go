package grocksdb

// #include <stdlib.h>
// #include "rocksdb/c.h"
import "C"

import (
	"unsafe"
)

// OptimisticTransactionDB is a reusable handle to a RocksDB optimistic transactional database on disk.
type OptimisticTransactionDB struct {
	c    *C.rocksdb_optimistictransactiondb_t
	name string
	opts *Options
}

// OpenOptimisticTransactionDb opens a database with the specified options.
func OpenOptimisticTransactionDb(
	opts *Options,
	name string,
) (tdb *OptimisticTransactionDB, err error) {
	var (
		cErr  *C.char
		cName = C.CString(name)
	)

	db := C.rocksdb_optimistictransactiondb_open(
		opts.c, cName, &cErr)
	if err = fromCError(cErr); err == nil {
		tdb = &OptimisticTransactionDB{
			name: name,
			c:    db,
			opts: opts,
		}
	}

	C.free(unsafe.Pointer(cName))
	return
}

// OpenOptimisticTransactionDbColumnFamilies opens a database with the specified column families.
func OpenOptimisticTransactionDbColumnFamilies(
	opts *Options,
	name string,
	cfNames []string,
	cfOpts []*Options,
) (db *OptimisticTransactionDB, cfHandles []*ColumnFamilyHandle, err error) {
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
	_db := C.rocksdb_optimistictransactiondb_open_column_families(
		opts.c,
		cName,
		C.int(numColumnFamilies),
		&cNames[0],
		&cOpts[0],
		&cHandles[0],
		&cErr,
	)
	if err = fromCError(cErr); err == nil {
		db = &OptimisticTransactionDB{
			name: name,
			c:    _db,
			opts: opts,
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

// TransactionBegin begins a new transaction
// with the WriteOptions and TransactionOptions given.
func (db *OptimisticTransactionDB) TransactionBegin(
	opts *WriteOptions,
	transactionOpts *OptimisticTransactionOptions,
	oldTransaction *Transaction,
) *Transaction {
	if oldTransaction != nil {
		cTx := C.rocksdb_optimistictransaction_begin(
			db.c,
			opts.c,
			transactionOpts.c,
			oldTransaction.c,
		)
		return newNativeTransaction(cTx)
	}

	cTx := C.rocksdb_optimistictransaction_begin(db.c, opts.c, transactionOpts.c, nil)
	return newNativeTransaction(cTx)
}

// NewCheckpoint creates a new Checkpoint for this db.
func (db *OptimisticTransactionDB) NewCheckpoint() (cp *Checkpoint, err error) {
	var cErr *C.char

	cCheckpoint := C.rocksdb_optimistictransactiondb_checkpoint_object_create(
		db.c, &cErr,
	)
	if err = fromCError(cErr); err == nil {
		cp = newNativeCheckpoint(cCheckpoint)
	}

	return
}

// Write batch.
func (db *OptimisticTransactionDB) Write(opts *WriteOptions, batch *WriteBatch) (err error) {
	var cErr *C.char

	C.rocksdb_optimistictransactiondb_write(db.c, opts.c, batch.c, &cErr)
	err = fromCError(cErr)

	return
}

// Close closes the database.
func (db *OptimisticTransactionDB) Close() {
	C.rocksdb_optimistictransactiondb_close(db.c)
	db.c = nil
}

// GetBaseDB returns base-database.
func (db *OptimisticTransactionDB) GetBaseDB() *DB {
	return &DB{
		c:    C.rocksdb_optimistictransactiondb_get_base_db(db.c),
		name: db.name,
		opts: db.opts,
	}
}

// CloseBaseDB closes base-database.
func (db *OptimisticTransactionDB) CloseBaseDB(base *DB) {
	C.rocksdb_optimistictransactiondb_close_base_db(base.c)
	base.c = nil
}
