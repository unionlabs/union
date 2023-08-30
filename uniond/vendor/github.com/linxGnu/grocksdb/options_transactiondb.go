package grocksdb

// #include "rocksdb/c.h"
import "C"

// TransactionDBOptions represent all of the available options when opening a transactional database
// with OpenTransactionDb.
type TransactionDBOptions struct {
	c *C.rocksdb_transactiondb_options_t
}

// NewDefaultTransactionDBOptions creates a default TransactionDBOptions object.
func NewDefaultTransactionDBOptions() *TransactionDBOptions {
	return newNativeTransactionDBOptions(C.rocksdb_transactiondb_options_create())
}

// NewNativeTransactionDBOptions creates a TransactionDBOptions from native object.
func newNativeTransactionDBOptions(c *C.rocksdb_transactiondb_options_t) *TransactionDBOptions {
	return &TransactionDBOptions{c: c}
}

// SetMaxNumLocks sets the maximum number of keys that can be locked at the same time
// per column family.
// If the number of locked keys is greater than max_num_locks, transaction
// writes (or GetForUpdate) will return an error.
// If this value is not positive, no limit will be enforced.
func (opts *TransactionDBOptions) SetMaxNumLocks(maxNumLocks int64) {
	C.rocksdb_transactiondb_options_set_max_num_locks(opts.c, C.int64_t(maxNumLocks))
}

// SetNumStripes sets the concurrency level.
// Increasing this value will increase the concurrency by dividing the lock
// table (per column family) into more sub-tables, each with their own
// separate
// mutex.
func (opts *TransactionDBOptions) SetNumStripes(numStripes uint64) {
	C.rocksdb_transactiondb_options_set_num_stripes(opts.c, C.size_t(numStripes))
}

// SetTransactionLockTimeout if positive, specifies the default wait timeout in milliseconds when
// a transaction attempts to lock a key if not specified by
// TransactionOptions::lock_timeout.
//
// If 0, no waiting is done if a lock cannot instantly be acquired.
// If negative, there is no timeout.  Not using a timeout is not recommended
// as it can lead to deadlocks.  Currently, there is no deadlock-detection to
// recover from a deadlock.
func (opts *TransactionDBOptions) SetTransactionLockTimeout(txnLockTimeout int64) {
	C.rocksdb_transactiondb_options_set_transaction_lock_timeout(opts.c, C.int64_t(txnLockTimeout))
}

// SetDefaultLockTimeout if posititve, specifies the wait timeout in milliseconds when writing a key
// OUTSIDE of a transaction (ie by calling DB::Put(),Merge(),Delete(),Write()
// directly).
// If 0, no waiting is done if a lock cannot instantly be acquired.
// If negative, there is no timeout and will block indefinitely when acquiring
// a lock.
//
// Not using a timeout can lead to deadlocks.  Currently, there
// is no deadlock-detection to recover from a deadlock.  While DB writes
// cannot deadlock with other DB writes, they can deadlock with a transaction.
// A negative timeout should only be used if all transactions have a small
// expiration set.
func (opts *TransactionDBOptions) SetDefaultLockTimeout(defaultLockTimeout int64) {
	C.rocksdb_transactiondb_options_set_default_lock_timeout(opts.c, C.int64_t(defaultLockTimeout))
}

// Destroy deallocates the TransactionDBOptions object.
func (opts *TransactionDBOptions) Destroy() {
	C.rocksdb_transactiondb_options_destroy(opts.c)
	opts.c = nil
}
