package store

// KVStore describes the basic interface for interacting with key-value stores.
type KVStore interface {
	// Get returns nil iff key doesn't exist. Errors on nil key.
	Get(key []byte) ([]byte, error)

	// Has checks if a key exists. Errors on nil key.
	Has(key []byte) (bool, error)

	// Set sets the key. Errors on nil key or value.
	Set(key, value []byte) error

	// Delete deletes the key. Errors on nil key.
	Delete(key []byte) error

	// Iterator iterates over a domain of keys in ascending order. End is exclusive.
	// Start must be less than end, or the Iterator is invalid.
	// Iterator must be closed by caller.
	// To iterate over entire domain, use store.Iterator(nil, nil)
	// CONTRACT: No writes may happen within a domain while an iterator exists over it.
	// Exceptionally allowed for cachekv.Store, safe to write in the modules.
	Iterator(start, end []byte) (Iterator, error)

	// ReverseIterator iterates over a domain of keys in descending order. End is exclusive.
	// Start must be less than end, or the Iterator is invalid.
	// Iterator must be closed by caller.
	// CONTRACT: No writes may happen within a domain while an iterator exists over it.
	// Exceptionally allowed for cachekv.Store, safe to write in the modules.
	ReverseIterator(start, end []byte) (Iterator, error)
}

// Batch represents a group of writes. They may or may not be written atomically depending on the
// backend. Callers must call Close on the batch when done.
//
// As with KVStore, given keys and values should be considered read-only, and must not be modified after
// passing them to the batch.
type Batch = interface {
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

	// GetByteSize that returns the current size of the batch in bytes. Depending on the implementation,
	// this may return the size of the underlying LSM batch, including the size of additional metadata
	// on top of the expected key and value total byte count.
	GetByteSize() (int, error)
}

// BatchCreator defines an interface for creating a new batch.
type BatchCreator interface {
	// NewBatch creates a new batch for atomic updates. The caller must call Batch.Close.
	NewBatch() Batch

	// NewBatchWithSize create a new batch for atomic updates, but with pre-allocated size.
	// This will does the same thing as NewBatch if the batch implementation doesn't support pre-allocation.
	NewBatchWithSize(int) Batch
}

// KVStoreWithBatch is an extension of KVStore that allows for batch writes.
type KVStoreWithBatch interface {
	KVStore
	BatchCreator

	// Close closes the KVStoreWithBatch, releasing any resources held.
	Close() error
}

// Iterator represents an iterator over a domain of keys. Callers must call
// Close when done. No writes can happen to a domain while there exists an
// iterator over it. Some backends may take out database locks to ensure this
// will not happen.
//
// Callers must make sure the iterator is valid before calling any methods on it,
// otherwise these methods will panic.
type Iterator = interface {
	// Domain returns the start (inclusive) and end (exclusive) limits of the iterator.
	Domain() (start, end []byte)

	// Valid returns whether the current iterator is valid. Once invalid, the Iterator remains
	// invalid forever.
	Valid() bool

	// Next moves the iterator to the next key in the database, as defined by order of iteration.
	// If Valid returns false, this method will panic.
	Next()

	// Key returns the key at the current position. Panics if the iterator is invalid.
	// Note, the key returned should be a copy and thus safe for modification.
	Key() []byte

	// Value returns the value at the current position. Panics if the iterator is
	// invalid.
	// Note, the value returned should be a copy and thus safe for modification.
	Value() []byte

	// Error returns the last error encountered by the iterator, if any.
	Error() error

	// Close closes the iterator, releasing any allocated resources.
	Close() error
}

// IteratorCreator defines an interface for creating forward and reverse iterators.
type IteratorCreator interface {
	// Iterator creates a new iterator for the given store name and domain, where
	// domain is defined by [start, end). Note, both start and end are optional.
	Iterator(storeKey string, start, end []byte) (Iterator, error)

	// ReverseIterator creates a new reverse iterator for the given store name
	// and domain, where domain is defined by [start, end). Note, both start and
	// end are optional.
	ReverseIterator(storeKey string, start, end []byte) (Iterator, error)
}

type Hash = []byte

var _ KVStore = (Writer)(nil)

// ReaderMap represents a readonly view over all the accounts state.
type ReaderMap interface {
	// GetReader must return the state for the provided actor.
	// Storage implements might treat this as a prefix store over an actor.
	// Prefix safety is on the implementer.
	GetReader(actor []byte) (Reader, error)
}

// WriterMap represents a writable actor state.
type WriterMap interface {
	ReaderMap
	// GetWriter must the return a WritableState
	// for the provided actor namespace.
	GetWriter(actor []byte) (Writer, error)
	// ApplyStateChanges applies all the state changes
	// of the accounts. Ordering of the returned state changes
	// is an implementation detail and must not be assumed.
	ApplyStateChanges(stateChanges []StateChanges) error
	// GetStateChanges returns the list of the state
	// changes so far applied. Order must not be assumed.
	GetStateChanges() ([]StateChanges, error)
}

// Writer defines an instance of an actor state at a specific version that can be written to.
type Writer interface {
	Reader
	Set(key, value []byte) error
	Delete(key []byte) error
	ApplyChangeSets(changes []KVPair) error
	ChangeSets() ([]KVPair, error)
}

// Reader defines a sub-set of the methods exposed by store.KVStore.
// The methods defined work only at read level.
type Reader interface {
	Has(key []byte) (bool, error)
	Get([]byte) ([]byte, error)
	Iterator(start, end []byte) (Iterator, error)        // consider removing iterate?
	ReverseIterator(start, end []byte) (Iterator, error) // consider removing reverse iterate
}
