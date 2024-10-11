package db

import (
	"bytes"
	"fmt"
	"sync"

	corestore "cosmossdk.io/core/store"
)

// PrefixDB wraps a namespace of another database as a logical database.
type PrefixDB struct {
	mtx    sync.Mutex
	prefix []byte
	db     corestore.KVStoreWithBatch
}

var _ corestore.KVStoreWithBatch = (*PrefixDB)(nil)

// NewPrefixDB lets you namespace multiple corestore.KVStores within a single corestore.KVStore.
func NewPrefixDB(db corestore.KVStoreWithBatch, prefix []byte) *PrefixDB {
	return &PrefixDB{
		prefix: prefix,
		db:     db,
	}
}

// Get implements corestore.KVStore.
func (pdb *PrefixDB) Get(key []byte) ([]byte, error) {
	if len(key) == 0 {
		return nil, ErrKeyEmpty
	}

	pkey := pdb.prefixed(key)
	value, err := pdb.db.Get(pkey)
	if err != nil {
		return nil, err
	}

	return value, nil
}

// Has implements corestore.KVStore.
func (pdb *PrefixDB) Has(key []byte) (bool, error) {
	if len(key) == 0 {
		return false, ErrKeyEmpty
	}

	ok, err := pdb.db.Has(pdb.prefixed(key))
	if err != nil {
		return ok, err
	}

	return ok, nil
}

// Set implements corestore.KVStore.
func (pdb *PrefixDB) Set(key, value []byte) error {
	if len(key) == 0 {
		return ErrKeyEmpty
	}

	return pdb.db.Set(pdb.prefixed(key), value)
}

// Delete implements corestore.KVStore.
func (pdb *PrefixDB) Delete(key []byte) error {
	if len(key) == 0 {
		return ErrKeyEmpty
	}

	return pdb.db.Delete(pdb.prefixed(key))
}

// Iterator implements corestore.KVStore.
func (pdb *PrefixDB) Iterator(start, end []byte) (corestore.Iterator, error) {
	if (start != nil && len(start) == 0) || (end != nil && len(end) == 0) {
		return nil, ErrKeyEmpty
	}

	var pstart, pend []byte
	pstart = append(cp(pdb.prefix), start...)
	if end == nil {
		pend = cpIncr(pdb.prefix)
	} else {
		pend = append(cp(pdb.prefix), end...)
	}
	itr, err := pdb.db.Iterator(pstart, pend)
	if err != nil {
		return nil, err
	}

	return newPrefixIterator(pdb.prefix, start, end, itr)
}

// ReverseIterator implements corestore.KVStore.
func (pdb *PrefixDB) ReverseIterator(start, end []byte) (corestore.Iterator, error) {
	if (start != nil && len(start) == 0) || (end != nil && len(end) == 0) {
		return nil, ErrKeyEmpty
	}

	var pstart, pend []byte
	pstart = append(cp(pdb.prefix), start...)
	if end == nil {
		pend = cpIncr(pdb.prefix)
	} else {
		pend = append(cp(pdb.prefix), end...)
	}
	ritr, err := pdb.db.ReverseIterator(pstart, pend)
	if err != nil {
		return nil, err
	}

	return newPrefixIterator(pdb.prefix, start, end, ritr)
}

// NewBatch implements corestore.BatchCreator.
func (pdb *PrefixDB) NewBatch() corestore.Batch {
	return newPrefixBatch(pdb.prefix, pdb.db.NewBatch())
}

// NewBatchWithSize implements corestore.BatchCreator.
func (pdb *PrefixDB) NewBatchWithSize(size int) corestore.Batch {
	return newPrefixBatch(pdb.prefix, pdb.db.NewBatchWithSize(size))
}

// Close implements corestore.KVStore.
func (pdb *PrefixDB) Close() error {
	pdb.mtx.Lock()
	defer pdb.mtx.Unlock()

	return pdb.db.Close()
}

// Print implements corestore.KVStore.
func (pdb *PrefixDB) Print() error {
	fmt.Printf("prefix: %X\n", pdb.prefix)

	itr, err := pdb.Iterator(nil, nil)
	if err != nil {
		return err
	}
	defer itr.Close()
	for ; itr.Valid(); itr.Next() {
		key := itr.Key()
		value := itr.Value()
		fmt.Printf("[%X]:\t[%X]\n", key, value)
	}
	return nil
}

func (pdb *PrefixDB) prefixed(key []byte) []byte {
	return append(cp(pdb.prefix), key...)
}

// IteratePrefix is a convenience function for iterating over a key domain
// restricted by prefix.
func IteratePrefix(db corestore.KVStore, prefix []byte) (corestore.Iterator, error) {
	var start, end []byte
	if len(prefix) == 0 {
		start = nil
		end = nil
	} else {
		start = cp(prefix)
		end = cpIncr(prefix)
	}
	itr, err := db.Iterator(start, end)
	if err != nil {
		return nil, err
	}
	return itr, nil
}

// Strips prefix while iterating from Iterator.
type prefixDBIterator struct {
	prefix []byte
	start  []byte
	end    []byte
	source corestore.Iterator
	valid  bool
	err    error
}

var _ corestore.Iterator = (*prefixDBIterator)(nil)

func newPrefixIterator(prefix, start, end []byte, source corestore.Iterator) (*prefixDBIterator, error) {
	pitrInvalid := &prefixDBIterator{
		prefix: prefix,
		start:  start,
		end:    end,
		source: source,
		valid:  false,
	}

	// Empty keys are not allowed, so if a key exists in the database that exactly matches the
	// prefix we need to skip it.
	if source.Valid() && bytes.Equal(source.Key(), prefix) {
		source.Next()
	}

	if !source.Valid() || !bytes.HasPrefix(source.Key(), prefix) {
		return pitrInvalid, nil
	}

	return &prefixDBIterator{
		prefix: prefix,
		start:  start,
		end:    end,
		source: source,
		valid:  true,
	}, nil
}

// Domain implements Iterator.
func (itr *prefixDBIterator) Domain() (start, end []byte) {
	return itr.start, itr.end
}

// Valid implements Iterator.
func (itr *prefixDBIterator) Valid() bool {
	if !itr.valid || itr.err != nil || !itr.source.Valid() {
		return false
	}

	key := itr.source.Key()
	if len(key) < len(itr.prefix) || !bytes.Equal(key[:len(itr.prefix)], itr.prefix) {
		itr.err = fmt.Errorf("received invalid key from backend: %x (expected prefix %x)",
			key, itr.prefix)
		return false
	}

	return true
}

// Next implements Iterator.
func (itr *prefixDBIterator) Next() {
	itr.assertIsValid()
	itr.source.Next()

	if !itr.source.Valid() || !bytes.HasPrefix(itr.source.Key(), itr.prefix) {
		itr.valid = false
	} else if bytes.Equal(itr.source.Key(), itr.prefix) {
		// Empty keys are not allowed, so if a key exists in the database that exactly matches the
		// prefix we need to skip it.
		itr.Next()
	}
}

// Key implements Iterator.
func (itr *prefixDBIterator) Key() []byte {
	itr.assertIsValid()
	key := itr.source.Key()
	return key[len(itr.prefix):] // we have checked the key in Valid()
}

// Value implements Iterator.
func (itr *prefixDBIterator) Value() []byte {
	itr.assertIsValid()
	return itr.source.Value()
}

// Error implements Iterator.
func (itr *prefixDBIterator) Error() error {
	if err := itr.source.Error(); err != nil {
		return err
	}
	return itr.err
}

// Close implements Iterator.
func (itr *prefixDBIterator) Close() error {
	return itr.source.Close()
}

func (itr *prefixDBIterator) assertIsValid() {
	if !itr.Valid() {
		panic("iterator is invalid")
	}
}

type prefixDBBatch struct {
	prefix []byte
	source corestore.Batch
}

var _ corestore.Batch = (*prefixDBBatch)(nil)

func newPrefixBatch(prefix []byte, source corestore.Batch) prefixDBBatch {
	return prefixDBBatch{
		prefix: prefix,
		source: source,
	}
}

// Set implements corestore.Batch.
func (pb prefixDBBatch) Set(key, value []byte) error {
	if len(key) == 0 {
		return ErrKeyEmpty
	}
	if value == nil {
		return ErrValueNil
	}
	pkey := append(cp(pb.prefix), key...)
	return pb.source.Set(pkey, value)
}

// Delete implements corestore.Batch.
func (pb prefixDBBatch) Delete(key []byte) error {
	if len(key) == 0 {
		return ErrKeyEmpty
	}
	pkey := append(cp(pb.prefix), key...)
	return pb.source.Delete(pkey)
}

// Write implements corestore.Batch.
func (pb prefixDBBatch) Write() error {
	return pb.source.Write()
}

// WriteSync implements corestore.Batch.
func (pb prefixDBBatch) WriteSync() error {
	return pb.source.WriteSync()
}

// Close implements corestore.Batch.
func (pb prefixDBBatch) Close() error {
	return pb.source.Close()
}

// GetByteSize implements corestore.Batch
func (pb prefixDBBatch) GetByteSize() (int, error) {
	if pb.source == nil {
		return 0, ErrBatchClosed
	}
	return pb.source.GetByteSize()
}
