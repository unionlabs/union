package db

import (
	"fmt"
	"sync"
)

// PrefixDB wraps a namespace of another database as a logical database.
type PrefixDB struct {
	mtx    sync.Mutex
	prefix []byte
	db     DB
}

var _ DB = (*PrefixDB)(nil)

// NewPrefixDB lets you namespace multiple DBs within a single DB.
func NewPrefixDB(db DB, prefix []byte) *PrefixDB {
	return &PrefixDB{
		prefix: prefix,
		db:     db,
	}
}

// Get implements DB.
func (pdb *PrefixDB) Get(key []byte) ([]byte, error) {
	if len(key) == 0 {
		return nil, errKeyEmpty
	}
	pdb.mtx.Lock()
	defer pdb.mtx.Unlock()

	pkey := pdb.prefixed(key)
	value, err := pdb.db.Get(pkey)
	if err != nil {
		return nil, err
	}
	return value, nil
}

// Has implements DB.
func (pdb *PrefixDB) Has(key []byte) (bool, error) {
	if len(key) == 0 {
		return false, errKeyEmpty
	}
	pdb.mtx.Lock()
	defer pdb.mtx.Unlock()

	ok, err := pdb.db.Has(pdb.prefixed(key))
	if err != nil {
		return ok, err
	}

	return ok, nil
}

// Set implements DB.
func (pdb *PrefixDB) Set(key []byte, value []byte) error {
	if len(key) == 0 {
		return errKeyEmpty
	}
	if value == nil {
		return errValueNil
	}
	pdb.mtx.Lock()
	defer pdb.mtx.Unlock()

	pkey := pdb.prefixed(key)
	if err := pdb.db.Set(pkey, value); err != nil {
		return err
	}
	return nil
}

// SetSync implements DB.
func (pdb *PrefixDB) SetSync(key []byte, value []byte) error {
	if len(key) == 0 {
		return errKeyEmpty
	}
	if value == nil {
		return errValueNil
	}
	pdb.mtx.Lock()
	defer pdb.mtx.Unlock()

	return pdb.db.SetSync(pdb.prefixed(key), value)
}

// Delete implements DB.
func (pdb *PrefixDB) Delete(key []byte) error {
	if len(key) == 0 {
		return errKeyEmpty
	}
	pdb.mtx.Lock()
	defer pdb.mtx.Unlock()

	return pdb.db.Delete(pdb.prefixed(key))
}

// DeleteSync implements DB.
func (pdb *PrefixDB) DeleteSync(key []byte) error {
	if len(key) == 0 {
		return errKeyEmpty
	}
	pdb.mtx.Lock()
	defer pdb.mtx.Unlock()

	return pdb.db.DeleteSync(pdb.prefixed(key))
}

// Iterator implements DB.
func (pdb *PrefixDB) Iterator(start, end []byte) (Iterator, error) {
	if (start != nil && len(start) == 0) || (end != nil && len(end) == 0) {
		return nil, errKeyEmpty
	}
	pdb.mtx.Lock()
	defer pdb.mtx.Unlock()

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

// ReverseIterator implements DB.
func (pdb *PrefixDB) ReverseIterator(start, end []byte) (Iterator, error) {
	if (start != nil && len(start) == 0) || (end != nil && len(end) == 0) {
		return nil, errKeyEmpty
	}
	pdb.mtx.Lock()
	defer pdb.mtx.Unlock()

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

// NewBatch implements DB.
func (pdb *PrefixDB) NewBatch() Batch {
	pdb.mtx.Lock()
	defer pdb.mtx.Unlock()

	return newPrefixBatch(pdb.prefix, pdb.db.NewBatch())
}

// Close implements DB.
func (pdb *PrefixDB) Close() error {
	pdb.mtx.Lock()
	defer pdb.mtx.Unlock()

	return pdb.db.Close()
}

// Print implements DB.
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

// Stats implements DB.
func (pdb *PrefixDB) Stats() map[string]string {
	stats := make(map[string]string)
	stats["prefixdb.prefix.string"] = string(pdb.prefix)
	stats["prefixdb.prefix.hex"] = fmt.Sprintf("%X", pdb.prefix)
	source := pdb.db.Stats()
	for key, value := range source {
		stats["prefixdb.source."+key] = value
	}
	return stats
}

func (pdb *PrefixDB) prefixed(key []byte) []byte {
	return append(cp(pdb.prefix), key...)
}
