package db

import "errors"

var (
	// errBatchClosed is returned when a closed or written batch is used.
	errBatchClosed = errors.New("batch has been written or closed")

	// errKeyEmpty is returned when attempting to use an empty or nil key.
	errKeyEmpty = errors.New("key cannot be empty")

	// errValueNil is returned when attempting to set a nil value.
	errValueNil = errors.New("value cannot be nil")
)

// DB is the main interface for all database backends. DBs are concurrency-safe. Callers must call
// Close on the database when done.
//
// Keys cannot be nil or empty, while values cannot be nil. Keys and values should be considered
// read-only, both when returned and when given, and must be copied before they are modified.
type DB interface {
	// Get fetches the value of the given key, or nil if it does not exist.
	// CONTRACT: key, value readonly []byte
	Get([]byte) ([]byte, error)

	// Has checks if a key exists.
	// CONTRACT: key, value readonly []byte
	Has(key []byte) (bool, error)

	// Set sets the value for the given key, replacing it if it already exists.
	// CONTRACT: key, value readonly []byte
	Set([]byte, []byte) error

	// SetSync sets the value for the given key, and flushes it to storage before returning.
	SetSync([]byte, []byte) error

	// Delete deletes the key, or does nothing if the key does not exist.
	// CONTRACT: key readonly []byte
	Delete([]byte) error

	// DeleteSync deletes the key, and flushes the delete to storage before returning.
	DeleteSync([]byte) error

	// Iterator returns an iterator over a domain of keys, in ascending order. The caller must call
	// Close when done. End is exclusive, and start must be less than end. A nil start iterates
	// from the first key, and a nil end iterates to the last key (inclusive). Empty keys are not
	// valid.
	// CONTRACT: No writes may happen within a domain while an iterator exists over it.
	// CONTRACT: start, end readonly []byte
	Iterator(start, end []byte) (Iterator, error)

	// ReverseIterator returns an iterator over a domain of keys, in descending order. The caller
	// must call Close when done. End is exclusive, and start must be less than end. A nil end
	// iterates from the last key (inclusive), and a nil start iterates to the first key (inclusive).
	// Empty keys are not valid.
	// CONTRACT: No writes may happen within a domain while an iterator exists over it.
	// CONTRACT: start, end readonly []byte
	ReverseIterator(start, end []byte) (Iterator, error)

	// Close closes the database connection.
	Close() error

	// NewBatch creates a batch for atomic updates. The caller must call Batch.Close.
	NewBatch() Batch

	// Print is used for debugging.
	Print() error

	// Stats returns a map of property values for all keys and the size of the cache.
	Stats() map[string]string
}

// Batch represents a group of writes. They may or may not be written atomically depending on the
// backend. Callers must call Close on the batch when done.
//
// As with DB, given keys and values should be considered read-only, and must not be modified after
// passing them to the batch.
type Batch interface {
	// Set sets a key/value pair.
	// CONTRACT: key, value readonly []byte
	Set(key, value []byte) error

	// Delete deletes a key/value pair.
	// CONTRACT: key readonly []byte
	Delete(key []byte) error

	// Write writes the batch, possibly without flushing to disk. Only Close() can be called after,
	// other methods will error.
	Write() error

	// WriteSync writes the batch and flushes it to disk. Only Close() can be called after, other
	// methods will error.
	WriteSync() error

	// Close closes the batch. It is idempotent, but calls to other methods afterwards will error.
	Close() error
}

// Iterator represents an iterator over a domain of keys. Callers must call Close when done.
// No writes can happen to a domain while there exists an iterator over it, some backends may take
// out database locks to ensure this will not happen.
//
// Callers must make sure the iterator is valid before calling any methods on it, otherwise
// these methods will panic. This is in part caused by most backend databases using this convention.
//
// As with DB, keys and values should be considered read-only, and must be copied before they are
// modified.
//
// Typical usage:
//
// var itr Iterator = ...
// defer itr.Close()
//
//	for ; itr.Valid(); itr.Next() {
//	  k, v := itr.Key(); itr.Value()
//	  ...
//	}
//
//	if err := itr.Error(); err != nil {
//	  ...
//	}
type Iterator interface {
	// Domain returns the start (inclusive) and end (exclusive) limits of the iterator.
	// CONTRACT: start, end readonly []byte
	Domain() (start []byte, end []byte)

	// Valid returns whether the current iterator is valid. Once invalid, the Iterator remains
	// invalid forever.
	Valid() bool

	// Next moves the iterator to the next key in the database, as defined by order of iteration.
	// If Valid returns false, this method will panic.
	Next()

	// Key returns the key at the current position. Panics if the iterator is invalid.
	// CONTRACT: key readonly []byte
	Key() (key []byte)

	// Value returns the value at the current position. Panics if the iterator is invalid.
	// CONTRACT: value readonly []byte
	Value() (value []byte)

	// Error returns the last error encountered by the iterator, if any.
	Error() error

	// Close closes the iterator, relasing any allocated resources.
	Close() error
}
