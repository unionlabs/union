package listenkv

import (
	"io"

	"cosmossdk.io/store/types"
)

var _ types.KVStore = &Store{}

// Store implements the KVStore interface with listening enabled.
// Operations are traced on each core KVStore call and written to any of the
// underlying listeners with the proper key and operation permissions
type Store struct {
	parent         types.KVStore
	listener       *types.MemoryListener
	parentStoreKey types.StoreKey
}

// NewStore returns a reference to a new traceKVStore given a parent
// KVStore implementation and a buffered writer.
func NewStore(parent types.KVStore, parentStoreKey types.StoreKey, listener *types.MemoryListener) *Store {
	return &Store{parent: parent, listener: listener, parentStoreKey: parentStoreKey}
}

// Get implements the KVStore interface. It traces a read operation and
// delegates a Get call to the parent KVStore.
func (s *Store) Get(key []byte) []byte {
	value := s.parent.Get(key)
	return value
}

// Set implements the KVStore interface. It traces a write operation and
// delegates the Set call to the parent KVStore.
func (s *Store) Set(key, value []byte) {
	types.AssertValidKey(key)
	s.parent.Set(key, value)
	s.listener.OnWrite(s.parentStoreKey, key, value, false)
}

// Delete implements the KVStore interface. It traces a write operation and
// delegates the Delete call to the parent KVStore.
func (s *Store) Delete(key []byte) {
	s.parent.Delete(key)
	s.listener.OnWrite(s.parentStoreKey, key, nil, true)
}

// Has implements the KVStore interface. It delegates the Has call to the
// parent KVStore.
func (s *Store) Has(key []byte) bool {
	return s.parent.Has(key)
}

// Iterator implements the KVStore interface. It delegates the Iterator call
// the to the parent KVStore.
func (s *Store) Iterator(start, end []byte) types.Iterator {
	return s.iterator(start, end, true)
}

// ReverseIterator implements the KVStore interface. It delegates the
// ReverseIterator call the to the parent KVStore.
func (s *Store) ReverseIterator(start, end []byte) types.Iterator {
	return s.iterator(start, end, false)
}

// iterator facilitates iteration over a KVStore. It delegates the necessary
// calls to it's parent KVStore.
func (s *Store) iterator(start, end []byte, ascending bool) types.Iterator {
	var parent types.Iterator

	if ascending {
		parent = s.parent.Iterator(start, end)
	} else {
		parent = s.parent.ReverseIterator(start, end)
	}

	return newTraceIterator(parent, s.listener)
}

type listenIterator struct {
	parent   types.Iterator
	listener *types.MemoryListener
}

func newTraceIterator(parent types.Iterator, listener *types.MemoryListener) types.Iterator {
	return &listenIterator{parent: parent, listener: listener}
}

// Domain implements the Iterator interface.
func (li *listenIterator) Domain() (start, end []byte) {
	return li.parent.Domain()
}

// Valid implements the Iterator interface.
func (li *listenIterator) Valid() bool {
	return li.parent.Valid()
}

// Next implements the Iterator interface.
func (li *listenIterator) Next() {
	li.parent.Next()
}

// Key implements the Iterator interface.
func (li *listenIterator) Key() []byte {
	key := li.parent.Key()
	return key
}

// Value implements the Iterator interface.
func (li *listenIterator) Value() []byte {
	value := li.parent.Value()
	return value
}

// Close implements the Iterator interface.
func (li *listenIterator) Close() error {
	return li.parent.Close()
}

// Error delegates the Error call to the parent iterator.
func (li *listenIterator) Error() error {
	return li.parent.Error()
}

// GetStoreType implements the KVStore interface. It returns the underlying
// KVStore type.
func (s *Store) GetStoreType() types.StoreType {
	return s.parent.GetStoreType()
}

// CacheWrap implements the KVStore interface. It panics as a Store
// cannot be cache wrapped.
func (s *Store) CacheWrap() types.CacheWrap {
	panic("cannot CacheWrap a ListenKVStore")
}

// CacheWrapWithTrace implements the KVStore interface. It panics as a
// Store cannot be cache wrapped.
func (s *Store) CacheWrapWithTrace(_ io.Writer, _ types.TraceContext) types.CacheWrap {
	panic("cannot CacheWrapWithTrace a ListenKVStore")
}
