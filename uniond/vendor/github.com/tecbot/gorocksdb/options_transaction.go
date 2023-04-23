package gorocksdb

// #include "rocksdb/c.h"
import "C"

// TransactionOptions represent all of the available options options for
// a transaction on the database.
type TransactionOptions struct {
	c *C.rocksdb_transaction_options_t
}

// NewDefaultTransactionOptions creates a default TransactionOptions object.
func NewDefaultTransactionOptions() *TransactionOptions {
	return NewNativeTransactionOptions(C.rocksdb_transaction_options_create())
}

// NewNativeTransactionOptions creates a TransactionOptions object.
func NewNativeTransactionOptions(c *C.rocksdb_transaction_options_t) *TransactionOptions {
	return &TransactionOptions{c}
}

// SetSetSnapshot to true is the same as calling
// Transaction::SetSnapshot().
func (opts *TransactionOptions) SetSetSnapshot(value bool) {
	C.rocksdb_transaction_options_set_set_snapshot(opts.c, boolToChar(value))
}

// SetDeadlockDetect to true means that before acquiring locks, this transaction will
// check if doing so will cause a deadlock. If so, it will return with
// Status::Busy.  The user should retry their transaction.
func (opts *TransactionOptions) SetDeadlockDetect(value bool) {
	C.rocksdb_transaction_options_set_deadlock_detect(opts.c, boolToChar(value))
}

// SetLockTimeout positive, specifies the wait timeout in milliseconds when
// a transaction attempts to lock a key.
// If 0, no waiting is done if a lock cannot instantly be acquired.
// If negative, TransactionDBOptions::transaction_lock_timeout will be used
func (opts *TransactionOptions) SetLockTimeout(lock_timeout int64) {
	C.rocksdb_transaction_options_set_lock_timeout(opts.c, C.int64_t(lock_timeout))
}

// SetExpiration sets the Expiration duration in milliseconds.
// If non-negative, transactions that last longer than this many milliseconds will fail to commit.
// If not set, a forgotten transaction that is never committed, rolled back, or deleted
// will never relinquish any locks it holds.  This could prevent keys from
// being written by other writers.
func (opts *TransactionOptions) SetExpiration(expiration int64) {
	C.rocksdb_transaction_options_set_expiration(opts.c, C.int64_t(expiration))
}

// SetDeadlockDetectDepth sets the number of traversals to make during deadlock detection.
func (opts *TransactionOptions) SetDeadlockDetectDepth(depth int64) {
	C.rocksdb_transaction_options_set_deadlock_detect_depth(opts.c, C.int64_t(depth))
}

// SetMaxWriteBatchSize sets the maximum number of bytes used for the write batch. 0 means no limit.
func (opts *TransactionOptions) SetMaxWriteBatchSize(size uint64) {
	C.rocksdb_transaction_options_set_max_write_batch_size(opts.c, C.size_t(size))
}

// Destroy deallocates the TransactionOptions object.
func (opts *TransactionOptions) Destroy() {
	C.rocksdb_transaction_options_destroy(opts.c)
	opts.c = nil
}
