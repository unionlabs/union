package testdb

import (
	"bytes"
	"fmt"
	"sync"

	"github.com/google/btree"
)

const (
	// The approximate number of items and children per B-tree node. Tuned with benchmarks.
	bTreeDegree = 32
)

// item is a btree.Item with byte slices as keys and values
type item struct {
	key   []byte
	value []byte
}

// Less implements btree.Item.
func (i *item) Less(other btree.Item) bool {
	// this considers nil == []byte{}, but that's ok since we handle nil endpoints
	// in iterators specially anyway
	return bytes.Compare(i.key, other.(*item).key) == -1
}

// newKey creates a new key item.
func newKey(key []byte) *item {
	return &item{key: key}
}

// newPair creates a new pair item.
func newPair(key, value []byte) *item {
	return &item{key: key, value: value}
}

// MemDB is an in-memory database backend using a B-tree for storage.
//
// For performance reasons, all given and returned keys and values are pointers to the in-memory
// database, so modifying them will cause the stored values to be modified as well. All DB methods
// already specify that keys and values should be considered read-only, but this is especially
// important with MemDB.
type MemDB struct {
	mtx   sync.RWMutex
	btree *btree.BTree
}

// NewMemDB creates a new in-memory database.
func NewMemDB() *MemDB {
	database := &MemDB{
		btree: btree.New(bTreeDegree),
	}
	return database
}

// Get implements DB.
func (db *MemDB) Get(key []byte) ([]byte, error) {
	if len(key) == 0 {
		return nil, errKeyEmpty
	}
	db.mtx.RLock()
	defer db.mtx.RUnlock()

	i := db.btree.Get(newKey(key))
	if i != nil {
		return i.(*item).value, nil
	}
	return nil, nil
}

// Has implements DB.
func (db *MemDB) Has(key []byte) (bool, error) {
	if len(key) == 0 {
		return false, errKeyEmpty
	}
	db.mtx.RLock()
	defer db.mtx.RUnlock()

	return db.btree.Has(newKey(key)), nil
}

// Set implements DB.
func (db *MemDB) Set(key []byte, value []byte) error {
	if len(key) == 0 {
		return errKeyEmpty
	}
	if value == nil {
		return errValueNil
	}
	db.mtx.Lock()
	defer db.mtx.Unlock()

	db.set(key, value)
	return nil
}

// set sets a value without locking the mutex.
func (db *MemDB) set(key []byte, value []byte) {
	db.btree.ReplaceOrInsert(newPair(key, value))
}

// SetSync implements DB.
func (db *MemDB) SetSync(key []byte, value []byte) error {
	return db.Set(key, value)
}

// Delete implements DB.
func (db *MemDB) Delete(key []byte) error {
	if len(key) == 0 {
		return errKeyEmpty
	}
	db.mtx.Lock()
	defer db.mtx.Unlock()

	db.delete(key)
	return nil
}

// delete deletes a key without locking the mutex.
func (db *MemDB) delete(key []byte) {
	db.btree.Delete(newKey(key))
}

// DeleteSync implements DB.
func (db *MemDB) DeleteSync(key []byte) error {
	return db.Delete(key)
}

// Close implements DB.
func (db *MemDB) Close() error {
	// Close is a noop since for an in-memory database, we don't have a destination to flush
	// contents to nor do we want any data loss on invoking Close().
	// See the discussion in https://github.com/tendermint/tendermint/libs/pull/56
	return nil
}

// Print implements DB.
func (db *MemDB) Print() error {
	db.mtx.RLock()
	defer db.mtx.RUnlock()

	db.btree.Ascend(func(i btree.Item) bool {
		item := i.(*item)
		fmt.Printf("[%X]:\t[%X]\n", item.key, item.value)
		return true
	})
	return nil
}

// Stats implements DB.
func (db *MemDB) Stats() map[string]string {
	db.mtx.RLock()
	defer db.mtx.RUnlock()

	stats := make(map[string]string)
	stats["database.type"] = "memDB"
	stats["database.size"] = fmt.Sprintf("%d", db.btree.Len())
	return stats
}

// Iterator implements DB.
// Takes out a read-lock on the database until the iterator is closed.
func (db *MemDB) Iterator(start, end []byte) (Iterator, error) {
	if (start != nil && len(start) == 0) || (end != nil && len(end) == 0) {
		return nil, errKeyEmpty
	}
	return newMemDBIterator(db, start, end, false), nil
}

// ReverseIterator implements DB.
// Takes out a read-lock on the database until the iterator is closed.
func (db *MemDB) ReverseIterator(start, end []byte) (Iterator, error) {
	if (start != nil && len(start) == 0) || (end != nil && len(end) == 0) {
		return nil, errKeyEmpty
	}
	return newMemDBIterator(db, start, end, true), nil
}

// IteratorNoMtx makes an iterator with no mutex.
func (db *MemDB) IteratorNoMtx(start, end []byte) (Iterator, error) {
	if (start != nil && len(start) == 0) || (end != nil && len(end) == 0) {
		return nil, errKeyEmpty
	}
	return newMemDBIteratorMtxChoice(db, start, end, false, false), nil
}

// ReverseIteratorNoMtx makes an iterator with no mutex.
func (db *MemDB) ReverseIteratorNoMtx(start, end []byte) (Iterator, error) {
	if (start != nil && len(start) == 0) || (end != nil && len(end) == 0) {
		return nil, errKeyEmpty
	}
	return newMemDBIteratorMtxChoice(db, start, end, true, false), nil
}
